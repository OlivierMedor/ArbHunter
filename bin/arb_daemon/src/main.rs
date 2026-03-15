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
/// Phase 4: Uses real DEX event decoding for PendingLog events.
fn ingest_to_pool_update(event: &IngestEvent, pipeline: &IngestPipeline) -> Option<PoolUpdate> {
    match event {
        IngestEvent::Flashblock(_fb) => {
            // Phase 4: Synthetic Flashblock state updates are disabled by default.
            // Only real DEX logs drive the state engine now.
            None
        }
        IngestEvent::PendingLog(pl) => {
            // Use the real DEX decoder from the pipeline
            pipeline.decoder.decode_log(pl)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting ArbHunter Daemon (Phase 4 Real DEX Event Decoding)...");

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
    let ingest_for_bridge = ingest_pipeline.clone();
    tokio::spawn(async move {
        let mut update_count = 0u64;
        while let Ok(event) = event_rx.recv().await {
            if let Some(pool_update) = ingest_to_pool_update(&event, &ingest_for_bridge) {
                engine_for_bridge.apply(pool_update).await;
                update_count += 1;
                if update_count % 100 == 0 {
                    info!(
                        "State engine: {} real updates applied, {} pools tracked",
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

    // Phase 3.5: Support REPLAY_FIXTURE for validation
    if let Ok(fixture_path) = std::env::var("REPLAY_FIXTURE") {
        info!("Replay trigger detected: {}", fixture_path);
        let harness = arb_ingest::ReplayHarness::new(fixture_path);
        if let Err(e) = harness.run_replay(&ingest_pipeline).await {
            warn!("Replay harness failed: {}", e);
        } else {
            info!("Replay completion signal sent to pipeline.");
        }
    }

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


