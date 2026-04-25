// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Script, console }   from "forge-std/Script.sol";
import { ConfidentialVoting } from "../src/ConfidentialVoting.sol";

contract DeployConfidentialVotingScript is Script {
    function run() external {
        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        vm.startBroadcast(deployerKey);

        ConfidentialVoting voting = new ConfidentialVoting();

        console.log("Contract Address:", address(voting));

        vm.stopBroadcast();
    }
}
