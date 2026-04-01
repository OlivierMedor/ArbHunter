use std::collections::HashMap;
use std::sync::Arc;
use petgraph::graph::{NodeIndex, DiGraph};
use petgraph::visit::EdgeRef;
use arb_types::{PoolStateSnapshot, TokenAddress, GraphEdge, RouteLeg, RoutePath, PoolId, CandidateOpportunity, QuoteSizeBucket, PoolKind, RouteFamily};
use alloy_primitives::U256;
use arb_state::{Quoter, StateEngine};

pub struct RouteGraph {
    graph: DiGraph<TokenAddress, GraphEdge>,
    token_to_node: HashMap<TokenAddress, NodeIndex>,
}

impl RouteGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            token_to_node: HashMap::new(),
        }
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn build_from_snapshots(&mut self, snapshots: Vec<PoolStateSnapshot>) {
        for pool in snapshots {
            let t0 = match pool.token0.as_ref() { Some(t) => t.clone(), None => continue };
            let t1 = match pool.token1.as_ref() { Some(t) => t.clone(), None => continue };

            let n0 = *self.token_to_node.entry(t0.clone()).or_insert_with(|| self.graph.add_node(t0.clone()));
            let n1 = *self.token_to_node.entry(t1.clone()).or_insert_with(|| self.graph.add_node(t1.clone()));

            self.graph.add_edge(n0, n1, GraphEdge {
                pool_id: pool.pool_id.clone(),
                kind: pool.kind,
                token_in: t0.clone(),
                token_out: t1.clone(),
                fee_bps: pool.fee_bps,
                is_stale: false,
            });

            self.graph.add_edge(n1, n0, GraphEdge {
                pool_id: pool.pool_id.clone(),
                kind: pool.kind,
                token_in: t1,
                token_out: t0,
                fee_bps: pool.fee_bps,
                is_stale: false,
            });
        }
    }

    pub fn find_2hop_cycles(&self, root: &TokenAddress) -> Vec<RoutePath> {
        let mut routes = Vec::new();
        let root_node = match self.token_to_node.get(root) { Some(n) => *n, None => return routes };

        for edge_ab in self.graph.edges(root_node) {
            let node_b = edge_ab.target();
            for edge_ba in self.graph.edges(node_b) {
                if edge_ba.target() == root_node {
                    routes.push(RoutePath {
                        root_asset: root.clone(),
                        legs: vec![
                            RouteLeg { edge: edge_ab.weight().clone() },
                            RouteLeg { edge: edge_ba.weight().clone() },
                        ],
                    });
                }
            }
        }
        routes
    }

    pub fn find_3hop_cycles(&self, root: &TokenAddress) -> Vec<RoutePath> {
        let mut routes = Vec::new();
        let root_node = match self.token_to_node.get(root) { Some(n) => *n, None => return routes };

        for edge_ab in self.graph.edges(root_node) {
            let node_b = edge_ab.target();
            if node_b == root_node { continue; }
            for edge_bc in self.graph.edges(node_b) {
                let node_c = edge_bc.target();
                if node_c == root_node || node_c == node_b { continue; }
                for edge_ca in self.graph.edges(node_c) {
                    if edge_ca.target() == root_node {
                        routes.push(RoutePath {
                            root_asset: root.clone(),
                            legs: vec![
                                RouteLeg { edge: edge_ab.weight().clone() },
                                RouteLeg { edge: edge_bc.weight().clone() },
                                RouteLeg { edge: edge_ca.weight().clone() },
                            ],
                        });
                    }
                }
            }
        }
        routes
    }
}

pub struct CandidateGenerator {
    _engine: Arc<StateEngine>,
}

impl CandidateGenerator {
    pub fn new(engine: Arc<StateEngine>) -> Self {
        Self { _engine: engine }
    }

    pub fn generate_candidates(
        &self,
        graph: &RouteGraph,
        root_asset: &TokenAddress,
        buckets: &[QuoteSizeBucket],
        pool_map: &HashMap<PoolId, arb_types::PoolStateSnapshot>,
    ) -> Vec<CandidateOpportunity> {
        use rayon::prelude::*;
        let all_paths: Vec<_> = graph.find_2hop_cycles(root_asset).into_iter()
            .chain(graph.find_3hop_cycles(root_asset).into_iter()).collect();

        all_paths.par_iter().flat_map(|path| {
            buckets.par_iter().filter_map(|&bucket| {
                let amount_in = self.bucket_to_amount(bucket);
                self.evaluate_path(path.clone(), bucket, amount_in, pool_map)
            }).collect::<Vec<_>>()
        }).collect()
    }

    pub fn evaluate_path(
        &self,
        path: RoutePath,
        bucket: QuoteSizeBucket,
        amount_in: U256,
        pool_map: &HashMap<PoolId, arb_types::PoolStateSnapshot>,
    ) -> Option<CandidateOpportunity> {
        let mut current_amount = amount_in;
        let mut is_fresh = true;

        for leg in &path.legs {
            let edge = &leg.edge;
            if edge.is_stale { is_fresh = false; }
            let pool = pool_map.get(&edge.pool_id)?;
            let zero_for_one = edge.token_in.0 < edge.token_out.0;

            let next_amount = match edge.kind {
                PoolKind::ReserveBased => {
                    let reserves = pool.reserves.as_ref()?;
                    Some(Quoter::quote_v2_exact_in(reserves, current_amount, zero_for_one, pool.fee_bps))
                }
                PoolKind::ConcentratedLiquidity => {
                    let cl_state = if let Some(full) = pool.cl_full_state.as_ref() { full.clone() } 
                    else if let Some(snap) = pool.cl_snapshot.as_ref() {
                        arb_types::CLFullState {
                            sqrt_price_x96: snap.sqrt_price_x96,
                            liquidity: snap.liquidity,
                            tick: snap.tick,
                            ticks: std::collections::HashMap::new(),
                        }
                    } else { return None; };
                    Some(Quoter::quote_v3_exact_in(&cl_state, current_amount, zero_for_one, pool.fee_bps))
                }
                PoolKind::Unknown => return None,
            }?;
            if next_amount.is_zero() { return None; }
            current_amount = next_amount;
        }

        let estimated_gross_profit = current_amount.saturating_sub(amount_in);
        let estimated_gross_bps = if !amount_in.is_zero() {
            let bps = (estimated_gross_profit * U256::from(10000)) / amount_in;
            bps.saturating_to::<u32>()
        } else { 0 };

        let leg_count = path.legs.len();
        let route_family = RouteFamily::classify_by_leg_count(leg_count);

        Some(CandidateOpportunity {
            path, bucket, amount_in,
            estimated_amount_out: current_amount,
            estimated_gross_profit,
            estimated_gross_bps,
            is_fresh,
            route_family,
        })
    }

    pub fn bucket_to_amount(&self, bucket: QuoteSizeBucket) -> U256 {
        match bucket {
            QuoteSizeBucket::Small => U256::from(100_000_000_000_000_000u64), 
            QuoteSizeBucket::Medium => U256::from(1_000_000_000_000_000_000u64), 
            QuoteSizeBucket::Large => U256::from(10_000_000_000_000_000_000u64), 
            QuoteSizeBucket::Custom(a) => U256::from(a),
        }
    }
}
