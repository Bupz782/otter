// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {Script} from "forge-std/Script.sol";
import {console2} from "forge-std/console2.sol";
import {stdJson} from "forge-std/StdJson.sol";
import {IVerifier, SolvencyRegistry} from "../src/SolvencyRegistry.sol";
import {SolvencyVerifier} from "../src/SolvencyVerifier.sol";

contract DeploySolvencyRegistry is Script {
    using stdJson for string;

    string constant FIXTURE_DIR = "test/fixtures/solvency/";

    function run() external returns (SolvencyRegistry registry) {
        // 1. Optionally reuse an existing verifier, otherwise deploy the
        // freshly generated UltraHonk verifier for the solvency circuit.
        address verifierAddr = vm.envOr("SOLVENCY_VERIFIER_ADDRESS", address(0));
        if (verifierAddr == address(0)) {
            vm.startBroadcast();
            SolvencyVerifier verifier = new SolvencyVerifier();
            vm.stopBroadcast();
            verifierAddr = address(verifier);
            console2.log("SolvencyVerifier deployed at:", verifierAddr);
        }

        // 2. Deploy the registry backed by this verifier.
        vm.startBroadcast();
        registry = new SolvencyRegistry(IVerifier(verifierAddr));
        vm.stopBroadcast();

        console2.log("SolvencyRegistry deployed at:", address(registry));
        console2.log("Verifier:", verifierAddr);

        // 3. Seed the registry with the real ZK fixture (unless explicitly skipped).
        bool skipProof = vm.envOr("SKIP_INITIAL_PROOF", false);
        if (!skipProof) {
            _updateRoot(registry);
        }
    }

    function _updateRoot(SolvencyRegistry registry) internal {
        string memory fixtureJson = vm.readFile(string.concat(FIXTURE_DIR, "fixture.json"));
        uint256 rootUint = vm.parseJsonUint(fixtureJson, ".merkle_root");
        uint256 totalDeposits = vm.parseJsonUint(fixtureJson, ".total_deposits");
        bytes memory proof = vm.readFileBinary(string.concat(FIXTURE_DIR, "proof"));

        bytes32 root = bytes32(rootUint);

        // Public inputs are 34 field elements: 32 root bytes, total deposits,
        // then timestamp. The registry only consumes the first 33 elements.
        bytes32[] memory publicInputs = new bytes32[](34);
        for (uint256 i = 0; i < 32; i++) {
            publicInputs[i] = bytes32(uint256(uint8(bytes32(root)[i])));
        }
        publicInputs[32] = bytes32(totalDeposits);
        publicInputs[33] = bytes32(vm.parseJsonUint(fixtureJson, ".timestamp"));

        vm.startBroadcast();
        registry.updateRoot(root, totalDeposits, proof, publicInputs);
        vm.stopBroadcast();

        console2.log("Seeded registry with root:", uint256(root));
        console2.log("Total deposits:", totalDeposits);
    }
}
