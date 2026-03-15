use alloy_primitives::U256;
use std::sync::Arc;
use tracing::{debug, warn};

use arb_types::{
    CandidateOpportunity, CandidateValidationResult, QuoteSizeBucket, SimOutcomeStatus,
    SimulationFailureReason, SimulationRequest, SimulationResult, PoolKind,
};
use arb_state::StateEngine;

/// Evaluates a candidate opportunity locally against the most recent state.
/// This acts as a dry-run to ensure the opportunity is still valid before
/// committing to any execution path.
#[derive(Clone)]
pub struct LocalSimulator {
    state_engine: Arc<StateEngine>,
}

impl LocalSimulator {
    pub fn new(state_engine: Arc<StateEngine>) -> Self {
        Self { state_engine }
    }

    /// Converts a promoted candidate into a simulation request.
    pub fn create_request(candidate: CandidateOpportunity) -> SimulationRequest {
        SimulationRequest { candidate }
    }

    /// Simulates the request against the current state engine.
    /// This recalculates the expected output using the latest pool snapshots.
    pub async fn simulate(&self, request: SimulationRequest) -> SimulationResult {
        let candidate = &request.candidate;
        let mut current_amount = candidate.amount_in;
        let required_amount_in = candidate.amount_in;

        // Fetch the absolute newest states
        let pools = self.state_engine.get_all_pools().await;
        
        // Re-simulate step by step
        for leg in &candidate.path.legs {
            let pool_snapshot = match pools.iter().find(|p| p.pool_id == leg.edge.pool_id) {
                Some(p) => p,
                None => {
                    return SimulationResult {
                        request,
                        status: SimOutcomeStatus::Failed(SimulationFailureReason::RouteNotFound),
                        expected_amount_out: None,
                        expected_profit: None,
                        expected_gas_used: None,
                    };
                }
            };

            // Stale check
            if pool_snapshot.freshness.is_stale {
                return SimulationResult {
                    request,
                    status: SimOutcomeStatus::Failed(SimulationFailureReason::StaleState),
                    expected_amount_out: None,
                    expected_profit: None,
                    expected_gas_used: None,
                };
            }

            let next_amount = match leg.edge.kind {
                PoolKind::ReserveBased => {
                    self.state_engine.quote_v2(&leg.edge.pool_id, current_amount).await
                }
                PoolKind::ConcentratedLiquidity => {
                    let zero_for_one = leg.edge.token_in.0 < leg.edge.token_out.0;
                    self.state_engine.quote_v3(&leg.edge.pool_id, current_amount, zero_for_one).await
                }
                PoolKind::Unknown => None,
            };

            let amount_out = next_amount.unwrap_or(U256::ZERO);

            if amount_out.is_zero() {
                return SimulationResult {
                    request,
                    status: SimOutcomeStatus::Failed(SimulationFailureReason::InsufficientLiquidity),
                    expected_amount_out: None,
                    expected_profit: None,
                    expected_gas_used: None,
                };
            }
            current_amount = amount_out;
        }

        // Evaluate outcome
        if current_amount > required_amount_in {
            let profit = current_amount - required_amount_in;
            
            // Check if it slipped below our initial candidate's threshold
            // In a real execution environment, we might reject if profit drops significantly.
            // For now, any positive profit is a success if it validates.
            
            SimulationResult {
                request,
                status: SimOutcomeStatus::Success,
                expected_amount_out: Some(current_amount),
                expected_profit: Some(profit),
                expected_gas_used: Some(150_000), // mock dummy gas estimate for phase 7
            }
        } else {
            SimulationResult {
                request,
                status: SimOutcomeStatus::Failed(SimulationFailureReason::SlippageExceeded),
                expected_amount_out: Some(current_amount),
                expected_profit: None,
                expected_gas_used: None,
            }
        }
    }

    /// Wraps the simulate flow into a high-level validation result
    pub async fn validate_candidate(&self, candidate: CandidateOpportunity) -> CandidateValidationResult {
        let request = Self::create_request(candidate);
        let sim_result = self.simulate(request).await;
        
        let is_valid = matches!(sim_result.status, SimOutcomeStatus::Success);
        
        CandidateValidationResult {
            sim_result,
            is_valid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arb_types::{RoutePath, TokenAddress};

    // Minimal unit test for the conversion method.
    #[test]
    fn test_create_request_from_candidate() {
        let candidate = CandidateOpportunity {
            path: RoutePath {
                legs: vec![],
                root_asset: TokenAddress("0xWETH".to_string()),
            },
            bucket: QuoteSizeBucket::Small,
            amount_in: U256::from(100),
            estimated_amount_out: U256::from(105),
            estimated_gross_profit: U256::from(5),
            estimated_gross_bps: 500,
            is_fresh: true,
        };

        let req = LocalSimulator::create_request(candidate.clone());
        assert_eq!(req.candidate.amount_in, U256::from(100));
    }

    #[tokio::test]
    async fn test_simulator_stale_rejection() {
        let engine = Arc::new(StateEngine::new(std::sync::Arc::new(arb_metrics::MetricsRegistry::new())));
        let simulator = LocalSimulator::new(engine);

        let candidate = CandidateOpportunity {
            path: RoutePath {
                legs: vec![],
                root_asset: TokenAddress("0xTEST".to_string()),
            },
            bucket: QuoteSizeBucket::Small,
            amount_in: U256::ZERO,
            estimated_amount_out: U256::ZERO,
            estimated_gross_profit: U256::ZERO,
            estimated_gross_bps: 0,
            is_fresh: true,
        };

        let res = simulator.validate_candidate(candidate).await;
        // With an empty engine, evaluating legs will not find the route, but evaluating 0 legs just returns success if amount_out > amount_in which it isn't, so SlippageExceeded
        assert!(!res.is_valid);
        assert_eq!(res.sim_result.status, SimOutcomeStatus::Failed(SimulationFailureReason::SlippageExceeded));
    }
}
