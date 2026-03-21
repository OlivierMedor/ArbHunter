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
    function swap(address recipient, bool, int256, uint160, bytes calldata data) external {
        token.setBalance(recipient, mintAmount);
        (bool success, ) = msg.sender.call(
            abi.encodeWithSignature("uniswapV3SwapCallback(int256,int256,bytes)", int256(1000), int256(0), data)
        );
        require(success, "Callback failed");
    }
}

contract FakeV2Pool {
    MockToken public token;
    uint256 public mintAmount = 1050;
    constructor(MockToken _token) { token = _token; }
    function setMintAmount(uint256 ans) external { mintAmount = ans; }
    function swap(uint256 amount0Out, uint256 amount1Out, address to, bytes calldata) external {
        uint256 out = amount0Out > 0 ? amount0Out : amount1Out;
        token.setBalance(to, out);
    }
}

contract ArbExecutorTest is Test {
    ArbExecutor executor;
    MockToken token;
    FakePool poolV3;
    FakeV2Pool poolV2;
    address owner = address(0x123);
    address nonOwner = address(0x456);

    function setUp() public {
        vm.prank(owner);
        executor = new ArbExecutor();
        token = new MockToken();
        poolV3 = new FakePool(token);
        poolV2 = new FakeV2Pool(token);
    }

    function buildMockPlan() internal view returns (ExecutionPlan memory) {
        // V3 leg (poolKind = 1)
        ExecutionLeg memory leg = ExecutionLeg(address(poolV3), 1, address(token), address(0x3), true, 1050);
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

    function buildV2MockPlan() internal view returns (ExecutionPlan memory) {
        // V2 leg (poolKind = 0)
        ExecutionLeg memory leg = ExecutionLeg(address(poolV2), 0, address(token), address(0x3), true, 1060);
        ExecutionLeg[] memory legs = new ExecutionLeg[](1);
        legs[0] = leg;

        return ExecutionPlan({
            targetToken: address(token),
            path: ExecutionPath(legs),
            outcome: ExpectedOutcome(1000, 1060, 60),
            guard: SlippageGuard(MinOutConstraint(1020), 0),
            hasFlashloan: false
        });
    }

    function buildAtomicMockPlan(bool flash) internal view returns (AtomicExecutionPlan memory) {
        ExecutionLeg memory leg = ExecutionLeg(address(poolV3), 1, address(token), address(0x3), true, 1050);
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

    function test_V3_Success() public {
        ExecutionPlan memory plan = buildMockPlan();
        vm.prank(owner);
        executor.executePlan(plan);
    }

    function test_V2_Success() public {
        ExecutionPlan memory plan = buildV2MockPlan();
        poolV2.setMintAmount(1060);
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
        poolV3.setMintAmount(1000); // 1000 < 1020 (minAmountOut)
        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(ArbExecutor.SlippageExceeded.selector, 1020, 1000));
        executor.executeAtomicPlan(plan);
    }

    function test_Atomic_NoProfit_Revert() public {
        AtomicExecutionPlan memory plan = buildAtomicMockPlan(false);
        token.setBalance(address(executor), 0);
        
        plan.minAmountOut = 1000;
        plan.profitGuard = ProfitGuard(1100);
        poolV3.setMintAmount(1050); 
        
        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(ArbExecutor.ProfitTooLow.selector, 1100, 1050));
        executor.executeAtomicPlan(plan);
    }

    function test_Atomic_InsufficientRepayment_Revert() public {
        AtomicExecutionPlan memory plan = buildAtomicMockPlan(true);
        plan.minAmountOut = 1000;
        poolV3.setMintAmount(1000); // 1000 < 1001 repayment required
        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(ArbExecutor.InsufficientRepayment.selector, 1001, 1000));
        executor.executeAtomicPlan(plan);
    }
}
