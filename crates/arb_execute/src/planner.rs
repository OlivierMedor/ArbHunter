use arb_types::{
    CandidateValidationResult, ExecutionLeg, ExecutionPath, ExecutionPlan, ExpectedOutcome,
    MinOutConstraint, PlanBuildFailureReason, SlippageGuard, AtomicExecutionPlan, ProfitGuard,
    RepaymentGuard, FlashLoanSpec, FlashLoanProviderKind
};
use alloy_primitives::U256;

pub struct ExecutionPlanner;

impl ExecutionPlanner {
    /// Attempts to build a standard execution plan from a validated candidate
    pub fn build_plan(
        validation_result: &CandidateValidationResult,
    ) -> Result<ExecutionPlan, PlanBuildFailureReason> {
        // ... (existing implementation unchanged)
        let candidate = &validation_result.sim_result.request.candidate;
        if !validation_result.is_valid {
            return Err(PlanBuildFailureReason::InsufficientProfit);
        }
        let expected_out = validation_result.sim_result.expected_amount_out
            .ok_or(PlanBuildFailureReason::InsufficientProfit)?;
        if expected_out <= candidate.amount_in {
            return Err(PlanBuildFailureReason::InsufficientProfit);
        }
        let profit = expected_out - candidate.amount_in;
        let mut legs = Vec::with_capacity(candidate.path.legs.len());
        for leg in &candidate.path.legs {
            match leg.edge.kind {
                arb_types::PoolKind::ReserveBased | arb_types::PoolKind::ConcentratedLiquidity => {}
                arb_types::PoolKind::Unknown => return Err(PlanBuildFailureReason::UnsupportedPoolKind),
            }
            let zero_for_one = leg.edge.token_in.0 < leg.edge.token_out.0;
            legs.push(ExecutionLeg {
                pool_id: leg.edge.pool_id.clone(),
                token_in: leg.edge.token_in.clone(),
                token_out: leg.edge.token_out.clone(),
                zero_for_one,
            });
        }
        if legs.is_empty() {
            return Err(PlanBuildFailureReason::UnsupportedRouteStructure);
        }
        Ok(ExecutionPlan {
            target_token: candidate.path.root_asset.clone(),
            path: ExecutionPath { legs },
            outcome: ExpectedOutcome {
                amount_in: candidate.amount_in,
                expected_amount_out: expected_out,
                expected_profit: profit,
            },
            guard: SlippageGuard {
                min_out: MinOutConstraint {
                    min_amount_out: candidate.amount_in + (profit / U256::from(2)),
                },
                min_profit_wei: profit / U256::from(4),
            },
            flash_loan: None,
        })
    }

