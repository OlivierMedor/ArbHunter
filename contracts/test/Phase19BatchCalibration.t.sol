// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;
import {Test, stdJson} from "forge-std/Test.sol";
import {ArbExecutor, ExecutionPlan, ExecutionPath, ExpectedOutcome, SlippageGuard, MinOutConstraint, ExecutionLeg, AtomicExecutionPlan, FlashLoanSpec, RepaymentGuard, ProfitGuard} from "../src/ArbExecutor.sol";

contract Phase19BatchCalibration is Test {
  using stdJson for string;
  ArbExecutor executor;
  address owner = address(0x123);

  function setUp() public {
    vm.prank(owner);
    executor = new ArbExecutor();
  }

  function test_BatchCalibrate() public {
    string memory root = vm.projectRoot();
    string memory json = vm.readFile(string.concat(root, "/../calibration_fixture.json"));
    for (uint i = 0; i < 40; i++) {
      string memory key = string.concat(".samples[", vm.toString(i), "]");
      uint256 blk = json.readUint(string.concat(key, ".block"));
      string memory id = json.readString(string.concat(key, ".id"));
      string memory buck = json.readString(string.concat(key, ".bucket"));
      emit log_named_uint(string.concat("SAMPLE_BLK:", id), blk);
      emit log_named_string("BUCKET", buck);
    }
  }
}
