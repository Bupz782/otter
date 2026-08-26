// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {Test} from "forge-std/Test.sol";
import {IVerifier, SolvencyRegistry} from "../src/SolvencyRegistry.sol";

/// @dev Mock verifier that accepts or rejects proofs based on a flag set by
/// the test, so forge tests run without barretenberg or real ZK fixtures.
contract MockVerifier is IVerifier {
    bool public accept = true;

    function setAccept(bool accept_) external {
        accept = accept_;
    }

    function verify(bytes calldata, bytes32[] calldata) external view returns (bool) {
        return accept;
    }
}

contract SolvencyRegistryTest is Test {
    MockVerifier internal mockVerifier;
    SolvencyRegistry internal registry;

    bytes32 constant ROOT = 0x123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef0;
    uint256 constant DEPOSITS = 1_000_000 ether;

    event RootUpdated(bytes32 indexed newRoot, uint256 totalDeposits, uint256 lastProvenAt);

    function setUp() public {
        mockVerifier = new MockVerifier();
        registry = new SolvencyRegistry(mockVerifier);
    }

    /// @dev Build 33 public inputs: [0..31] = big-endian bytes of `root`,
    /// [32] = deposits (same encoding as DelegationVault._reconstructHash).
    function _publicInputs(bytes32 root, uint256 deposits) internal pure returns (bytes32[] memory inputs) {
        inputs = new bytes32[](33);
        for (uint256 i = 0; i < 32; i++) {
            inputs[i] = bytes32(uint256(uint8(root[i])));
        }
        inputs[32] = bytes32(deposits);
    }

    function test_IsSolventFalseBeforeAnyProof() public view {
        assertFalse(registry.isSolvent());
        (bytes32 root0,, uint256 ts0) = registry.current();
        assertEq(root0, bytes32(0));
        assertEq(ts0, 0);
    }

    function test_UpdateRootWithValidProofUpdatesStateAndEmits() public {
        mockVerifier.setAccept(true);

        vm.expectEmit(true, true, true, true);
        emit RootUpdated(ROOT, DEPOSITS, block.timestamp);
        registry.updateRoot(ROOT, DEPOSITS, hex"deadbeef", _publicInputs(ROOT, DEPOSITS));

        (bytes32 root1, uint256 dep1, uint256 ts1) = registry.current();
        assertEq(root1, ROOT);
        assertEq(dep1, DEPOSITS);
        assertEq(ts1, block.timestamp);
        assertTrue(registry.isSolvent());
    }

    function test_RevertWhen_ProofInvalid() public {
        mockVerifier.setAccept(false);
        vm.expectRevert(SolvencyRegistry.InvalidProof.selector);
        registry.updateRoot(ROOT, DEPOSITS, "", _publicInputs(ROOT, DEPOSITS));
        assertFalse(registry.isSolvent());
    }

    function test_RevertWhen_RootMismatchInPublicInputs() public {
        mockVerifier.setAccept(true);
        bytes32 forged = keccak256("forged");
        vm.expectRevert(SolvencyRegistry.RootMismatch.selector);
        registry.updateRoot(forged, DEPOSITS, hex"deadbeef", _publicInputs(ROOT, DEPOSITS));
    }

    function test_RevertWhen_TotalDepositsMismatch() public {
        mockVerifier.setAccept(true);
        vm.expectRevert(SolvencyRegistry.DepositMismatch.selector);
        registry.updateRoot(ROOT, DEPOSITS + 1, hex"deadbeef", _publicInputs(ROOT, DEPOSITS));
    }

    function test_SecondUpdateOverwritesState() public {
        mockVerifier.setAccept(true);
        registry.updateRoot(ROOT, DEPOSITS, hex"01", _publicInputs(ROOT, DEPOSITS));

        uint256 later = block.timestamp + 1 hours;
        vm.warp(later);

        bytes32 newRoot = keccak256("root-2");
        uint256 newDeposits = 500 ether;
        vm.expectEmit(true, true, true, true);
        emit RootUpdated(newRoot, newDeposits, later);
        registry.updateRoot(newRoot, newDeposits, hex"02", _publicInputs(newRoot, newDeposits));

        (bytes32 root2, uint256 dep2, uint256 ts2) = registry.current();
        assertEq(root2, newRoot);
        assertEq(dep2, newDeposits);
        assertEq(ts2, later);
    }

    function testFuzz_UpdateRoot(bytes32 root, uint64 ts, uint128 deposits) public {
        vm.assume(deposits < type(uint256).max / 2);
        vm.assume(ts > 0);
        vm.warp(ts);
        mockVerifier.setAccept(true);
        registry.updateRoot(root, deposits, hex"f00d", _publicInputs(root, deposits));
        (bytes32 rootF, uint256 depF,) = registry.current();
        assertEq(rootF, root);
        assertEq(depF, deposits);
        assertTrue(registry.isSolvent());
    }
}
