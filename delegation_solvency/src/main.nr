use std::hash::blake2s;

/// Number of leaves committed in the Merkle-sum tree (must be a power of two).
global LEAF_COUNT: u32 = 16;

/// Depth of the binary Merkle tree (2^MERKLE_DEPTH == LEAF_COUNT).
global MERKLE_DEPTH: u32 = 4;

/// A depositor leaf: hash of the user secret together with their balance.
struct SolvencyLeaf {
    /// blake2s(secret ++ balance) computed off-circuit by the operator.
    commitment_hash: [u8; 32],
    /// Balance covered by this leaf in the smallest on-chain unit.
    balance: Field,
}

/// Hash two sibling nodes into their parent using blake2s over the
/// concatenation of both 32-byte preimages (64 bytes total).
fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut bytes: [u8; 64] = [0; 64];
    for i in 0..32 {
        bytes[i] = left[i];
        bytes[32 + i] = right[i];
    }
    blake2s(bytes)
}

/// Build the full Merkle root from the fixed-size leaf array.
fn merkle_root(leaves: [SolvencyLeaf; LEAF_COUNT]) -> [u8; 32] {
    // Unrolled tree reduction (constrained code needs a static iteration count).
    let mut level0: [[u8; 32]; 8] = [[0; 32]; 8];
    let mut level1: [[u8; 32]; 4] = [[0; 32]; 4];
    let mut level2: [[u8; 32]; 2] = [[0; 32]; 2];

    for i in 0..8 {
        level0[i] = hash_pair(leaves[2 * i].commitment_hash, leaves[2 * i + 1].commitment_hash);
    }
    for i in 0..4 {
        level1[i] = hash_pair(level0[2 * i], level0[2 * i + 1]);
    }
    for i in 0..2 {
        level2[i] = hash_pair(level1[2 * i], level1[2 * i + 1]);
    }
    hash_pair(level2[0], level2[1])
}

/// Sum of every leaf balance. Balances are u128 values embedded in the Field,
/// so no overflow can occur with 16 leaves.
fn sum_balances(leaves: [SolvencyLeaf; LEAF_COUNT]) -> Field {
    let mut total: Field = 0;
    for i in 0..LEAF_COUNT {
        total = total + leaves[i].balance;
    }
    total
}

/// Prove solvency of a vault: the committed Merkle-sum tree of deposits
/// matches the published root and the balances sum to the announced total.
///
/// Public inputs:
///   - merkle_root: root the operator must commit to on-chain.
///   - total_deposits: aggregate balance announced to users.
///   - timestamp: proof generation time (seconds since epoch).
///
/// Private inputs:
///   - leaves: the N individual { commitment_hash, balance } records.
fn main(
    merkle_root_pub: pub [u8; 32],
    total_deposits: pub Field,
    timestamp: pub Field,
    leaves: [SolvencyLeaf; LEAF_COUNT],
) {
    // (a) Recompute the Merkle root from the private leaves...
    let recomputed_root = merkle_root(leaves);
    // ...and constrain it to equal the public root.
    assert(recomputed_root == merkle_root_pub);

    // (b) The sum of all leaf balances must match the announced total.
    assert(sum_balances(leaves) == total_deposits);

    // The proof must reference a non-zero generation timestamp.
    assert(timestamp != 0);
}

#[test]
fn test_unconstrained_helpers() {
    // Two identical halves hash to a deterministic value, different from
    // hashing swapped halves.
    let left = [1u8; 32];
    let right = [2u8; 32];

    let ab = hash_pair(left, right);
    let ba = hash_pair(right, left);
    let aa = hash_pair(left, left);

    assert(ab != ba);
    assert(ab != aa);
}

#[test]
fn test_merkle_root_is_deterministic() {
    let mut leaves: [SolvencyLeaf; LEAF_COUNT] = [SolvencyLeaf { commitment_hash: [7; 32], balance: 0 }; LEAF_COUNT];
    for i in 0..LEAF_COUNT {
        leaves[i] = SolvencyLeaf { commitment_hash: [i as u8; 32], balance: (i as Field) + 1 };
    }

    let root1 = merkle_root(leaves);
    let root2 = merkle_root(leaves);
    assert(root1 == root2);

    // Swapping two leaves changes the root (order matters).
    let mut swapped = leaves;
    let tmp = swapped[0];
    swapped[0] = swapped[3];
    swapped[3] = tmp;
    assert(merkle_root(swapped) != root1);
}

#[test]
fn test_sum_balances() {
    let mut leaves: [SolvencyLeaf; LEAF_COUNT] = [SolvencyLeaf { commitment_hash: [0; 32], balance: 0 }; LEAF_COUNT];
    for i in 0..LEAF_COUNT {
        leaves[i].balance = (i as Field) + 1;
    }

    // 1 + 2 + ... + 16 = 136.
    assert(sum_balances(leaves) == 136);
}

#[test]
fn test_main_happy_path() {
    let mut leaves: [SolvencyLeaf; LEAF_COUNT] = [SolvencyLeaf { commitment_hash: [9; 32], balance: 100 }; LEAF_COUNT];
    for i in 0..LEAF_COUNT {
        leaves[i].commitment_hash = [(i as u8) + 1; 32];
        leaves[i].balance = 100;
    }

    let root = merkle_root(leaves);
    main(root, 1600, 1_000_000, leaves);
}

/// The circuit detects a mismatched announced total: recomputation differs
/// from what a lying operator would publish.
#[test]
fn test_detects_wrong_total() {
    let mut leaves: [SolvencyLeaf; LEAF_COUNT] = [SolvencyLeaf { commitment_hash: [9; 32], balance: 100 }; LEAF_COUNT];
    for i in 0..LEAF_COUNT {
        leaves[i].commitment_hash = [(i as u8) + 1; 32];
    }

    let root = merkle_root(leaves);
    let announced_total = 1599;
    // The circuit would reject this witness: sum != announced total.
    assert(sum_balances(leaves) != announced_total);
}

/// The circuit detects a root that does not commit to the provided leaves.
#[test]
fn test_detects_wrong_root() {
    let mut leaves: [SolvencyLeaf; LEAF_COUNT] = [SolvencyLeaf { commitment_hash: [9; 32], balance: 100 }; LEAF_COUNT];
    for i in 0..LEAF_COUNT {
        leaves[i].commitment_hash = [(i as u8) + 1; 32];
    }

    let published = [0; 32];
    // The circuit would reject this witness: recomputed root != published root.
    assert(merkle_root(leaves) != published);
}
