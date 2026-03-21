use alloy_sol_types::{sol, SolCall};
use alloy_primitives::{Address, U256};
use arb_types::{ExecutionPlan as ArbExecutionPlan, BuiltTransaction};

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

    struct FlashLoanSpec {
        uint8 provider;
        address asset;
        uint256 amount;
    }

    struct RepaymentGuard {
        address asset;
        uint256 amount;
    }

    struct ProfitGuard {
        uint256 minProfitWei;
    }

    struct ExecutionPlanSol {
        address targetToken;
        ExecutionPath path;
        ExpectedOutcome outcome;
        SlippageGuard guard;
        bool hasFlashloan;
    }

    struct AtomicExecutionPlanSol {
        FlashLoanSpec flashloan;
        ExecutionPath path;
        uint256 minAmountOut;
        RepaymentGuard repayment;
        ProfitGuard profitGuard;
        bool hasFlashloan;
        bool hasRepayment;
    }

    function executePlan(ExecutionPlanSol calldata plan) external;
    function executeAtomicPlan(AtomicExecutionPlanSol calldata plan) external;
}

pub struct TxBuilder {
    pub executor_address: Address,
    pub chain_id: u64,
    pub force_legacy: bool,
}

impl TxBuilder {
    pub fn new(executor_address: Address, chain_id: u64) -> Self {
        Self {
            executor_address,
            chain_id,
            force_legacy: false,
        }
    }

    pub fn with_force_legacy(mut self, force: bool) -> Self {
        self.force_legacy = force;
        self
    }

    /// Converts an ExecutionPlan into a BuiltTransaction.
    pub fn build_tx(
        &self,
        plan: &ArbExecutionPlan,
        nonce: u64,
        max_fee: u128,
        max_priority_fee: u128,
        gas_limit: u64,
    ) -> Result<BuiltTransaction, String> {
        // Map arb_types to Solidity structs
        let mut sol_legs = Vec::with_capacity(plan.path.legs.len());
        for l in &plan.path.legs {
            sol_legs.push(ExecutionLeg {
                poolId: l.pool_id.0.parse().map_err(|e| format!("Invalid pool address '{}': {}", l.pool_id.0, e))?,
                tokenIn: l.token_in.0.parse().map_err(|e| format!("Invalid tokenIn address '{}': {}", l.token_in.0, e))?,
                tokenOut: l.token_out.0.parse().map_err(|e| format!("Invalid tokenOut address '{}': {}", l.token_out.0, e))?,
                zeroForOne: l.zero_for_one,
            });
        }

        let sol_plan = ExecutionPlanSol {
            targetToken: plan.target_token.0.parse().map_err(|e| format!("Invalid targetToken address '{}': {}", plan.target_token.0, e))?,
            path: ExecutionPath { legs: sol_legs },
            outcome: ExpectedOutcome {
                amountIn: plan.outcome.amount_in,
                expectedAmountOut: plan.outcome.expected_amount_out,
                expectedProfit: plan.outcome.expected_profit,
            },
            guard: SlippageGuard {
                minOut: MinOutConstraint {
                    minAmountOut: plan.guard.min_out.min_amount_out,
                },
                minProfitWei: plan.guard.min_profit_wei,
            },
            hasFlashloan: plan.flash_loan.is_some(),
        };

        // Encode calldata
        let call = executePlanCall { plan: sol_plan };
        let calldata = call.abi_encode();

        Ok(BuiltTransaction {
            to: format!("{:#x}", self.executor_address),
            data: calldata,
            value: U256::ZERO,
            nonce,
            gas_limit,
            max_fee_per_gas: if self.force_legacy { 0 } else { max_fee },
            max_priority_fee_per_gas: if self.force_legacy { 0 } else { max_priority_fee },
            gas_price: if self.force_legacy { Some(max_fee) } else { None },
            chain_id: self.chain_id,
        })
    }

