// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/ArbExecutor.sol";

contract DebugReplayTest is Test {
    ArbExecutor executor;
    
    function setUp() public {}

    function test_Debug_Case1_V3() public {
        address pool = 0xd0b53D9277642d899DF5C87A3966A349A798F224;
        address usdc = 0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913;
        address weth = 0x4200000000000000000000000000000000000006;

        vm.createSelectFork("https://mainnet.base.org", 43640036);
        executor = new ArbExecutor();
        
        deal(usdc, address(executor), 100_000_000);
        
        ExecutionLeg[] memory legs = new ExecutionLeg[](1);
        legs[0] = ExecutionLeg({
            poolId: pool,
            poolKind: 1, // V3
            tokenIn: usdc,
            tokenOut: weth,
            zeroForOne: false, // 1 -> 0 (USDC -> WETH)
            amountOut: 45287221866718482252464 // from sim
        });
        
        ExecutionPlan memory plan = ExecutionPlan({
            targetToken: weth,
            path: ExecutionPath({ legs: legs }),
            outcome: ExpectedOutcome({
                amountIn: 100_000_000,
                expectedAmountOut: 45287221866718482252464,
                expectedProfit: 1 // dummy
            }),
            guard: SlippageGuard({
                minOut: MinOutConstraint({ minAmountOut: 0 }),
                minProfitWei: 0
            }),
            hasFlashloan: false
        });
        
        executor.executePlan(plan);
    }
}
