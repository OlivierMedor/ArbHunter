// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ArbExecutor, ExecutionPlan, ExecutionPath, ExpectedOutcome, SlippageGuard, MinOutConstraint, ExecutionLeg, AtomicExecutionPlan, FlashLoanSpec, RepaymentGuard, ProfitGuard} from "../src/ArbExecutor.sol";

// ... (existing MockToken and FakePool unchanged)
contract MockToken {
    function totalSupply() external view returns (uint256) { return 0; }
    function balanceOf(address account) external view returns (uint256) { return balances[account]; }
    function transfer(address to, uint256 amount) external returns (bool) { return true; }
    function allowance(address owner, address spender) external view returns (uint256) { return 0; }
    function approve(address spender, uint256 amount) external returns (bool) { return true; }
    function transferFrom(address from, address to, uint256 amount) external returns (bool) { return true; }
    
    // Test helper to mock balances
    mapping(address => uint256) public balances;
    function setBalance(address account, uint256 amount) external {
        balances[account] = amount;
    }
}

contract FakePool {
    MockToken public token;
    uint256 public mintAmount = 1050;
    constructor(MockToken _token) { token = _token; }
    function setMintAmount(uint256 ans) external { mintAmount = ans; }
    function swap() external {
        token.setBalance(msg.sender, mintAmount);
    }
}

contract ArbExecutorTest is Test {
    ArbExecutor executor;
    MockToken token;
    FakePool pool;
    address owner = address(0x123);
    address nonOwner = address(0x456);

    function setUp() public {
        vm.prank(owner);
        executor = new ArbExecutor();
        token = new MockToken();
        pool = new FakePool(token);
    }

    function buildMockPlan() internal view returns (ExecutionPlan memory) {
        ExecutionLeg memory leg = ExecutionLeg(address(pool), address(token), address(0x3), true);
        ExecutionLeg[] memory legs = new ExecutionLeg[](1);
        legs[0] = leg;

        return ExecutionPlan({
            targetToken: address(token),
            path: ExecutionPath(legs),
            outcome: ExpectedOutcome(1000, 1050, 50),
            guard: SlippageGuard(MinOutConstraint(1020), 0),
            hasFlashloan: false
        });
    }

    function buildAtomicMockPlan(bool flash) internal view returns (AtomicExecutionPlan memory) {
        ExecutionLeg memory leg = ExecutionLeg(address(pool), address(token), address(0x3), true);
        ExecutionLeg[] memory legs = new ExecutionLeg[](1);
        legs[0] = leg;

        FlashLoanSpec memory fl = FlashLoanSpec(0, address(token), 1000);
        RepaymentGuard memory rep = RepaymentGuard(address(token), 1001);
        ProfitGuard memory profit = ProfitGuard(10);

        return AtomicExecutionPlan({
            flashloan: fl,
            path: ExecutionPath(legs),
            minAmountOut: 1020,
            repayment: rep,
            profitGuard: profit,
            hasFlashloan: flash,
            hasRepayment: flash
        });
    }

    function test_RevertIf_Unauthorized() public {
        ExecutionPlan memory plan = buildMockPlan();
        vm.prank(nonOwner);
        vm.expectRevert(ArbExecutor.Unauthorized.selector);
        executor.executePlan(plan);
    }

    function test_Success_WithProfit() public {
        ExecutionPlan memory plan = buildMockPlan();
        vm.prank(owner);
        executor.executePlan(plan);
    }

    // Phase 11 Atomic Tests
    function test_Atomic_Success() public {
        AtomicExecutionPlan memory plan = buildAtomicMockPlan(false);
        vm.prank(owner);
        executor.executeAtomicPlan(plan);
    }

    function test_Atomic_Slippage_Revert() public {
        AtomicExecutionPlan memory plan = buildAtomicMockPlan(false);
        pool.setMintAmount(1000); // 1000 < 1020 (minAmountOut)
        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(ArbExecutor.SlippageExceeded.selector, 1020, 1000));
        executor.executeAtomicPlan(plan);
    }

    function test_Atomic_NoProfit_Revert() public {
        AtomicExecutionPlan memory plan = buildAtomicMockPlan(false);
        // Start with zero balance
        token.setBalance(address(executor), 0);
        
        plan.minAmountOut = 1000;
        plan.profitGuard = ProfitGuard(1100);
        pool.setMintAmount(1050); // 1050 > 1000 (slippage ok), netProfit 1050 < 1100 (NoProfit!)
        
        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(ArbExecutor.ProfitTooLow.selector, 1100, 1050));
        executor.executeAtomicPlan(plan);
    }

    function test_Atomic_InsufficientRepayment_Revert() public {
        AtomicExecutionPlan memory plan = buildAtomicMockPlan(true);
        // buildAtomicMockPlan(true) sets hasFlashloan=true, hasRepayment=true
        // amount_in = 1000, repayment = 1001.
        plan.minAmountOut = 1000; // Pass slippage check
        pool.setMintAmount(1000); // 1000 < 1001 repayment required
        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(ArbExecutor.InsufficientRepayment.selector, 1001, 1000));
        executor.executeAtomicPlan(plan);
    }
}
