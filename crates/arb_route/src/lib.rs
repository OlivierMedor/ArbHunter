use std::collections::HashMap;
use petgraph::graph::{NodeIndex, DiGraph};
use arb_types::{PoolStateSnapshot, TokenAddress, GraphEdge, RouteLeg, RoutePath, PoolId};

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

    /// Build or rebuild the graph from a collection of pool snapshots.
    pub fn build_from_snapshots(&mut self, snapshots: Vec<PoolStateSnapshot>) {
        self.graph.clear();
        self.token_to_node.clear();

        for snap in snapshots {
            let t0 = match snap.token0 {
                Some(ref t) => t.clone(),
                None => continue,
            };
            let t1 = match snap.token1 {
                Some(ref t) => t.clone(),
                None => continue,
            };

            let n0 = self.get_or_create_node(t0.clone());
            let n1 = self.get_or_create_node(t1.clone());

            // Add directed edges for both directions
            // In -> Out (Swap t0 for t1)
            self.graph.add_edge(n0, n1, GraphEdge {
                pool_id: snap.pool_id.clone(),
                kind: snap.kind,
                token_in: t0.clone(),
                token_out: t1.clone(),
                fee_bps: snap.fee_bps,
                is_stale: snap.freshness.is_stale,
            });

            // In -> Out (Swap t1 for t0)
            self.graph.add_edge(n1, n0, GraphEdge {
                pool_id: snap.pool_id,
                kind: snap.kind,
                token_in: t1,
                token_out: t0,
                fee_bps: snap.fee_bps,
                is_stale: snap.freshness.is_stale,
            });
        }
    }

    fn get_or_create_node(&mut self, token: TokenAddress) -> NodeIndex {
        if let Some(&idx) = self.token_to_node.get(&token) {
            idx
        } else {
            let idx = self.graph.add_node(token.clone());
            self.token_to_node.insert(token, idx);
            idx
        }
    }

    /// Find all 2-hop cyclic routes starting and ending at the root asset.
    pub fn find_2hop_cycles(&self, root: &TokenAddress) -> Vec<RoutePath> {
        let mut routes = Vec::new();
        let root_idx = match self.token_to_node.get(root) {
            Some(&idx) => idx,
            None => return routes,
        };

        // A -> B -> A
        for edge_ab in self.graph.edges_connecting(root_idx, root_idx) {
            // Self-loop? Unusual but possible. 
            // Actually edges_connecting finds edges between two specific nodes.
        }

        // Broaden: for each neighbor B of A
        for edge_ab in self.graph.edges(root_idx) {
            let b_idx = edge_ab.target();
            if b_idx == root_idx { continue; }

            // Find return edges B -> A
            for edge_ba in self.graph.edges(b_idx) {
                if edge_ba.target() == root_idx {
                    // Valid 2-hop cycle
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

    /// Find selected 3-hop cyclic routes starting and ending at the root asset.
    pub fn find_3hop_cycles(&self, root: &TokenAddress) -> Vec<RoutePath> {
        let mut routes = Vec::new();
        let root_idx = match self.token_to_node.get(root) {
            Some(&idx) => idx,
            None => return routes,
        };

        // A -> B -> C -> A
        for edge_ab in self.graph.edges(root_idx) {
            let b_idx = edge_ab.target();
            if b_idx == root_idx { continue; }

            for edge_bc in self.graph.edges(b_idx) {
                let c_idx = edge_bc.target();
                if c_idx == root_idx || c_idx == b_idx { continue; }

                for edge_ca in self.graph.edges(c_idx) {
                    if edge_ca.target() == root_idx {
                        // Valid 3-hop cycle
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

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}


use arb_types::{CandidateOpportunity, QuoteSizeBucket, PoolKind};
use alloy_primitives::U256;
use arb_state::Quoter;
use arb_state::StateEngine;

pub struct CandidateGenerator {
    engine: Arc<StateEngine>,
}

impl CandidateGenerator {
    pub fn new(engine: Arc<StateEngine>) -> Self {
        Self { engine }
    }

    /// Generate candidates from cycles found in the graph.
    pub async fn generate_candidates(
        &self,
        graph: &RouteGraph,
        root_asset: &TokenAddress,
        buckets: &[QuoteSizeBucket],
    ) -> Vec<CandidateOpportunity> {
        let mut candidates = Vec::new();

        let cycles_2hop = graph.find_2hop_cycles(root_asset);
        let cycles_3hop = graph.find_3hop_cycles(root_asset);
        let all_paths = cycles_2hop.into_iter().chain(cycles_3hop.into_iter());

        for path in all_paths {
            for &bucket in buckets {
                let amount_in = self.bucket_to_amount(bucket);
                if let Some(candidate) = self.evaluate_path(path.clone(), bucket, amount_in).await {
                    candidates.push(candidate);
                }
            }
        }

        candidates
    }

    async fn evaluate_path(
        &self,
        path: RoutePath,
        bucket: QuoteSizeBucket,
        amount_in: U256,
    ) -> Option<CandidateOpportunity> {
        let mut current_amount = amount_in;
        let mut is_fresh = true;

        for leg in &path.legs {
            let edge = &leg.edge;
            if edge.is_stale { is_fresh = false; }

            let next_amount = match edge.kind {
                PoolKind::ReserveBased => {
                    self.engine.quote_v2(&edge.pool_id, current_amount).await?
                }
                PoolKind::ConcentratedLiquidity => {
                    // We need zero_for_one. 
                    // In UniV3, token0 < token1. 
                    // If token_in == token0, then zero_for_one = true.
                    let zero_for_one = edge.token_in.0 < edge.token_out.0;
                    self.engine.quote_v3(&edge.pool_id, current_amount, zero_for_one).await?
                }
                PoolKind::Unknown => return None,
            };

            if next_amount.is_zero() { return None; }
            current_amount = next_amount;
        }

        let estimated_gross_profit = if current_amount > amount_in {
            current_amount - amount_in
        } else {
            U256::ZERO
        };

        let estimated_gross_bps = if !amount_in.is_zero() {
            let bps = (estimated_gross_profit * U256::from(10000)) / amount_in;
            bps.to::<u32>()
        } else {
            0
        };

        Some(CandidateOpportunity {
            path,
            bucket,
            amount_in,
            estimated_amount_out: current_amount,
            estimated_gross_profit,
            estimated_gross_bps,
            is_fresh,
        })
    }

    fn bucket_to_amount(&self, bucket: QuoteSizeBucket) -> U256 {
        match bucket {
            QuoteSizeBucket::Small => U256::from(100_000_000_000_000_000u64), // 0.1 ETH/Tokens
            QuoteSizeBucket::Medium => U256::from(1_000_000_000_000_000_000u64), // 1 ETH
            QuoteSizeBucket::Large => U256::from(10_000_000_000_000_000_000u64), // 10 ETH
            QuoteSizeBucket::Custom(a) => U256::from(a),
        }
    }
}

use std::sync::Arc;
use petgraph::visit::EdgeRef;
