// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {IVerifier} from "../src/SolvencyRegistry.sol";
import {SolvencyRegistry} from "../src/SolvencyRegistry.sol";

contract DeploySolvencyRegistry is Script {
    function run() external returns (SolvencyRegistry registry) {
        // Address of the deployed UltraHonk verifier for the solvency circuit.
        address verifierAddr = vm.envOr("SOLVENCY_VERIFIER_ADDRESS", address(0));
        require(verifierAddr != address(0), "SOLVENCY_VERIFIER_ADDRESS not set");

        vm.startBroadcast();
        registry = new SolvencyRegistry(IVerifier(verifierAddr));
        vm.stopBroadcast();

        console2.log("SolvencyRegistry deployed at:", address(registry));
        console2.log("Verifier:", verifierAddr);
    }
}
