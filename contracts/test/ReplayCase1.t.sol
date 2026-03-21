// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test, console2} from "forge-std/Test.sol";
import {ArbExecutor, ExecutionPlan, ExecutionPath, ExecutionLeg, ExpectedOutcome, SlippageGuard, MinOutConstraint} from "../src/ArbExecutor.sol";
import {IERC20} from "forge-std/interfaces/IERC20.sol";

contract ReplayCase1 is Test {
    ArbExecutor executor;
    address USDC;
    address WETH;
    address POOL1;
    address POOL2;

    function setUp() public {
        vm.createSelectFork("https://base-mainnet.g.alchemy.com/v2/TVjVM6xK62VPdkP6OXfmk", 43652157);
        executor = new ArbExecutor();
        USDC = vm.parseAddress("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
        WETH = vm.parseAddress("0x4200000000000000000000000000000000000006");
        POOL1 = vm.parseAddress("0xd0b53d9277642d899df5c87a3966a349a798f224");
        POOL2 = vm.parseAddress("0x6c561b446416e1a00e8e93e221854d6ea4171372");
    }

    function test_Replay_Case1() public {
        // Deal USDC to executor
        deal(USDC, address(executor), 100_000_000);

        ExecutionLeg[] memory legs = new ExecutionLeg[](2);
        legs[0] = ExecutionLeg({
            poolId: POOL1,
            poolKind: 1, // V3
            tokenIn: USDC,
            tokenOut: WETH,
            zeroForOne: false,
            amountOut: 0 // not used for V3 in executor but needs a value
        });
        legs[1] = ExecutionLeg({
            poolId: POOL2,
            poolKind: 1, // V3
            tokenIn: WETH,
            tokenOut: USDC,
            zeroForOne: true,
            amountOut: 0
        });

        ExecutionPlan memory plan = ExecutionPlan({
            targetToken: USDC,
            path: ExecutionPath({legs: legs}),
            outcome: ExpectedOutcome({
                amountIn: 100_000_000,
                expectedAmountOut: 100_000_000, // dummy
                expectedProfit: 0
            }),
            guard: SlippageGuard({
                minOut: MinOutConstraint({minAmountOut: 0}),
                minProfitWei: 0
            }),
            hasFlashloan: false
        });

        vm.prank(executor.owner());
        executor.executePlan(plan);
        
        uint256 finalBal = IERC20(USDC).balanceOf(address(executor));
        console2.log("Final USDC Balance:", finalBal);
        assert(finalBal > 0);
    }
}
