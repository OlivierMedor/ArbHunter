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

contract ArbExecutor {
    address public owner;

    error Unauthorized();
    error SlippageExceeded(uint256 expected, uint256 actual);
    error InsufficientProfit(uint256 balanceBefore, uint256 balAfter);

    modifier onlyOwner() {
        if (msg.sender != owner) revert Unauthorized();
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    /// Deterministic execution entrypoint
    function executePlan(ExecutionPlan calldata plan) external onlyOwner {
        IERC20 targetToken = IERC20(plan.targetToken);
        uint256 balanceBefore = targetToken.balanceOf(address(this));

        // For Phase 8 we mock execution.
        // We iterate out the legs dynamically.
        for (uint256 i = 0; i < plan.path.legs.length; i++) {
             ExecutionLeg memory leg = plan.path.legs[i];
             // Phase 8 dummy execution trigger
             leg.poolId.call(abi.encodeWithSignature("swap()"));
        }

        uint256 balanceAfter = targetToken.balanceOf(address(this));

        uint256 actualAmountOut;
        if (balanceAfter > balanceBefore) {
            actualAmountOut = balanceAfter - balanceBefore;
        } else {
            actualAmountOut = 0;
        }

        // Slippage Guard
        if (actualAmountOut < plan.guard.minOut.minAmountOut) {
            revert SlippageExceeded(plan.guard.minOut.minAmountOut, actualAmountOut);
        }

        // Expected Profit constraint
        if (balanceAfter <= balanceBefore) {
            revert InsufficientProfit(balanceBefore, balanceAfter);
        }
    }
}
