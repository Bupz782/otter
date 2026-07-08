// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {Test} from "forge-std/Test.sol";
import {ERC20Mock} from "@openzeppelin/contracts/mocks/token/ERC20Mock.sol";
import {DelegationVerifier} from "../src/DelegationVerifier.sol";
import {DelegationVault} from "../src/DelegationVault.sol";

contract DelegationVaultTest is Test {
    DelegationVerifier public verifier;
    DelegationVault public vault;

    bytes32 public delegationHash =
        0x91ade02b79eb31565b2b5e9cbf73c2113af09524cc4dac59305b8a9ef7fad9f5;
    uint256 public allowedIntents = 0x05;
    uint256[10] public maxAmounts = [
        uint256(1_000_000),
        2_000_000,
        3_000_000,
        0,
        0,
        0,
        0,
        0,
        0,
        0
    ];
    uint256[5] public allowedProtocols = [uint256(1), 2, 0, 0, 0];
    uint256 public expiry = 4_000_000_000;
    uint256 public nonce = 42;

    function setUp() public {
        verifier = new DelegationVerifier();
        vault = new DelegationVault(verifier);
    }

    function _loadPublicInputs() internal view returns (bytes32[] memory) {
        bytes memory publicInputsBytes = vm.readFileBinary(
            "test/fixtures/public_inputs.bin"
        );
        require(
            publicInputsBytes.length == vault.PUBLIC_INPUTS_SIZE() * 32,
            "fixture size mismatch"
        );

        bytes32[] memory publicInputs = new bytes32[](vault.PUBLIC_INPUTS_SIZE());
        for (uint256 i = 0; i < vault.PUBLIC_INPUTS_SIZE(); i++) {
            bytes32 value;
            assembly {
                value := mload(add(publicInputsBytes, add(32, mul(i, 32))))
            }
            publicInputs[i] = value;
        }
        return publicInputs;
    }

    function test_executeWithProof_succeeds() public {
        bytes memory proof = vm.readFileBinary("test/fixtures/proof.bin");
        bytes32[] memory publicInputs = _loadPublicInputs();

        vm.prank(alice);
        vault.delegate(
            delegationHash,
            allowedIntents,
            maxAmounts,
            allowedProtocols,
            expiry,
            nonce
        );

        vm.deal(alice, 10 ether);
        vm.prank(alice);
        vault.deposit{value: 10 ether}();
        assertEq(vault.balances(alice), 10 ether);

        // The fixture executes intent type 2 with amount 2_000_000.
        uint256 amount = 2_000_000;
        vm.prank(agent);
        vault.executeWithProof(proof, publicInputs);

        assertEq(vault.balances(alice), 10 ether - amount);
        assertTrue(vault.usedNonces(delegationHash, nonce));
    }

    function test_executeWithProof_revertsWhenDelegationNotFound() public {
        bytes memory proof = vm.readFileBinary("test/fixtures/proof.bin");
        bytes32[] memory publicInputs = _loadPublicInputs();

        vm.expectRevert(DelegationVault.DelegationNotFound.selector);
        vault.executeWithProof(proof, publicInputs);
    }

    function test_executeWithProof_revertsWhenIntentNotAllowed() public {
        bytes memory proof = vm.readFileBinary("test/fixtures/proof.bin");
        bytes32[] memory publicInputs = _loadPublicInputs();

        // The fixture uses intent type 2; remove it from the bitfield.
        uint256 badAllowedIntents = 0x01;

        vm.prank(alice);
        vault.delegate(
            delegationHash,
            badAllowedIntents,
            maxAmounts,
            allowedProtocols,
            expiry,
            nonce
        );

        vm.deal(alice, 10 ether);
        vm.prank(alice);
        vault.deposit{value: 10 ether}();

        vm.expectRevert(DelegationVault.IntentNotAllowed.selector);
        vault.executeWithProof(proof, publicInputs);
    }

    function test_executeWithProof_revertsWhenProofTampered() public {
        bytes memory proof = vm.readFileBinary("test/fixtures/proof.bin");
        proof[0] = bytes1(uint8(proof[0]) ^ 0xff);
        bytes32[] memory publicInputs = _loadPublicInputs();

        vm.prank(alice);
        vault.delegate(
            delegationHash,
            allowedIntents,
            maxAmounts,
            allowedProtocols,
            expiry,
            nonce
        );

        vm.expectRevert();
        vault.executeWithProof(proof, publicInputs);
    }

    function test_executeWithProof_revertsOnReplay() public {
        bytes memory proof = vm.readFileBinary("test/fixtures/proof.bin");
        bytes32[] memory publicInputs = _loadPublicInputs();

        vm.deal(alice, 10 ether);
        vm.startPrank(alice);
        vault.delegate(
            delegationHash,
            allowedIntents,
            maxAmounts,
            allowedProtocols,
            expiry,
            nonce
        );
        vault.deposit{value: 10 ether}();
        vm.stopPrank();

        assertTrue(verifier.verify(proof, publicInputs), "first direct verify");
        vault.executeWithProof(proof, publicInputs);
        assertTrue(verifier.verify(proof, publicInputs), "second direct verify");

        vm.expectRevert(DelegationVault.InvalidNonce.selector);
        vault.executeWithProof(proof, publicInputs);
    }

    address alice = makeAddr("alice");
    address agent = makeAddr("agent");

    function test_deposit_withdraw_erc20() public {
        ERC20Mock token = new ERC20Mock();
        token.mint(alice, 10_000e6);

        vm.startPrank(alice);
        token.approve(address(vault), 5_000e6);
        vault.deposit(address(token), 5_000e6);
        vm.stopPrank();

        assertEq(vault.tokenBalances(alice, address(token)), 5_000e6);
        assertEq(token.balanceOf(address(vault)), 5_000e6);

        vm.prank(alice);
        vault.withdraw(address(token), 2_000e6);

        assertEq(vault.tokenBalances(alice, address(token)), 3_000e6);
        assertEq(token.balanceOf(alice), 7_000e6);
    }
}
