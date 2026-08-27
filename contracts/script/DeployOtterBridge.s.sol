// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {Script, console} from "forge-std/Script.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {OtterBridge} from "../src/OtterBridge.sol";
import {BridgeToken} from "../src/BridgeToken.sol";

/// @notice Deploy one side of the OtterBridge (lock on source, mint on destination).
///
/// Run with:
///   UNDERLYING_ADDRESS=0x... forge script script/DeployOtterBridge.s.sol \
///     --rpc-url http://localhost:8545 --broadcast
/// BRIDGE_OWNER overrides the owner (defaults to the broadcaster; must be the
/// API signer for the owner-gated mint path). Set BRIDGE_TOKEN_NAME (and
/// optionally BRIDGE_TOKEN_SYMBOL) on the destination side to also deploy the
/// wrapped token and wire it via setBridgeToken.
contract DeployOtterBridge is Script {
    function run() public {
        address underlying = vm.envAddress("UNDERLYING_ADDRESS");
        address owner = vm.envOr("BRIDGE_OWNER", msg.sender);

        vm.startBroadcast();

        OtterBridge bridge = new OtterBridge(IERC20(underlying), owner);

        string memory tokenName = vm.envOr("BRIDGE_TOKEN_NAME", string(""));
        if (bytes(tokenName).length > 0) {
            string memory tokenSymbol = vm.envOr("BRIDGE_TOKEN_SYMBOL", string("otTOK"));
            BridgeToken token = new BridgeToken(tokenName, tokenSymbol, address(bridge));
            bridge.setBridgeToken(token);
            console.log("BridgeToken deployed at:", address(token));
        }

        vm.stopBroadcast();

        console.log("OtterBridge deployed at:", address(bridge));
        console.log("Owner:", owner);
    }
}
