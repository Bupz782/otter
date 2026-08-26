// SPDX-License-Identifier: MIT
pragma solidity ^0.8.21;

/// @title Interface for a Noir UltraHonk proof verifier.
/// @notice Minimal surface used by SolvencyRegistry; intentionally decoupled
/// from the generated DelegationVerifier contract.
interface IVerifier {
    /// @notice Verify a proof against its serialized public inputs.
    /// @param proof Raw proof bytes.
    /// @param publicInputs Field-element public inputs of the circuit.
    /// @return True when the proof verifies.
    function verify(bytes calldata proof, bytes32[] calldata publicInputs) external returns (bool);
}

/// @title SolvencyRegistry
/// @notice On-chain registry of the latest proof-of-solvency commitment:
/// Merkle root, aggregate deposits and last proven timestamp.
contract SolvencyRegistry {
    struct State {
        bytes32 merkleRoot;
        uint256 totalDeposits;
        uint256 lastProvenAt;
    }

    /// @notice Latest solvency state committed by a valid proof.
    State public current;

    /// @notice Verifier used to check solvency proofs.
    IVerifier public immutable verifier;

    error InvalidProof();
    error RootMismatch();
    error DepositMismatch();

    event RootUpdated(bytes32 indexed newRoot, uint256 totalDeposits, uint256 lastProvenAt);

    constructor(IVerifier verifier_) {
        verifier = verifier_;
    }

    /// @notice Commit a new solvency state backed by a valid ZK proof.
    /// @param newRoot New Merkle-sum tree root.
    /// @param totalDeposits Aggregate deposits covered by the tree.
    /// @param proof Raw proof bytes.
    /// @param publicInputs Circuit public inputs. Inputs [0..31] encode
    /// `newRoot` as 32 big-endian bytes (one byte per field element, same
    /// encoding as DelegationVault._reconstructHash); input [32] must equal
    /// `totalDeposits`.
    function updateRoot(bytes32 newRoot, uint256 totalDeposits, bytes calldata proof, bytes32[] calldata publicInputs)
        external
    {
        if (!verifier.verify(proof, publicInputs)) revert InvalidProof();

        bytes32 reconstructed;
        for (uint256 i = 0; i < 32; i++) {
            reconstructed |= publicInputs[i] << (8 * (31 - i));
        }
        if (reconstructed != newRoot) revert RootMismatch();
        if (uint256(publicInputs[32]) != totalDeposits) revert DepositMismatch();

        current.merkleRoot = newRoot;
        current.totalDeposits = totalDeposits;
        current.lastProvenAt = block.timestamp;

        emit RootUpdated(newRoot, totalDeposits, block.timestamp);
    }

    /// @notice Whether at least one valid solvency proof has been committed.
    function isSolvent() external view returns (bool) {
        return current.lastProvenAt > 0;
    }
}