    /// Converts an AtomicExecutionPlan into a BuiltTransaction.
    pub fn build_atomic_tx(
        &self,
        plan: &arb_types::AtomicExecutionPlan,
        nonce: u64,
        max_fee: u128,
        max_priority_fee: u128,
        gas_limit: u64,
    ) -> Result<BuiltTransaction, String> {
        let mut sol_legs = Vec::with_capacity(plan.legs.len());
        for l in &plan.legs {
            sol_legs.push(ExecutionLeg {
                poolId: l.pool_id.0.parse().map_err(|e| format!("Invalid pool address '{}': {}", l.pool_id.0, e))?,
                tokenIn: l.token_in.0.parse().map_err(|e| format!("Invalid tokenIn address '{}': {}", l.token_in.0, e))?,
                tokenOut: l.token_out.0.parse().map_err(|e| format!("Invalid tokenOut address '{}': {}", l.token_out.0, e))?,
                zeroForOne: l.zero_for_one,
            });
        }

        let is_flashloan = plan.flash_loan.is_some();
        let flash_loan_sol = plan.flash_loan.as_ref().map(|f| FlashLoanSpec {
            provider: f.provider.clone() as u8,
            asset: f.asset.parse().unwrap_or_default(),
            amount: f.amount,
        }).unwrap_or(FlashLoanSpec {
            provider: 0,
            asset: Address::ZERO,
            amount: U256::ZERO,
        });

        let is_repayment = plan.repayment.is_some();
        let repayment_sol = plan.repayment.as_ref().map(|r| RepaymentGuard {
            asset: r.asset.parse().unwrap_or_default(),
            amount: r.amount,
        }).unwrap_or(RepaymentGuard {
            asset: Address::ZERO,
            amount: U256::ZERO,
        });

        let sol_plan = AtomicExecutionPlanSol {
            flashloan: flash_loan_sol,
            path: ExecutionPath { legs: sol_legs },
            minAmountOut: plan.min_amount_out,
            repayment: repayment_sol,
            profitGuard: ProfitGuard {
                minProfitWei: plan.profit_guard.min_profit_wei,
            },
            hasFlashloan: is_flashloan,
            hasRepayment: is_repayment,
        };

        let call = executeAtomicPlanCall { plan: sol_plan };
        let calldata = call.abi_encode();

        Ok(BuiltTransaction {
            to: format!("{:#x}", self.executor_address),
            data: calldata,
            value: U256::ZERO,
            nonce,
            gas_limit,
            max_fee_per_gas: if self.force_legacy { 0 } else { max_fee },
            max_priority_fee_per_gas: if self.force_legacy { 0 } else { max_priority_fee },
            gas_price: if self.force_legacy { Some(max_fee) } else { None },
            chain_id: self.chain_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arb_types::{ExecutionPath as ArbPath, ExpectedOutcome as ArbOutcome, SlippageGuard as ArbGuard, MinOutConstraint as ArbMinOut, PoolId, TokenAddress, ExecutionLeg as ArbLeg};

    #[test]
    fn test_tx_builder_basic() {
        let builder = TxBuilder::new(Address::ZERO, 8453);
        
        let plan = ArbExecutionPlan {
            target_token: TokenAddress("0x4200000000000000000000000000000000000006".to_string()),
            path: ArbPath {
                legs: vec![ArbLeg {
                    pool_id: PoolId("0x1111111111111111111111111111111111111111".to_string()),
                    token_in: TokenAddress("0x4200000000000000000000000000000000000006".to_string()),
                    token_out: TokenAddress("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()),
                    zero_for_one: true,
                }],
            },
            outcome: ArbOutcome {
                amount_in: U256::from(100),
                expected_amount_out: U256::from(99),
                expected_profit: U256::from(1),
            },
            guard: ArbGuard {
                min_out: ArbMinOut {
                    min_amount_out: U256::from(98),
                },
                min_profit_wei: U256::ZERO,
            },
            flash_loan: None,
        };

        let tx = builder.build_tx(&plan, 1, 1000, 10, 200000).unwrap();
        
        assert_eq!(tx.nonce, 1);
        assert_eq!(tx.gas_limit, 200000);
        assert_eq!(tx.max_fee_per_gas, 1000);
        assert_eq!(tx.max_priority_fee_per_gas, 10);
        assert!(!tx.data.is_empty());
        // Selector for executePlan(ExecutionPlanSol)
        assert_eq!(tx.data[0..4], executePlanCall::SELECTOR);
    }

    #[test]
    fn test_tx_builder_invalid_address() {
        let builder = TxBuilder::new(Address::ZERO, 8453);
        
        let mut plan = ArbExecutionPlan {
            target_token: TokenAddress("invalid".to_string()),
            path: ArbPath { legs: vec![] },
            outcome: ArbOutcome {
                amount_in: U256::ZERO,
                expected_amount_out: U256::ZERO,
                expected_profit: U256::ZERO,
            },
            guard: ArbGuard {
                min_out: ArbMinOut { min_amount_out: U256::ZERO },
                min_profit_wei: U256::ZERO,
            },
            flash_loan: None,
        };

        let result = builder.build_tx(&plan, 1, 1000, 10, 200000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid targetToken address"));

        // Test invalid pool address
        plan.target_token = TokenAddress(format!("{:#x}", Address::ZERO));
        plan.path.legs.push(ArbLeg {
            pool_id: PoolId("invalid_pool".to_string()),
            token_in: TokenAddress(format!("{:#x}", Address::ZERO)),
            token_out: TokenAddress(format!("{:#x}", Address::ZERO)),
            zero_for_one: true,
        });

        let result = builder.build_tx(&plan, 1, 1000, 10, 200000);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid pool address"));
    }
}
