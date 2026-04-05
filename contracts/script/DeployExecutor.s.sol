// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/ArbExecutor.sol";

contract DeployExecutor is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("SIGNER_PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        ArbExecutor executor = new ArbExecutor();

        console.log("ArbExecutor deployed at:", address(executor));

        vm.stopBroadcast();
    }
}
