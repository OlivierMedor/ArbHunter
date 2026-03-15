// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";
import {ArbExecutor, ExecutionPlan, ExecutionPath, ExpectedOutcome, SlippageGuard, MinOutConstraint, ExecutionLeg} from "../src/ArbExecutor.sol";

// Dummy token for testing balances
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
        ExecutionLeg memory leg = ExecutionLeg(address(pool), address(0x2), address(0x3), true);
        ExecutionLeg[] memory legs = new ExecutionLeg[](1);
        legs[0] = leg;

        return ExecutionPlan({
            targetToken: address(token),
            path: ExecutionPath(legs),
            outcome: ExpectedOutcome(1000, 1050, 50),
            guard: SlippageGuard(MinOutConstraint(1020)),
            hasFlashloan: false
        });
    }

    function test_RevertIf_Unauthorized() public {
        ExecutionPlan memory plan = buildMockPlan();
        vm.prank(nonOwner);
        vm.expectRevert(ArbExecutor.Unauthorized.selector);
        executor.executePlan(plan);
    }

    function test_RevertIf_SlippageExceeded() public {
        ExecutionPlan memory plan = buildMockPlan();
        // Do not give executor any token balance increase
        pool.setMintAmount(0);
        token.setBalance(address(executor), 0);
        
        vm.prank(owner);
        vm.expectRevert(abi.encodeWithSelector(ArbExecutor.SlippageExceeded.selector, 1020, 0));
        executor.executePlan(plan);
    }

    function test_Success_WithProfit() public {
        ExecutionPlan memory plan = buildMockPlan();
        
        // Starts with 0 balance, FakePool will mint 1050 to the executor!
        // Profit > 1020 constraint => SUCCESS!
        vm.prank(owner);
        executor.executePlan(plan);
    }
}
