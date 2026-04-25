// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Script, console } from "forge-std/Script.sol";
import { BlindAuction }    from "../src/BlindAuction.sol";

contract DeployBlindAuctionScript is Script {
    function run() external {
        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        vm.startBroadcast(deployerKey);

        BlindAuction auction = new BlindAuction();

        console.log("Contract Address:", address(auction));

        vm.stopBroadcast();
    }
}
