use std::sync::Arc;
use tokio::signal;
use tracing::{info, warn};

use arb_config::Config;
use arb_metrics::MetricsRegistry;
use arb_providers::ProviderManager;
use arb_ingest::IngestPipeline;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Basic startup logs
    tracing_subscriber::fmt::init();
    info!("Starting ArbHunter Daemon (Phase 2 Foundation)...");

    // 1. Config Loading
    let config = Config::load();
    info!("Configuration loaded for Chain ID: {}", config.chain_id);
    if config.enable_failover {
        info!("Failover provider is enabled.");
    }

    // 2. Metrics Startup
    let metrics = Arc::new(MetricsRegistry::new());
    info!("Metrics registry initialized on port: {}", config.metrics_port);

    // 3. Provider and Ingest Wiring
    let ingest_pipeline = Arc::new(IngestPipeline::new(1024));
    let mut event_rx = ingest_pipeline.subscribe();
    info!("Ingest pipeline initialized.");

    // The Provider->Ingest MPSC bridge channel
    let (provider_tx, mut provider_rx) = tokio::sync::mpsc::channel::<String>(1000);

    let mut provider_manager = ProviderManager::new(&config, metrics.clone());
    info!("Provider manager initialized. Primary: QuickNode.");
    
    // Start provider loop (in background)
    tokio::spawn(async move {
        provider_manager.start_management_loop(provider_tx).await;
    });

    // 4. Ingest Consumer Bridge: Live Websocket frames to Ingest
    let ingest_clone = ingest_pipeline.clone();
    tokio::spawn(async move {
        while let Some(raw_json) = provider_rx.recv().await {
            // Forward raw websocket payload to structured ingestion
            ingest_clone.handle_raw_payload(&raw_json);
        }
    });

    // Simple consumer to prove structural conversion success
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            info!("Successfully normalized structured event: {:?}", event);
        }
    });

    // 5. Graceful Shutdown
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received shutdown signal. Commencing graceful shutdown.");
        }
        Err(err) => {
            warn!("Unable to listen for shutdown signal: {}", err);
        }
    }

    info!("ArbHunter Daemon shutdown successfully.");
    Ok(())
}
