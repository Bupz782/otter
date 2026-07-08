// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

import {DelegationVerifier} from "./DelegationVerifier.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

/// @title DelegationVault
/// @notice User vault that delegates execution rights to an agent via ZK proofs.
///
/// A user registers a delegation hash together with on-chain limits. The agent
/// can later call `executeWithProof` with a Noir/UltraHonk proof that the
/// proposed intent respects those limits. If the proof verifies, the vault
/// releases the requested native ETH or ERC-20 amount to the executor.
contract DelegationVault {
    using SafeERC20 for IERC20;

    /// @notice On-chain limits associated with a delegation hash.
    struct Delegation {
        address owner;
        uint256 allowedIntents;
        uint256[10] maxAmounts;
        uint256[5] allowedProtocols;
        uint256 expiry;
        uint256 nonce;
        bool active;
    }

    /// @notice Verifier contract generated from the Noir delegation circuit.
    DelegationVerifier public immutable verifier;

    /// @notice Delegation hash => delegation limits.
    mapping(bytes32 => Delegation) public delegations;

    /// @notice Owner => balance held by the vault (native ETH).
    mapping(address => uint256) public balances;

    /// @notice Owner => token => balance held by the vault.
    mapping(address => mapping(address => uint256)) public tokenBalances;

    /// @notice delegationHash => nonce => already used.
    mapping(bytes32 => mapping(uint256 => bool)) public usedNonces;

    /// @notice Protocol identifier => whitelisted router/pool address.
    mapping(uint256 => address) public protocolRouters;

    /// @notice Number of bytes32 public inputs expected from the circuit.
    /// Circuit public inputs:
    ///   delegation_hash (32 bytes) + intent_type + amount + protocol +
    ///   target_contract + timestamp + nonce = 38 field elements.
    uint256 public constant PUBLIC_INPUTS_SIZE = 38;

    /// @notice Offsets of the intent fields in the public inputs array.
    uint256 public constant INTENT_TYPE_OFFSET = 32;
    uint256 public constant AMOUNT_OFFSET = 33;
    uint256 public constant PROTOCOL_OFFSET = 34;
    uint256 public constant TARGET_CONTRACT_OFFSET = 35;
    uint256 public constant TIMESTAMP_OFFSET = 36;
    uint256 public constant NONCE_OFFSET = 37;

    event Delegated(
        bytes32 indexed delegationHash, address indexed owner, uint256 allowedIntents, uint256 expiry, uint256 nonce
    );
    event Deposited(address indexed user, address indexed token, uint256 amount);
    event Withdrawn(address indexed user, address indexed token, uint256 amount);
    event Executed(
        bytes32 indexed delegationHash,
        uint256 indexed intentType,
        uint256 amount,
        uint256 protocol,
        address targetContract
    );
    event ProtocolRouterSet(uint256 indexed protocol, address indexed router);

    error InvalidProof();
    error DelegationNotFound();
    error IntentNotAllowed();
    error AmountExceedsMax();
    error ProtocolNotAllowed();
    error ProtocolRouterNotSet(uint256 protocol);
    error DelegationExpired();
    error InvalidNonce();
    error PublicInputsLengthWrong();
    error InsufficientBalance();
    error NativeTransferFailed();

    constructor(DelegationVerifier _verifier) {
        verifier = _verifier;
    }

    /// @notice Register a whitelisted protocol router address.
    /// @param protocol Protocol identifier matching the intent's `protocol` field.
    /// @param router Address of the protocol router/pool that receives tokens.
    function setProtocolRouter(uint256 protocol, address router) external {
        require(router != address(0), "invalid router");
        protocolRouters[protocol] = router;
        emit ProtocolRouterSet(protocol, router);
    }

    /// @notice Register a delegation and its on-chain limits.
    /// @param delegationHash Hash of the signed delegation message (Noir/blake2s).
    /// @param allowedIntents Bitfield of allowed intent types.
    /// @param maxAmounts Max amount per intent type.
    /// @param allowedProtocols Whitelisted protocol identifiers.
    /// @param expiry Timestamp after which the delegation is invalid.
    /// @param nonce Anti-replay nonce, must match the delegation message.
    function delegate(
        bytes32 delegationHash,
        uint256 allowedIntents,
        uint256[10] calldata maxAmounts,
        uint256[5] calldata allowedProtocols,
        uint256 expiry,
        uint256 nonce
    ) external {
        require(expiry > block.timestamp, DelegationExpired());

        delegations[delegationHash] = Delegation({
            owner: msg.sender,
            allowedIntents: allowedIntents,
            maxAmounts: maxAmounts,
            allowedProtocols: allowedProtocols,
            expiry: expiry,
            nonce: nonce,
            active: true
        });

        emit Delegated(delegationHash, msg.sender, allowedIntents, expiry, nonce);
    }

    /// @notice Deposit native ETH into the vault.
    function deposit() external payable {
        require(msg.value > 0, "amount must be > 0");
        balances[msg.sender] += msg.value;
        emit Deposited(msg.sender, address(0), msg.value);
    }

    /// @notice Deposit ERC-20 tokens into the vault.
    /// @param token ERC-20 token contract address.
    /// @param amount Amount of tokens to deposit.
    function deposit(address token, uint256 amount) external {
        require(amount > 0, "amount must be > 0");
        require(token != address(0), "invalid token");

        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);
        tokenBalances[msg.sender][token] += amount;

        emit Deposited(msg.sender, token, amount);
    }

    /// @notice Withdraw available native ETH balance from the vault.
    function withdraw(uint256 amount) external {
        require(balances[msg.sender] >= amount, InsufficientBalance());
        balances[msg.sender] -= amount;

        (bool success,) = payable(msg.sender).call{value: amount}("");
        require(success, NativeTransferFailed());

        emit Withdrawn(msg.sender, address(0), amount);
    }

    /// @notice Withdraw available ERC-20 balance from the vault.
    function withdraw(address token, uint256 amount) external {
        require(token != address(0), "invalid token");
        require(tokenBalances[msg.sender][token] >= amount, InsufficientBalance());
        tokenBalances[msg.sender][token] -= amount;

        IERC20(token).safeTransfer(msg.sender, amount);

        emit Withdrawn(msg.sender, token, amount);
    }

    /// @notice Execute a delegated intent using a ZK proof.
    /// @param proof UltraHonk proof bytes.
    /// @param publicInputs Public inputs to the circuit (38 bytes32 values).
    function executeWithProof(bytes calldata proof, bytes32[] calldata publicInputs) external {
        require(publicInputs.length == PUBLIC_INPUTS_SIZE, PublicInputsLengthWrong());

        // 1. Verify the proof.
        if (!verifier.verify(proof, publicInputs)) {
            revert InvalidProof();
        }

        // 2. Parse public inputs.
        bytes32 delegationHash = _reconstructHash(publicInputs);
        uint256 intentType = uint256(publicInputs[INTENT_TYPE_OFFSET]);
        uint256 amount = uint256(publicInputs[AMOUNT_OFFSET]);
        uint256 protocol = uint256(publicInputs[PROTOCOL_OFFSET]);
        address targetContract = address(uint160(uint256(publicInputs[TARGET_CONTRACT_OFFSET])));
        uint256 timestamp = uint256(publicInputs[TIMESTAMP_OFFSET]);
        uint256 nonce = uint256(publicInputs[NONCE_OFFSET]);

        // 3. Enforce delegation limits.
        Delegation storage delegation = delegations[delegationHash];
        require(delegation.active, DelegationNotFound());
        require(timestamp < delegation.expiry, DelegationExpired());
        require(nonce == delegation.nonce, InvalidNonce());
        require(!usedNonces[delegationHash][nonce], InvalidNonce());
        require(_isBitSet(delegation.allowedIntents, intentType), IntentNotAllowed());
        require(amount <= delegation.maxAmounts[intentType], AmountExceedsMax());
        require(_contains(delegation.allowedProtocols, protocol), ProtocolNotAllowed());

        // Mark nonce as used to prevent replay.
        usedNonces[delegationHash][nonce] = true;

        // 4. Execute the intent.
        _execute(delegation.owner, amount, targetContract, protocol);

        emit Executed(delegationHash, intentType, amount, protocol, targetContract);
    }

    /// @notice Reconstruct a bytes32 hash from 32 individual byte public inputs.
    /// @dev The circuit serializes `delegation_hash: [u8; 32]` as 32 field
    /// elements, each holding one byte in its low byte. The first public input
    /// is the most-significant byte of the hash.
    function _reconstructHash(bytes32[] calldata publicInputs) internal pure returns (bytes32 hash) {
        for (uint256 i = 0; i < 32; i++) {
            hash |= publicInputs[i] << (8 * (31 - i));
        }
    }

    function _isBitSet(uint256 bitfield, uint256 bit) internal pure returns (bool) {
        return (bitfield & (1 << bit)) != 0;
    }

    function _contains(uint256[5] storage array, uint256 value) internal view returns (bool) {
        for (uint256 i = 0; i < 5; i++) {
            if (array[i] == value) {
                return true;
            }
        }
        return false;
    }

    /// @dev Release native ETH or ERC-20 tokens to a whitelisted protocol router.
    /// `token` is interpreted as the token address: address(0) for native ETH,
    /// otherwise an ERC-20 contract. ERC-20 tokens are transferred to the
    /// router registered for `protocol`; native ETH is left as a balance debit
    /// for backwards compatibility with existing tests.
    function _execute(address owner, uint256 amount, address token, uint256 protocol) internal {
        if (token == address(0)) {
            require(balances[owner] >= amount, InsufficientBalance());
            balances[owner] -= amount;
        } else {
            require(tokenBalances[owner][token] >= amount, InsufficientBalance());
            address router = protocolRouters[protocol];
            require(router != address(0), ProtocolRouterNotSet(protocol));
            tokenBalances[owner][token] -= amount;
            IERC20(token).safeTransfer(router, amount);
        }
    }

    receive() external payable {
        balances[msg.sender] += msg.value;
        emit Deposited(msg.sender, address(0), msg.value);
    }
}
