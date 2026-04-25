// SPDX-License-Identifier: MIT
pragma solidity ^0.8.27;

import { Script, console }        from "forge-std/Script.sol";
import { ConfidentialVault }      from "../src/ConfidentialVault.sol";
import { ConfidentialCollateral } from "../src/tokens/ConfidentialCollateral.sol";
import { ConfidentialDebt }       from "../src/tokens/ConfidentialDebt.sol";
import { PriceOracle }            from "../src/PriceOracle.sol";

contract DeployConfidentialVaultScript is Script {
    function run() external {
        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        vm.startBroadcast(deployerKey);

        // Deploy supporting tokens first
        ConfidentialCollateral collateral = new ConfidentialCollateral();
        ConfidentialDebt       debt       = new ConfidentialDebt();
        PriceOracle            oracle     = new PriceOracle(3000_000000); // $3000 ETH initial

        // Deploy vault
        ConfidentialVault vault = new ConfidentialVault(
            address(collateral),
            address(debt)
        );

        // Wire tokens to vault
        collateral.setVault(address(vault));
        debt.setVault(address(vault));

        console.log("Contract Address:", address(vault));
        console.log("Collateral token:", address(collateral));
        console.log("Debt token:      ", address(debt));
        console.log("Price oracle:    ", address(oracle));

        vm.stopBroadcast();
    }
}
