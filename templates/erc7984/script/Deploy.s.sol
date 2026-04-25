// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Script, console } from "forge-std/Script.sol";
import { ConfidentialToken } from "../src/ConfidentialToken.sol";

contract DeployConfidentialTokenScript is Script {
    function run() external {
        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        vm.startBroadcast(deployerKey);

        ConfidentialToken token = new ConfidentialToken(
            "Confidential USD",
            "cUSD",
            6,
            1_000_000_000000 // 1M initial supply
        );

        console.log("Contract Address:", address(token));
        console.log("Name:   ", token.name());
        console.log("Symbol: ", token.symbol());

        vm.stopBroadcast();
    }
}
