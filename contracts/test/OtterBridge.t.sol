// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {Test} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {OtterBridge} from "../src/OtterBridge.sol";
import {BridgeToken} from "../src/BridgeToken.sol";
import {TestToken} from "../src/TestToken.sol";

contract OtterBridgeTest is Test {
    TestToken internal underlying;
    OtterBridge internal sourceBridge;
    OtterBridge internal destBridge;
    BridgeToken internal wrapped;

    address internal owner = makeAddr("owner");
    address internal user = makeAddr("user");

    function setUp() public {
        underlying = new TestToken("Underlying", "UND", 18);
        sourceBridge = new OtterBridge(IERC20(address(underlying)), owner);
        destBridge = new OtterBridge(IERC20(address(0)), owner);
        vm.prank(owner);
        wrapped = new BridgeToken("Wrapped Underlying", "wUND", address(destBridge));
        vm.prank(owner);
        destBridge.setBridgeToken(wrapped);

        underlying.transfer(user, 10_000 ether);
    }

    function test_LockTransfersTokensToBridge() public {
        vm.startPrank(user);
        underlying.approve(address(sourceBridge), 1_000 ether);
        bytes32 bridgeId = sourceBridge.lock(1_000 ether, 42);
        vm.stopPrank();

        assertEq(underlying.balanceOf(user), 9_000 ether);
        assertEq(underlying.balanceOf(address(sourceBridge)), 1_000 ether);
        assertEq(sourceBridge.nonces(user), 1);
        assertTrue(bridgeId != bytes32(0));
    }

    function test_MintWrappedTokensByRelayer() public {
        uint256 amount = 1_000 ether;
        bytes32 bridgeId = keccak256("bridge-id");

        vm.prank(owner);
        destBridge.mint(user, amount, bridgeId);

        assertEq(wrapped.balanceOf(user), amount);
        assertTrue(destBridge.minted(bridgeId));
    }

    function test_RevertWhen_MintingTwice() public {
        uint256 amount = 1_000 ether;
        bytes32 bridgeId = keccak256("bridge-id");

        vm.startPrank(owner);
        destBridge.mint(user, amount, bridgeId);
        vm.expectRevert(abi.encodeWithSelector(OtterBridge.AlreadyMinted.selector, bridgeId));
        destBridge.mint(user, amount, bridgeId);
        vm.stopPrank();
    }

    function test_RevertWhen_MintingWithoutBridgeToken() public {
        OtterBridge standalone = new OtterBridge(IERC20(address(0)), owner);
        vm.prank(owner);
        vm.expectRevert(OtterBridge.NoBridgeToken.selector);
        standalone.mint(user, 1_000 ether, keccak256("id"));
    }

    function test_RevertWhen_ZeroAmountLock() public {
        vm.prank(user);
        vm.expectRevert(OtterBridge.ZeroAmount.selector);
        sourceBridge.lock(0, 42);
    }

    function test_NonOwnerCannotMint() public {
        vm.prank(user);
        vm.expectRevert();
        destBridge.mint(user, 1_000 ether, keccak256("id"));
    }
}