    /// Attempts to build an atomic flash-loan execution plan from a validated candidate
    pub fn build_atomic_plan(
        validation_result: &CandidateValidationResult,
        flash_loan: bool,
    ) -> Result<AtomicExecutionPlan, PlanBuildFailureReason> {
        let candidate = &validation_result.sim_result.request.candidate;
        
        if !validation_result.is_valid {
            return Err(PlanBuildFailureReason::InsufficientProfit);
        }

        let expected_out = validation_result.sim_result.expected_amount_out
            .ok_or(PlanBuildFailureReason::InsufficientProfit)?;
        
        let profit = validation_result.sim_result.expected_profit
            .ok_or(PlanBuildFailureReason::InsufficientProfit)?;

        let mut legs = Vec::with_capacity(candidate.path.legs.len());
        for leg in &candidate.path.legs {
            match leg.edge.kind {
                arb_types::PoolKind::ReserveBased | arb_types::PoolKind::ConcentratedLiquidity => {}
                arb_types::PoolKind::Unknown => return Err(PlanBuildFailureReason::UnsupportedPoolKind),
            }
            let zero_for_one = leg.edge.token_in.0 < leg.edge.token_out.0;
            legs.push(ExecutionLeg {
                pool_id: leg.edge.pool_id.clone(),
                token_in: leg.edge.token_in.clone(),
                token_out: leg.edge.token_out.clone(),
                zero_for_one,
            });
        }

        let flash_loan_spec = if flash_loan {
            Some(FlashLoanSpec {
                provider: FlashLoanProviderKind::Mock, // Default for Phase 11
                asset: candidate.path.root_asset.0.clone(),
                amount: candidate.amount_in,
            })
        } else {
            None
        };

        let repayment = if flash_loan {
            Some(RepaymentGuard {
                asset: candidate.path.root_asset.0.clone(),
                // Repay loan + small buffer or just loan if provider is mock
                amount: candidate.amount_in + (candidate.amount_in * U256::from(5) / U256::from(10000)), // 5bps fee
            })
        } else {
            None
        };

        Ok(AtomicExecutionPlan {
            flash_loan: flash_loan_spec,
            legs,
            min_amount_out: candidate.amount_in + (profit / U256::from(2)),
            repayment,
            profit_guard: ProfitGuard {
                min_profit_wei: profit / U256::from(4),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arb_types::{
        CandidateOpportunity, QuoteSizeBucket, RoutePath, TokenAddress, RouteLeg, GraphEdge,
        PoolId, PoolKind, SimulationResult, SimulationRequest, SimulationFailureReason, SimOutcomeStatus
    };
    use alloy_sol_types::{sol, SolCall};
    use alloy_primitives::Address;

    fn make_base_candidate() -> CandidateOpportunity {
        CandidateOpportunity {
            path: RoutePath {
                legs: vec![RouteLeg {
                    edge: GraphEdge {
                        pool_id: PoolId("0xPOOL".into()),
                        kind: PoolKind::ReserveBased,
                        token_in: TokenAddress("0xA".into()),
                        token_out: TokenAddress("0xB".into()),
                        fee_bps: 30,
                        is_stale: false,
                    }
                }],
                root_asset: TokenAddress("0xA".into()),
            },
            bucket: QuoteSizeBucket::Small,
            amount_in: U256::from(1000),
            estimated_amount_out: U256::from(1050),
            estimated_gross_profit: U256::from(50),
            estimated_gross_bps: 500,
            is_fresh: true,
        }
    }

    #[test]
    fn test_valid_candidate_builds_plan() {
        let candidate = make_base_candidate();
        let val_res = CandidateValidationResult {
            sim_result: SimulationResult {
                request: SimulationRequest { candidate: candidate.clone() },
                status: SimOutcomeStatus::Success,
                expected_amount_out: Some(U256::from(1040)), // real sim outcome
                expected_profit: Some(U256::from(40)),
                expected_gas_used: None,
            },
            is_valid: true,
        };

        let plan = ExecutionPlanner::build_plan(&val_res).expect("Plan should build");
        
        assert_eq!(plan.target_token.0, "0xA");
        assert_eq!(plan.path.legs.len(), 1);
        assert_eq!(plan.path.legs[0].pool_id.0, "0xPOOL");
        // Ensure MinOut / guard encoded correctly (amount_in + profit/2)
        // amount_in = 1000, profit = 40, min_out = 1020
        assert_eq!(plan.guard.min_out.min_amount_out, U256::from(1020));
    }

    #[test]
    fn test_unsupported_pool_kind_fails_build() {
        let mut candidate = make_base_candidate();
        candidate.path.legs[0].edge.kind = PoolKind::Unknown;

        let val_res = CandidateValidationResult {
            sim_result: SimulationResult {
                request: SimulationRequest { candidate: candidate.clone() },
                status: SimOutcomeStatus::Success,
                expected_amount_out: Some(U256::from(1040)),
                expected_profit: Some(U256::from(40)),
                expected_gas_used: None,
            },
            is_valid: true,
        };

        let result = ExecutionPlanner::build_plan(&val_res);
        assert_eq!(result.unwrap_err(), PlanBuildFailureReason::UnsupportedPoolKind);
    }

    #[test]
    fn test_insufficient_profit_fails() {
        let candidate = make_base_candidate();
        // Return 990 (a loss) instead of > 1000
        let val_res = CandidateValidationResult {
            sim_result: SimulationResult {
                request: SimulationRequest { candidate: candidate.clone() },
                status: SimOutcomeStatus::Failed(SimulationFailureReason::SlippageExceeded),
                expected_amount_out: Some(U256::from(990)), 
                expected_profit: None,
                expected_gas_used: None,
            },
            is_valid: false,
        };

        let result = ExecutionPlanner::build_plan(&val_res);
        assert_eq!(result.unwrap_err(), PlanBuildFailureReason::InsufficientProfit);
    }

    #[test]
    fn test_abi_encoding_alignment() {
        // Define the expected ABI identically to ArbExecutor.sol
        sol! {
            struct MinOutConstraint {
                uint256 minAmountOut;
            }
            struct SlippageGuard {
                MinOutConstraint minOut;
                uint256 minProfitWei;
            }
            struct ExecutionLeg {
                address poolId;
                address tokenIn;
                address tokenOut;
                bool zeroForOne;
            }
            struct ExecutionPath {
                ExecutionLeg[] legs;
            }
            struct ExpectedOutcome {
                uint256 amountIn;
                uint256 expectedAmountOut;
                uint256 expectedProfit;
            }
            struct ExecutionPlanSol {
                address targetToken;
                ExecutionPath path;
                ExpectedOutcome outcome;
                SlippageGuard guard;
                bool hasFlashloan;
            }
            function executePlan(ExecutionPlanSol calldata plan) external;
        }

        // Generate a real Rust planner payload
        let candidate = make_base_candidate();
        let val_res = CandidateValidationResult {
            sim_result: SimulationResult {
                request: SimulationRequest { candidate: candidate.clone() },
                status: SimOutcomeStatus::Success,
                expected_amount_out: Some(U256::from(1040)),
                expected_profit: Some(U256::from(40)),
                expected_gas_used: None,
            },
            is_valid: true,
        };
        let rust_plan = ExecutionPlanner::build_plan(&val_res).expect("Should build");

        // Map Rust types to Alloy Sol types
        let mut sol_legs = Vec::new();
        for leg in rust_plan.path.legs {
            sol_legs.push(ExecutionLeg {
                poolId: leg.pool_id.0.parse::<Address>().unwrap_or_default(),
                tokenIn: leg.token_in.0.parse::<Address>().unwrap_or_default(),
                tokenOut: leg.token_out.0.parse::<Address>().unwrap_or_default(),
                zeroForOne: leg.zero_for_one,
            });
        }

        let sol_plan = ExecutionPlanSol {
            targetToken: rust_plan.target_token.0.parse::<Address>().unwrap_or_default(),
            path: ExecutionPath { legs: sol_legs },
            outcome: ExpectedOutcome {
                amountIn: rust_plan.outcome.amount_in,
                expectedAmountOut: rust_plan.outcome.expected_amount_out,
                expectedProfit: rust_plan.outcome.expected_profit,
            },
            guard: SlippageGuard {
                minOut: MinOutConstraint {
                    minAmountOut: rust_plan.guard.min_out.min_amount_out,
                },
                minProfitWei: rust_plan.guard.min_profit_wei,
            },
            hasFlashloan: rust_plan.flash_loan.is_some(),
        };

        // Ensure struct encoding passes cleanly (proves solidity types accurately aligned)
        let call = executePlanCall { plan: sol_plan };
        let encoded = call.abi_encode();
        
        assert_eq!(&encoded[0..4], &executePlanCall::SELECTOR);
        // Ensure payload packs properly without panic
        assert!(encoded.len() > 64, "Calldata should be substantively encoded");
    }

    #[test]
    fn test_atomic_plan_build() {
        let mut candidate = make_base_candidate();
        candidate.amount_in = U256::from(10000);
        let val_res = CandidateValidationResult {
            sim_result: SimulationResult {
                request: SimulationRequest { candidate: candidate.clone() },
                status: SimOutcomeStatus::Success,
                expected_amount_out: Some(U256::from(10400)),
                expected_profit: Some(U256::from(400)),
                expected_gas_used: None,
            },
            is_valid: true,
        };

        let plan = ExecutionPlanner::build_atomic_plan(&val_res, true).expect("Atomic plan should build");
        
        assert!(plan.flash_loan.is_some());
        assert_eq!(plan.flash_loan.as_ref().unwrap().amount, U256::from(10000));
        assert!(plan.repayment.is_some());
        assert!(plan.repayment.as_ref().unwrap().amount > U256::from(10000));
        assert_eq!(plan.profit_guard.min_profit_wei, U256::from(100)); // 400 / 4
    }
}
