// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {Script, console} from "forge-std/Script.sol";
import {TestToken} from "../src/TestToken.sol";
import {OtterBridge} from "../src/OtterBridge.sol";
import {BridgeToken} from "../src/BridgeToken.sol";
import {DelegationVerifier} from "../src/DelegationVerifier.sol";
import {DelegationVault} from "../src/DelegationVault.sol";
import {SolvencyVerifier} from "../src/SolvencyVerifier.sol";
import {SolvencyRegistry} from "../src/SolvencyRegistry.sol";
import {IVerifier} from "../src/SolvencyRegistry.sol";

/// @notice Deploy the whole local demo stack in ONE broadcast, in the exact
/// nonce order the docker-compose env defaults assume (anvil account #0,
/// fresh chain). The Honk verifier libraries deploy first, then the
/// contracts:
///   RelationsLib+ZKTranscriptLib(0-2) -> TestToken(3) -> OtterBridge(4) ->
///   BridgeToken(5) -> DelegationVerifier(6) -> DelegationVault(7) ->
///   SolvencyVerifier(8) -> SolvencyRegistry(9).
/// Deterministic addresses on a fresh chain (account #0):
///   TestToken          0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9
///   OtterBridge        0xDc64a140Aa3E981100a9becA4E685f962f0cF6C9
///   BridgeToken        0x5FC8d32690cc91D4c39d9d3abcBD16989F875707
///   DelegationVerifier 0x0165878A594ca255338adfa4d48449f69242Eb8F
///   DelegationVault    0xa513E6E4b8f2a923D98304ec87F64353C4D5C853
///   SolvencyVerifier   0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6
///   SolvencyRegistry   0x8A791620dd6260079BF849Dc5567aDC3F2FdC318
///
/// Run with (api container STOPPED so the orchestrator cannot consume a
/// nonce mid-deploy):
///   forge script script/DeployLocalStack.s.sol --rpc-url http://localhost:8545 \
///     --private-key <anvil-key-0> --broadcast
contract DeployLocalStack is Script {
    function run() public {
        vm.startBroadcast();

        // nonce 0..2 — bridge side
        TestToken token = new TestToken("Test Token", "tTST", 18);
        OtterBridge bridge = new OtterBridge(token, msg.sender);
        BridgeToken bridged = new BridgeToken("Otter Test Token", "otTST", address(bridge));

        // nonce 3..4 — delegation side
        DelegationVerifier delegationVerifier = new DelegationVerifier();
        DelegationVault vault = new DelegationVault(delegationVerifier);

        // nonce 5..6 — solvency side
        SolvencyVerifier solvencyVerifier = new SolvencyVerifier();
        SolvencyRegistry registry = new SolvencyRegistry(IVerifier(address(solvencyVerifier)));

        // wiring only (regular calls, no new contract addresses)
        bridge.setBridgeToken(bridged);

        vm.stopBroadcast();

        console.log("TestToken:         ", address(token));
        console.log("OtterBridge:       ", address(bridge));
        console.log("BridgeToken:       ", address(bridged));
        console.log("DelegationVerifier:", address(delegationVerifier));
        console.log("DelegationVault:   ", address(vault));
        console.log("SolvencyVerifier:  ", address(solvencyVerifier));
        console.log("SolvencyRegistry:  ", address(registry));
    }
}
