// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";
import {BridgeToken} from "./BridgeToken.sol";

/// @title OtterBridge
/// @notice V1 cross-chain bridge: lock ERC20 tokens on the source chain and
/// mint a wrapped representation on the destination chain via a trusted relayer.
///
/// @dev V1 intentionally uses an owner-gated `mint` path. Decentralized
/// verification of the source-chain Lock event is planned for V2 via a
/// messaging or ZK light-client layer.
contract OtterBridge is Ownable {
    using SafeERC20 for IERC20;

    /// @notice Underlying token locked on this (source) chain.
    IERC20 public immutable underlying;

    /// @notice BridgeToken deployed on the destination chain, minted 1:1 for
    /// locked underlying tokens. On the source deployment this can be address(0)
    /// if the source contract is not also the minter.
    BridgeToken public bridgeToken;

    /// @notice Unique bridge nonces per sender, used to generate unique ids.
    mapping(address => uint256) public nonces;

    /// @notice Bridge ids that have already been minted on the destination side.
    mapping(bytes32 => bool) public minted;

    event Lock(
        address indexed user, uint256 amount, uint256 indexed destinationChainId, bytes32 indexed bridgeId, uint256 nonce
    );

    event Mint(address indexed user, uint256 amount, bytes32 indexed bridgeId);

    error ZeroAmount();
    error AlreadyMinted(bytes32 bridgeId);
    error NoBridgeToken();

    constructor(IERC20 underlying_, address owner_) Ownable(owner_) {
        underlying = underlying_;
    }

    /// @notice Set the wrapped token that this bridge is allowed to mint.
    /// Called once after both sides of the bridge are deployed.
    function setBridgeToken(BridgeToken bridgeToken_) external onlyOwner {
        bridgeToken = bridgeToken_;
    }

    /// @notice Lock `amount` of the underlying token on this chain. Emits a
    /// `Lock` event that the trusted relayer observes to mint on the destination
    /// chain.
    /// @param amount Number of underlying tokens to lock.
    /// @param destinationChainId Destination chain where wrapped tokens should be
    /// minted.
    function lock(uint256 amount, uint256 destinationChainId) external returns (bytes32 bridgeId) {
        if (amount == 0) revert ZeroAmount();

        bridgeId = keccak256(abi.encodePacked(block.chainid, msg.sender, amount, destinationChainId, nonces[msg.sender]));
        nonces[msg.sender]++;

        underlying.safeTransferFrom(msg.sender, address(this), amount);

        emit Lock(msg.sender, amount, destinationChainId, bridgeId, nonces[msg.sender] - 1);
    }

    /// @notice Mint wrapped tokens to `user` for a previously-locked bridge.
    /// Only callable by the bridge owner (trusted relayer in V1).
    /// @param user Recipient of the wrapped tokens on this chain.
    /// @param amount Amount to mint (must match the Lock event on the source).
    /// @param bridgeId Unique identifier of the cross-chain transfer.
    function mint(address user, uint256 amount, bytes32 bridgeId) external onlyOwner {
        if (amount == 0) revert ZeroAmount();
        if (minted[bridgeId]) revert AlreadyMinted(bridgeId);
        if (address(bridgeToken) == address(0)) revert NoBridgeToken();

        minted[bridgeId] = true;
        bridgeToken.mint(user, amount);

        emit Mint(user, amount, bridgeId);
    }
}
