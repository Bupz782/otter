#!/usr/bin/env python3
"""Generate delegation_solvency/Prover.toml and a JSON summary fixture."""
import hashlib
import json
import os
import sys

ROOT_DIR = os.path.join(os.path.dirname(__file__), "..")
CIRCUIT_DIR = os.path.join(ROOT_DIR, "delegation_solvency")
FIXTURE_DIR = os.path.join(ROOT_DIR, "contracts", "test", "fixtures", "solvency")

LEAF_COUNT = 16


def hash_pair(left: bytes, right: bytes) -> bytes:
    return hashlib.blake2s(left + right).digest()


def build_leaves():
    leaves = []
    for i in range(LEAF_COUNT):
        secret = bytes([i % 256] * 32)
        commitment = hashlib.blake2s(secret).digest()
        leaves.append({"commitment_hash": list(commitment), "balance": 100})
    return leaves


def merkle_root(leaves):
    level = [bytes(leaf["commitment_hash"]) for leaf in leaves]
    while len(level) > 1:
        next_level = []
        for i in range(0, len(level), 2):
            next_level.append(hash_pair(level[i], level[i + 1]))
        level = next_level
    return level[0]


def main():
    leaves = build_leaves()
    root = merkle_root(leaves)
    total = sum(leaf["balance"] for leaf in leaves)
    timestamp = 1_720_000_000

    os.makedirs(CIRCUIT_DIR, exist_ok=True)
    os.makedirs(FIXTURE_DIR, exist_ok=True)

    lines = [
        "merkle_root_pub = [" + ", ".join(str(b) for b in root) + "]",
        f'total_deposits = "{total}"',
        f'timestamp = "{timestamp}"',
        "",
    ]
    for leaf in leaves:
        hash_str = "[" + ", ".join(str(b) for b in leaf["commitment_hash"]) + "]"
        lines.append("[[leaves]]")
        lines.append(f"commitment_hash = {hash_str}")
        lines.append(f'balance = "{leaf["balance"]}"')
        lines.append("")

    with open(os.path.join(CIRCUIT_DIR, "Prover.toml"), "w") as f:
        f.write("\n".join(lines) + "\n")

    summary = {
        "merkle_root": "0x" + root.hex(),
        "total_deposits": str(total),
        "timestamp": str(timestamp),
    }
    with open(os.path.join(FIXTURE_DIR, "fixture.json"), "w") as f:
        json.dump(summary, f, indent=2)

    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    main()
