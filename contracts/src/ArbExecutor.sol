// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "forge-std/interfaces/IERC20.sol";

struct MinOutConstraint { uint256 minAmountOut; }
struct SlippageGuard { MinOutConstraint minOut; uint256 minProfitWei; }
struct ExecutionLeg { address poolId; uint8 poolKind; address tokenIn; address tokenOut; bool zeroForOne; uint256 amountOut; }
struct ExecutionPath { ExecutionLeg[] legs; }
struct ExpectedOutcome { uint256 amountIn; uint256 expectedAmountOut; uint256 expectedProfit; }
struct ExecutionPlan { address targetToken; ExecutionPath path; ExpectedOutcome outcome; SlippageGuard guard; bool hasFlashloan; }
struct FlashLoanSpec { uint8 provider; address asset; uint256 amount; }
struct RepaymentGuard { address asset; uint256 amount; }
struct ProfitGuard { uint256 minProfitWei; }
struct AtomicExecutionPlan { FlashLoanSpec flashloan; ExecutionPath path; uint256 minAmountOut; RepaymentGuard repayment; ProfitGuard profitGuard; bool hasFlashloan; bool hasRepayment; }

interface IUniswapV2Pair {
    function swap(uint256 amount0Out, uint256 amount1Out, address to, bytes calldata data) external;
}

contract ArbExecutor {
    address public owner;

    error Unauthorized();
    error SlippageExceeded(uint256 expected, uint256 actual);
    error ProfitTooLow(uint256 expected, uint256 actual);
    error InsufficientRepayment(uint256 expected, uint256 actual);

    modifier onlyOwner() {
        if (msg.sender != owner) revert Unauthorized();
        _;
    }

    constructor() {
        owner = msg.sender;
    }

    function executePlan(ExecutionPlan calldata plan) external onlyOwner {
        IERC20 targetToken = IERC20(plan.targetToken);
        uint256 balanceBefore = targetToken.balanceOf(address(this));

        uint256 currentAmountIn = plan.outcome.amountIn;

        for (uint256 i = 0; i < plan.path.legs.length; i++) {
             ExecutionLeg calldata leg = plan.path.legs[i];
             uint256 balBeforeLeg = IERC20(leg.tokenOut).balanceOf(address(this));
             if (leg.poolKind == 1) { // ConcentratedLiquidity (V3)
                 uint160 sqrtPriceLimit = leg.zeroForOne ? uint160(4295128740) : uint160(1461446703485210103287273052203988822378723970341);
                 
                 (bool success, ) = leg.poolId.call(
                     abi.encodeWithSignature(
                         "swap(address,bool,int256,uint160,bytes)",
                         address(this),
                         leg.zeroForOne,
                         int256(currentAmountIn),
                         sqrtPriceLimit,
                         abi.encode(leg.tokenIn)
                     )
                 );
                 if (!success) revert("V3 Swap Failed");
             } else if (leg.poolKind == 0) { // ReserveBased (V2)
                IERC20(leg.tokenIn).transfer(leg.poolId, currentAmountIn);
                // swap(amount0Out, amount1Out, to, data)
                // If zeroForOne is true: we give token0, receive token1 (amount1Out > 0)
                // If zeroForOne is false: we give token1, receive token0 (amount0Out > 0)
                uint256 out0 = leg.zeroForOne ? 0 : (leg.amountOut * 995) / 1000;
                uint256 out1 = leg.zeroForOne ? (leg.amountOut * 995) / 1000 : 0;
                IUniswapV2Pair(leg.poolId).swap(out0, out1, address(this), "");
            } else {
                 revert("Unsupported PoolKind");
             }
             currentAmountIn = IERC20(leg.tokenOut).balanceOf(address(this)) - balBeforeLeg;
        }

        uint256 balanceAfter = targetToken.balanceOf(address(this));
        uint256 actualAmountOut = balanceAfter > balanceBefore ? balanceAfter - balanceBefore : 0;

        if (actualAmountOut < plan.guard.minOut.minAmountOut) {
            revert SlippageExceeded(plan.guard.minOut.minAmountOut, actualAmountOut);
        }

        uint256 actualProfit = actualAmountOut > plan.outcome.amountIn ? actualAmountOut - plan.outcome.amountIn : 0;
        if (actualProfit < plan.guard.minProfitWei) {
            revert ProfitTooLow(plan.guard.minProfitWei, actualProfit);
        }
    }

    function uniswapV3SwapCallback(
        int256 amount0Delta,
        int256 amount1Delta,
        bytes calldata data
    ) external {
        address tokenIn = abi.decode(data, (address));
        uint256 amountToPay = amount0Delta > 0 ? uint256(amount0Delta) : uint256(amount1Delta);
        IERC20(tokenIn).transfer(msg.sender, amountToPay);
    }

    function executeAtomicPlan(AtomicExecutionPlan calldata plan) external onlyOwner {
        _executeAtomicInternal(plan);
    }

    function _executeAtomicInternal(AtomicExecutionPlan calldata plan) internal {
        address rootAsset = plan.path.legs[0].tokenIn;
        uint256 balanceBefore = IERC20(rootAsset).balanceOf(address(this));

        // Use the same logic as executePlan for legs
        uint256 currentAmountIn = plan.flashloan.amount; // Initial amount from flashloan

        for (uint256 i = 0; i < plan.path.legs.length; i++) {
             ExecutionLeg calldata leg = plan.path.legs[i];
             uint256 balBeforeLeg = IERC20(leg.tokenOut).balanceOf(address(this));
             if (leg.poolKind == 1) { // V3
                 uint160 sqrtPriceLimit = leg.zeroForOne ? uint160(4295128740) : uint160(1461446703485210103287273052203988822378723970341);
                 (bool success, ) = leg.poolId.call(
                     abi.encodeWithSignature(
                         "swap(address,bool,int256,uint160,bytes)",
                         address(this),
                         leg.zeroForOne,
                         int256(currentAmountIn),
                         sqrtPriceLimit,
                         abi.encode(leg.tokenIn)
                     )
                 );
                 if (!success) revert("V3 Atomic Swap Failed");
             } else if (leg.poolKind == 0) { // V2
                 IERC20(leg.tokenIn).transfer(leg.poolId, currentAmountIn);
                 uint256 out0 = leg.zeroForOne ? 0 : (leg.amountOut * 995) / 1000;
                 uint256 out1 = leg.zeroForOne ? (leg.amountOut * 995) / 1000 : 0;
                 IUniswapV2Pair(leg.poolId).swap(out0, out1, address(this), "");
             }
             currentAmountIn = IERC20(leg.tokenOut).balanceOf(address(this)) - balBeforeLeg;
        }

        uint256 balanceAfter = IERC20(rootAsset).balanceOf(address(this));
        uint256 actualAmountOut = balanceAfter > balanceBefore ? balanceAfter - balanceBefore : 0;

        if (actualAmountOut < plan.minAmountOut) {
            revert SlippageExceeded(plan.minAmountOut, actualAmountOut);
        }

        if (plan.hasRepayment) {
            if (actualAmountOut < plan.repayment.amount) {
                revert InsufficientRepayment(plan.repayment.amount, actualAmountOut);
            }
        }

        uint256 baseAmount = plan.hasRepayment ? plan.repayment.amount : 0; 
        uint256 actualProfit = actualAmountOut > baseAmount ? actualAmountOut - baseAmount : 0;
        if (actualProfit < plan.profitGuard.minProfitWei) {
            revert ProfitTooLow(plan.profitGuard.minProfitWei, actualProfit);
        }
    }
}
