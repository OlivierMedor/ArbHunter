use axum::{extract::State, routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use arb_config::Config;
use arb_ingest::IngestPipeline;
use arb_metrics::MetricsRegistry;
use arb_providers::ProviderManager;
use arb_state::StateEngine;
use arb_types::{
    EventStamp, IngestEvent, PoolId, PoolKind, PoolUpdate, ReserveSnapshot, TokenAddress,
};

async fn metrics_handler(State(metrics): State<Arc<MetricsRegistry>>) -> String {
    metrics.gather_metrics()
}

/// Convert a normalized `IngestEvent` into a `PoolUpdate` if it carries state-relevant data.
/// For Phase 3, only Flashblock events are modelled as pool state inputs.
/// PendingLog events are deliberately ignored for pool state (routing-free design).
fn ingest_to_pool_update(event: &IngestEvent) -> Option<PoolUpdate> {
    match event {
        IngestEvent::Flashblock(fb) => {
            // Phase 3: derive a synthetic pool update from the block's base-fee context.
            // In Phase 4+ this will be replaced by real Sync/Swap log parsing.
            // For now we model one canonical "block-level" entry so the state engine
            // demonstrates apply/freshness/metrics without fake data.
            Some(PoolUpdate {
                pool_id: PoolId(format!("block:{}", fb.block_number)),
                kind: PoolKind::Unknown,
                token0: TokenAddress("0x0000000000000000000000000000000000000000".to_string()),
                token1: TokenAddress("0x0000000000000000000000000000000000000000".to_string()),
                reserves: Some(ReserveSnapshot {
                    reserve0: fb.base_fee_per_gas as u128,
                    reserve1: fb.transaction_count as u128,
                }),
                stamp: EventStamp {
                    block_number: fb.block_number,
                    log_index: 0,
                },
            })
        }
        IngestEvent::PendingLog(_) => None, // No pool state from raw pending logs in Phase 3
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting ArbHunter Daemon (Phase 3 State Engine)...");

    let config = Config::load();
    info!("Configuration loaded for Chain ID: {}", config.chain_id);

    let metrics = Arc::new(MetricsRegistry::new());
    info!("Metrics registry initialized on port: {}", config.metrics_port);

    // Start Metrics HTTP Server
    let metrics_state = metrics.clone();
    let metrics_port = config.metrics_port;
    tokio::spawn(async move {
        let app = Router::new()
            .route("/metrics", get(metrics_handler))
            .with_state(metrics_state);
        let addr = SocketAddr::from(([0, 0, 0, 0], metrics_port));
        if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
            info!("Serving Prometheus metrics at http://127.0.0.1:{}/metrics", metrics_port);
            let _ = axum::serve(listener, app).await;
        } else {
            warn!("Failed to bind metrics port {}", metrics_port);
        }
    });

    // Ingest pipeline
    let ingest_pipeline = Arc::new(IngestPipeline::new(1024, metrics.clone()));
    let mut event_rx = ingest_pipeline.subscribe();
    info!("Ingest pipeline initialized.");

    // Provider → Ingest MPSC bridge
    let (provider_tx, mut provider_rx) = tokio::sync::mpsc::channel::<String>(1000);
    let mut provider_manager = ProviderManager::new(&config, metrics.clone());
    info!("Provider manager initialized. Primary: QuickNode.");

    tokio::spawn(async move {
        provider_manager.start_management_loop(provider_tx).await;
    });

    // Forward raw WSS frames to the ingest pipeline
    let ingest_clone = ingest_pipeline.clone();
    tokio::spawn(async move {
        while let Some(raw_json) = provider_rx.recv().await {
            ingest_clone.handle_raw_payload(&raw_json);
        }
    });

    // State Engine
    let state_engine = Arc::new(StateEngine::new(metrics.clone()));
    info!("State engine initialized.");

    // Ingest → State bridge: consume broadcast IngestEvents and apply to state
    let engine_for_bridge = state_engine.clone();
    tokio::spawn(async move {
        let mut update_count = 0u64;
        while let Ok(event) = event_rx.recv().await {
            if let Some(pool_update) = ingest_to_pool_update(&event) {
                engine_for_bridge.apply(pool_update).await;
                update_count += 1;
                if update_count % 100 == 0 {
                    info!(
                        "State engine: {} updates applied, {} pools tracked",
                        update_count,
                        engine_for_bridge.pool_count().await
                    );
                }
            }
        }
    });

    // Freshness tick: refresh staleness every 10 seconds
    let engine_for_tick = state_engine.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await;
            engine_for_tick.tick_freshness().await;
        }
    });

    // Graceful shutdown
    match signal::ctrl_c().await {
        Ok(()) => info!("Received shutdown signal. Commencing graceful shutdown."),
        Err(err) => warn!("Unable to listen for shutdown signal: {}", err),
    }

    info!(
        "Shutdown. Final state: {} pool(s) tracked.",
        state_engine.pool_count().await
    );
    Ok(())
}


