// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {Script, console} from "forge-std/Script.sol";
import {DelegationVerifier} from "../src/DelegationVerifier.sol";
import {DelegationVault} from "../src/DelegationVault.sol";

/// @notice Deploy the DelegationVerifier and DelegationVault contracts.
///
/// Run with:
///   anvil
///   forge script script/DeployDelegationVault.s.sol --rpc-url http://localhost:8545 --broadcast
contract DeployDelegationVault is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        DelegationVerifier verifier = new DelegationVerifier();
        DelegationVault vault = new DelegationVault(verifier);

        vm.stopBroadcast();

        console.log("DelegationVerifier deployed at:", address(verifier));
        console.log("DelegationVault deployed at:", address(vault));
    }
}
