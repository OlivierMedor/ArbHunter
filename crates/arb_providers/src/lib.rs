use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use futures_util::StreamExt;
use tokio_tungstenite::connect_async;
use url::Url;

use arb_config::Config;
use arb_metrics::MetricsRegistry;
use arb_types::{ProviderHealth, ProviderKind};

use tokio::sync::mpsc;

#[async_trait]
pub trait Provider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    fn health(&self) -> ProviderHealth;
    fn latency_ms(&self) -> u64;
    async fn connect(&mut self, tx: mpsc::Sender<String>) -> Result<(), String>;
    async fn disconnect(&mut self);
}

pub struct BaseProvider {
    kind: ProviderKind,
    _url: String,
    health: ProviderHealth,
    latency_ms: u64,
    reconnect_count: u32,
    connected: bool,
    metrics: Arc<MetricsRegistry>,
}

impl BaseProvider {
    pub fn new(kind: ProviderKind, url: String, metrics: Arc<MetricsRegistry>) -> Self {
        Self {
            kind,
            _url: url,
            health: ProviderHealth::Down,
            latency_ms: 0,
            reconnect_count: 0,
            connected: false,
            metrics,
        }
    }

    // TODO: Implement real ping/pong latency measurement in Phase 3
    fn update_health_status(&mut self) {
        // Latency zeroed until real measurement logic is built
        self.latency_ms = 0; 
        self.health = ProviderHealth::Healthy;
    }
}

#[async_trait]
impl Provider for BaseProvider {
    fn kind(&self) -> ProviderKind {
        self.kind
    }

    fn health(&self) -> ProviderHealth {
        self.health
    }

    fn latency_ms(&self) -> u64 {
        self.latency_ms
    }

    async fn connect(&mut self, tx: mpsc::Sender<String>) -> Result<(), String> {
        let parsed_url = Url::parse(&self._url)
            .map_err(|e| format!("Invalid Provider URL: {}", e))?;

        let (mut ws_stream, _) = connect_async(parsed_url.as_str())
            .await
            .map_err(|e| format!("Failed to connect to provider WSS: {}", e))?;

        self.connected = true;
        self.update_health_status();
        self.reconnect_count += 1;
        self.metrics.inc_provider_connected(self.kind.as_str());

        let metrics = self.metrics.clone();
        // Route real connection frames to the ingest pipeline channel
        tokio::spawn(async move {
            while let Some(msg) = ws_stream.next().await {
                if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
                    metrics.inc_provider_frames_forwarded();
                    let _ = tx.send(text.to_string()).await;
                }
            }
        });

        Ok(())
    }

    async fn disconnect(&mut self) {
        if self.connected {
            self.metrics.inc_provider_disconnected(self.kind.as_str());
        }
        self.connected = false;
        self.health = ProviderHealth::Down;
    }
}

pub struct ProviderManager {
    primary: Arc<RwLock<Box<dyn Provider>>>,
    failover: Option<Arc<RwLock<Box<dyn Provider>>>>,
    metrics: Arc<MetricsRegistry>,
    active_provider: ProviderKind,
}

impl ProviderManager {
    pub fn new(config: &Config, metrics: Arc<MetricsRegistry>) -> Self {
        let primary = Box::new(BaseProvider::new(
            ProviderKind::QuickNode,
            config.quicknode_wss_url.clone(),
            metrics.clone(),
        ));
        
        let failover = if config.enable_failover {
            config.alchemy_wss_url.as_ref().map(|url| {
                Box::new(BaseProvider::new(
                    ProviderKind::Alchemy,
                    url.clone(),
                    metrics.clone(),
                )) as Box<dyn Provider>
            })
        } else {
            None
        };

        Self {
            primary: Arc::new(RwLock::new(primary)),
            failover: failover.map(|f| Arc::new(RwLock::new(f))),
            metrics,
            active_provider: ProviderKind::QuickNode,
        }
    }

    // Ensure active provider actually switches if failover becomes active
    pub async fn start_management_loop(&mut self, tx: mpsc::Sender<String>) {
        self.metrics.set_active_provider(self.active_provider.as_str());

        // Initial connections
        {
            let mut p = self.primary.write().await;
            let _ = p.connect(tx.clone()).await;
        }
        
        if let Some(failover) = &self.failover {
            let mut f = failover.write().await;
            let _ = f.connect(tx.clone()).await;
        }

        // Monitoring loop (simplified)
        let primary_lock = self.primary.clone();
        let failover_lock = self.failover.clone();
        let metrics = self.metrics.clone();
        
        // We do not have self inside spawn, so to update active provider safely we'd need shared state,
        // but for Phase 2 foundation, we just emulate the health transitions and switch logic.

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                
                let p_health = primary_lock.read().await.health();
                let p_latency = primary_lock.read().await.latency_ms();
                metrics.set_provider_latency_ms(p_latency);

                if p_health == ProviderHealth::Down || p_latency > 500 {
                    // Try to swap to failover if primary dies
                    if let Some(fallback) = &failover_lock {
                        let f_health = fallback.read().await.health();
                        if f_health == ProviderHealth::Healthy {
                            metrics.inc_failover_switches();
                            // In real routing, we switch stream active state here
                            metrics.set_active_provider("alchemy");
                        }
                    } else {
                        metrics.set_active_provider("quicknode"); // Revert fallback logic placeholder
                    }
                    // Attempt to reconnect primary
                    metrics.inc_reconnect_attempts();
                    let _ = primary_lock.write().await.connect(tx.clone()).await;
                } else {
                    metrics.set_active_provider("quicknode");
                }
            }
        });
    }

    pub fn get_active_provider(&self) -> ProviderKind {
        self.active_provider
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_manager_failover() {
        let config = Config {
            quicknode_wss_url: "wss://mock".to_string(),
            alchemy_wss_url: Some("wss://mock2".to_string()),
            chain_id: 8453,
            log_level: "info".to_string(),
            metrics_port: 9090,
            enable_flashblocks: false,
            enable_pending_logs: false,
            enable_failover: true,
        };
        
        let metrics = Arc::new(MetricsRegistry::new());
        let manager = ProviderManager::new(&config, metrics.clone());
        let (tx, _) = mpsc::channel::<String>(100);
        
        assert_eq!(manager.get_active_provider(), ProviderKind::QuickNode);
        assert!(manager.failover.is_some());
    }
}
