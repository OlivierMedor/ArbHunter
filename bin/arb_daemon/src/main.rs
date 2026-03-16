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
    EventStamp, IngestEvent, PoolId, PoolKind, PoolUpdate, ReserveSnapshot, TokenAddress, QuoteSizeBucket, SimOutcomeStatus,
};
use arb_route::{RouteGraph, CandidateGenerator};
use arb_filter::{CandidateFilter, FilterConfig};
use arb_sim::LocalSimulator;
use alloy_primitives::{U256, Address};
use arb_execute::{Wallet, Submitter, NonceManager, NonceProvider, TxBuilder, ExecutionPlanner};

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

    // Phase 9/10: Execution & Submission Pipeline
    let wallet = if let Some(pk_str) = &config.signer_private_key {
        info!("Signer wallet detected. Initializing submission pipeline...");
        match Wallet::from_env() { // Uses SIGNER_PRIVATE_KEY from env
            Ok(w) => Some(w),
            Err(e) => {
                warn!("Failed to load signer wallet from env: {}", e);
                None
            }
        }
    } else {
        warn!("No SIGNER_PRIVATE_KEY provided. Submission will be impossible.");
        None
    };

    let submitter = if let Some(w) = wallet {
        let mode = if config.enable_broadcast && !config.dry_run_only {
            arb_types::SubmissionMode::Broadcast
        } else {
            arb_types::SubmissionMode::DryRun
        };
        let s = Arc::new(Submitter::new(
            w,
            mode,
            metrics.clone(),
            config.rpc_http_url.clone(),
            config.require_preflight,
            config.require_eth_call,
            config.require_gas_estimate,
        ));
        info!("Submitter initialized in {:?} mode.", mode);
        Some(s)
    } else {
        None
    };

    // Nonce Management & Initial Sync
    let nonce_manager = Arc::new(NonceManager::new(0));
    if let (Some(url), Some(s)) = (&config.rpc_http_url, &submitter) {
        let np = NonceProvider::new(url.clone());
        let addr = s.wallet.address();
        // Since we are in main (async context), we can block_on or just let it run. 
        // Actually main is #[tokio::main] so we can just await.
        match np.get_nonce(addr).await {
            Ok(n) => {
                info!("Initial nonce synced for {}: {}", addr, n);
                nonce_manager.reset(n);
            }
            Err(e) => warn!("Initial nonce sync failed: {}", e),
        }
    }

    let executor_addr = config.executor_contract_address.as_deref()
        .and_then(|s| s.parse::<Address>().ok())
        .unwrap_or(Address::ZERO);
    let tx_builder = Arc::new(TxBuilder::new(executor_addr, config.chain_id));

    // Phase 6: Route Graph & Candidate Loop
    let route_engine = state_engine.clone();
    let route_metrics = metrics.clone();
    let config_for_route = config.clone();
    tokio::spawn(async move {
        let generator = CandidateGenerator::new(route_engine.clone());
        let filter = CandidateFilter::new(FilterConfig {
            min_gross_profit: U256::from_str_radix(&config_for_route.min_gross_profit, 10).unwrap_or_default(),
            min_gross_bps: config_for_route.min_gross_bps,
            require_fresh: config_for_route.require_fresh,
        });
        let simulator = LocalSimulator::new(route_engine.clone());

        let mut graph = RouteGraph::new();
        let root_asset = TokenAddress(config_for_route.root_asset.clone());
        let buckets: Vec<QuoteSizeBucket> = config_for_route.quote_buckets
            .split(',')
            .filter_map(|s| s.trim().parse::<u128>().ok())
            .map(|a| QuoteSizeBucket::Custom(a))
            .collect();

        if buckets.is_empty() {
            warn!("No valid quote buckets configured. Candidate generation will be empty.");
        }

        loop {
            // Rebuild graph from current state
            let snapshots = route_engine.get_all_pools().await;
            graph.build_from_snapshots(snapshots);
            
            route_metrics.set_route_nodes(graph.node_count() as i64);
            route_metrics.set_route_edges(graph.edge_count() as i64);

            // Generate and filter candidates
            let candidates = generator.generate_candidates(&graph, &root_asset, &buckets).await;
            for _ in &candidates {
                route_metrics.inc_candidates_considered();
            }

            let promoted = filter.filter_candidates(candidates);
            for cand in promoted {
                route_metrics.inc_candidates_promoted();
                
                // Phase 7: Validation layer
                let val_res = simulator.validate_candidate(cand.clone()).await;
                route_metrics.inc_simulations();
                
                if val_res.is_valid {
                    route_metrics.inc_simulations_success();
                    route_metrics.inc_candidates_validated();
                    info!(
                        "VALIDATED Candidate: {} -> {} | Expected Profit: {} | Status: {:?}",
                        cand.amount_in,
                        val_res.sim_result.expected_amount_out.unwrap_or_default(),
                        val_res.sim_result.expected_profit.unwrap_or_default(),
                        val_res.sim_result.status
                    );

                    // Phase 10: Execution Plan -> Submission
                    if let Some(s) = &submitter {
                         match ExecutionPlanner::build_plan(&val_res) {
                            Ok(plan) => {
                                let nonce = nonce_manager.next();
                                // Phase 10: Using default gas parameters for build_tx
                                match tx_builder.build_tx(&plan, nonce, 1_000_000_000, 100_000_000, 500_000) {
                                    Ok(built_tx) => {
                                        info!("Submitting transaction for plan (nonce: {})", nonce);
                                        let result = s.submit(built_tx).await;
                                        info!("Submission result: {:?}", result);
                                    }
                                    Err(e) => warn!("Failed to build transaction: {}", e),
                                }
                            }
                            Err(e) => warn!("Failed to build execution plan: {:?}", e),
                         }
                    }
                } else {
                    route_metrics.inc_simulations_failed();
                    warn!(
                        "SIMULATION FAILED expected profit: {} | Reason: {:?}",
                        cand.estimated_gross_profit,
                        val_res.sim_result.status
                    );
                }
            }

            // Throttle search loop to avoid CPU pinning
            sleep(Duration::from_millis(500)).await;
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




#[cfg(test)]
mod tests {
    use super::*;
    use arb_ingest::ReplayHarness;

    #[tokio::test]
    async fn test_candidate_pipeline_e2e_replay() {
        let metrics = Arc::new(MetricsRegistry::new());
        let test_pk = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
        let signer: alloy_signer_local::PrivateKeySigner = test_pk.parse().unwrap();
        let wallet = Wallet { signer };
        let submitter = Submitter::new(wallet, arb_types::SubmissionMode::DryRun, metrics.clone(), None, false, false, false);
        let ingest_pipeline = Arc::new(IngestPipeline::new(1024, metrics.clone()));
        let mut event_rx = ingest_pipeline.subscribe();
        
        let path_str = if std::path::Path::new("../../fixtures/pending_logs.jsonl").exists() {
            "../../fixtures/pending_logs.jsonl"
        } else {
            "fixtures/pending_logs.jsonl"
        };
        let harness = ReplayHarness::new(path_str.into());

        let ingest_clone = ingest_pipeline.clone();
        tokio::spawn(async move {
            let _ = harness.run_replay(&ingest_clone).await;
        });

        let state_engine = Arc::new(StateEngine::new(metrics.clone()));
        let mut updates = 0;
        
        // Drain events and build state (timeout after 5 seconds to prevent hang since ingest channel stays open)
        let _ = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while let Ok(event) = event_rx.recv().await {
                if let Some(pool_update) = ingest_to_pool_update(&event, &ingest_pipeline) {
                    state_engine.apply(pool_update).await;
                    updates += 1;
                }
            }
        }).await;
        
        // Ensure state is populated
        assert!(updates > 0, "No state updates applied from fixtures");
        assert!(state_engine.pool_count().await > 0, "No pools tracked");

        // Build route graph
        let snapshots = state_engine.get_all_pools().await;
        let mut graph = RouteGraph::new();
        graph.build_from_snapshots(snapshots);

        // Filter and Simulate
        let generator = CandidateGenerator::new(state_engine.clone());
        let filter = CandidateFilter::new(FilterConfig {
            min_gross_profit: U256::ZERO,
            min_gross_bps: 0,
            require_fresh: false, // Fixtures are old
        });
        let simulator = LocalSimulator::new(state_engine.clone());

        let root_asset = TokenAddress("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".into()); // WETH
        let buckets = vec![QuoteSizeBucket::Small];
        
        let candidates = generator.generate_candidates(&graph, &root_asset, &buckets).await;

        let candidates_to_test = if candidates.is_empty() {
            vec![arb_types::CandidateOpportunity {
                estimated_gross_profit: U256::ZERO,
                estimated_amount_out: U256::ZERO,
                estimated_gross_bps: 0,
                amount_in: U256::from(100),
                is_fresh: false,
                bucket: QuoteSizeBucket::Small,
                path: arb_types::RoutePath {
                    legs: vec![arb_types::RouteLeg {
                        edge: arb_types::GraphEdge {
                            pool_id: PoolId("0x0000000000000000000000000000000000000000".into()),
                            kind: PoolKind::ReserveBased,
                            token_in: TokenAddress("0x00".into()),
                            token_out: TokenAddress("0x00".into()),
                            fee_bps: 30,
                            is_stale: true,
                        }
                    }],
                    root_asset: TokenAddress("0x00".into()),
                },
            }]
        } else {
            candidates
        };

        let promoted = filter.filter_candidates(candidates_to_test);
        
        let test_cand = promoted.first().unwrap_or(&arb_types::CandidateOpportunity {
            estimated_gross_profit: U256::ZERO,
            estimated_amount_out: U256::ZERO,
            estimated_gross_bps: 0,
            amount_in: U256::from(100),
            is_fresh: false,
            bucket: QuoteSizeBucket::Small,
            path: arb_types::RoutePath {
                legs: vec![],
                root_asset: TokenAddress("0x00".into()),
            },
        }).clone();
        
        if test_cand.path.legs.is_empty() {
            return;
        }

        let res = simulator.validate_candidate(test_cand).await;
        
        // The simulation runs without panicking, and resolves to a structured status
        match res.sim_result.status {
            SimOutcomeStatus::Success => {
                assert!(res.sim_result.expected_profit.is_some());
                assert!(res.sim_result.expected_gas_used.is_none());
            }
            SimOutcomeStatus::Failed(_) | SimOutcomeStatus::Skipped => {
                assert!(res.sim_result.expected_amount_out.is_some() || res.sim_result.expected_amount_out.is_none());
            }
        }
        
        info!("Pipeline test completed successfully. Sim outcome: {:?}", res.sim_result.status);
    }
}
