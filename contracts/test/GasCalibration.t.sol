// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import {Test, stdJson} from "forge-std/Test.sol";
import {ArbExecutor, ExecutionPlan, ExecutionPath, ExpectedOutcome, SlippageGuard, MinOutConstraint, ExecutionLeg, AtomicExecutionPlan, FlashLoanSpec, RepaymentGuard, ProfitGuard} from "../src/ArbExecutor.sol";

contract GasCalibrationTest is Test {
  using stdJson for string;
  ArbExecutor executor;
  address owner = address(0x123);

  function setUp() public {
    vm.prank(owner);
    executor = new ArbExecutor();
  }

  function skip_test_calibrate() public {
    string memory json = vm.readFile("stratified_sample_plan.json");
    bytes memory data = json.parseRaw(".calldata");
    vm.prank(owner);
    (bool success, bytes memory returnData) = address(executor).call(data);
    require(success, "Execution failed");
  }
}
