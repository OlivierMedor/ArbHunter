// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "forge-std/interfaces/IERC20.sol";

struct MinOutConstraint {
    uint256 minAmountOut;
}

struct SlippageGuard {
    MinOutConstraint minOut;
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

struct ExecutionPlan {
    address targetToken;
    ExecutionPath path;
    ExpectedOutcome outcome;
    SlippageGuard guard;
    bool hasFlashloan;
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

struct AtomicExecutionPlan {
    FlashLoanSpec flashloan;
    ExecutionPath path;
    uint256 minAmountOut;
    RepaymentGuard repayment;
    ProfitGuard profitGuard;
    bool hasFlashloan;
    bool hasRepayment;
}

contract ArbExecutor {
    address public owner;

    error Unauthorized();
    error SlippageExceeded(uint256 expected, uint256 actual);
    error InsufficientProfit(uint256 expected, uint256 actual);
    error InsufficientRepayment(uint256 required, uint256 actual);

    modifier onlyOwner() {
        if (msg.sender != owner) revert Unauthorized();
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    /// Deterministic execution entrypoint (Standard)
    function executePlan(ExecutionPlan calldata plan) external onlyOwner {
        IERC20 targetToken = IERC20(plan.targetToken);
        uint256 balanceBefore = targetToken.balanceOf(address(this));

        for (uint256 i = 0; i < plan.path.legs.length; i++) {
             ExecutionLeg memory leg = plan.path.legs[i];
             leg.poolId.call(abi.encodeWithSignature("swap()"));
        }

        uint256 balanceAfter = targetToken.balanceOf(address(this));
        uint256 actualAmountOut = balanceAfter > balanceBefore ? balanceAfter - balanceBefore : 0;

        if (actualAmountOut < plan.guard.minOut.minAmountOut) {
            revert SlippageExceeded(plan.guard.minOut.minAmountOut, actualAmountOut);
        }

        if (balanceAfter <= balanceBefore) {
            revert InsufficientProfit(0, 0); // Placeholder
        }
    }

    /// Atomic execution entrypoint (Flash-Loan Capable)
    function executeAtomicPlan(AtomicExecutionPlan calldata plan) external onlyOwner {
        if (plan.hasFlashloan) {
            // Initiate flash loan
            // In Phase 11 Mock: we just call the logic as if we have funds
            _executeAtomicInternal(plan);
        } else {
            _executeAtomicInternal(plan);
        }
    }

    function _executeAtomicInternal(AtomicExecutionPlan calldata plan) internal {
        // Assume funds are in or will be repaid
        
        // Tracking balance of the first token in the route (usually the loan asset)
        address rootAsset = plan.path.legs[0].tokenIn;
        uint256 balanceBefore = IERC20(rootAsset).balanceOf(address(this));

        // Execute route legs
        for (uint256 i = 0; i < plan.path.legs.length; i++) {
            ExecutionLeg calldata leg = plan.path.legs[i];
            // Actual swap logic would go here
            leg.poolId.call(abi.encodeWithSignature("swap()"));
        }

        uint256 balanceAfter = IERC20(rootAsset).balanceOf(address(this));
        uint256 actualAmountOut = balanceAfter > balanceBefore ? balanceAfter - balanceBefore : 0;

        // Repayment enforcement
        if (plan.hasRepayment) {
            uint256 amountToRepay = plan.repayment.amount;
            if (balanceAfter < amountToRepay) {
                revert InsufficientRepayment(amountToRepay, balanceAfter);
            }
            // IERC20(plan.repayment.asset).transfer(msg.sender, amountToRepay);
            balanceAfter -= amountToRepay;
        }

        // Slippage Guard
        if (actualAmountOut < plan.minAmountOut) {
            revert SlippageExceeded(plan.minAmountOut, actualAmountOut);
        }

        // Profit Guard
        uint256 netProfit = balanceAfter > balanceBefore ? balanceAfter - balanceBefore : 0;
        if (netProfit < plan.profitGuard.minProfitWei) {
            revert InsufficientProfit(plan.profitGuard.minProfitWei, netProfit);
        }
    }
}
