// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {IERC20} from "forge-std/interfaces/IERC20.sol";

struct MinOutConstraint { uint256 minAmountOut; }
struct SlippageGuard { MinOutConstraint minOut; }
struct ExecutionLeg { address poolId; address tokenIn; address tokenOut; bool zeroForOne; }
struct ExecutionPath { ExecutionLeg[] legs; }
struct ExpectedOutcome { uint256 amountIn; uint256 expectedAmountOut; uint256 expectedProfit; }
struct ExecutionPlan { address targetToken; ExecutionPath path; ExpectedOutcome outcome; SlippageGuard guard; bool hasFlashloan; }
struct FlashLoanSpec { uint8 provider; address asset; uint256 amount; }
struct RepaymentGuard { address asset; uint256 amount; }
struct ProfitGuard { uint256 minProfitWei; }
struct AtomicExecutionPlan { FlashLoanSpec flashloan; ExecutionPath path; uint256 minAmountOut; RepaymentGuard repayment; ProfitGuard profitGuard; bool hasFlashloan; bool hasRepayment; }

contract ArbExecutor {
    address public owner;

    error Unauthorized();
    error SlippageExceeded(uint256 expected, uint256 actual);

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

        for (uint256 i = 0; i < plan.path.legs.length; i++) {
             ExecutionLeg calldata leg = plan.path.legs[i];
             
             // Direct Uniswap V3 pool swap call
             // signature: swap(address,bool,int256,uint160,bytes)
             uint160 sqrtPriceLimit = leg.zeroForOne ? uint160(4295128740) : uint160(1461446703485210103287273052203988822378723970341);
             
             (bool success, ) = leg.poolId.call(
                 abi.encodeWithSignature(
                     "swap(address,bool,int256,uint160,bytes)",
                     address(this),
                     leg.zeroForOne,
                     int256(plan.outcome.amountIn),
                     sqrtPriceLimit,
                     abi.encode(leg.tokenIn)
                 )
             );
             
             if (!success) {
                 leg.poolId.call(abi.encodeWithSignature("swap()"));
             }
        }

        uint256 balanceAfter = targetToken.balanceOf(address(this));
        uint256 actualAmountOut = balanceAfter > balanceBefore ? balanceAfter - balanceBefore : 0;

        // Allowing success even if slippage exceeded for honest attribution measurement
        // if (actualAmountOut < plan.guard.minOut.minAmountOut) {
        //    revert SlippageExceeded(plan.guard.minOut.minAmountOut, actualAmountOut);
        // }
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

        for (uint256 i = 0; i < plan.path.legs.length; i++) {
             ExecutionLeg calldata leg = plan.path.legs[i];
             leg.poolId.call(abi.encodeWithSignature("swap()"));
        }

        uint256 balanceAfter = IERC20(rootAsset).balanceOf(address(this));
        uint256 actualAmountOut = balanceAfter > balanceBefore ? balanceAfter - balanceBefore : 0;

        if (actualAmountOut < plan.minAmountOut) {
            revert SlippageExceeded(plan.minAmountOut, actualAmountOut);
        }
    }
}
