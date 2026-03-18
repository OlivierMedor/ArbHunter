// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {ArbExecutor} from "../src/ArbExecutor.sol";

contract DeployScript is Script {
    function run() public {
        uint256 deployerPrivateKey = vm.envUint("TEST_PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        ArbExecutor executor = new ArbExecutor();
        console.log("ArbExecutor deployed to:", address(executor));

        vm.stopBroadcast();
    }
}
