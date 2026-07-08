#!/usr/bin/env python3
"""Otter API load test.

Creates a configurable number of intents against a running API and reports
latency percentiles. The test is read-only with respect to on-chain execution
(conditions are not guaranteed to be met), so it is safe to run against a
live testnet agent.

Usage:
    export OTTER_API_URL=http://localhost:3002
    python3 scripts/load_test.py --intents 50 --concurrency 5
"""

import argparse
import json
import os
import statistics
import sys
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from typing import List, Tuple

DEFAULT_INTENTS = 50
DEFAULT_CONCURRENCY = 5
INTENTS = [
    "lend 1000 USDC on Aave if yield > 3",
    "swap 0.1 ETH for USDC on Uniswap if price > 1800",
    "borrow 500 DAI on Aave if yield > 2",
    "lend 1 ETH on Aave if price > 1000",
]


def api_base() -> str:
    return os.environ.get("OTTER_API_URL", "http://localhost:3002").rstrip("/")


def request_json(method: str, path: str, payload=None) -> Tuple[int, dict]:
    url = f"{api_base()}{path}"
    data = json.dumps(payload).encode("utf-8") if payload else None
    req = urllib.request.Request(url, data=data, method=method)
    if data is not None:
        req.add_header("Content-Type", "application/json")
    start = time.perf_counter()
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            body = resp.read().decode("utf-8")
            status = resp.status
    except urllib.error.HTTPError as exc:
        body = exc.read().decode("utf-8")
        status = exc.code
    elapsed = time.perf_counter() - start
    try:
        parsed = json.loads(body) if body else {}
    except json.JSONDecodeError:
        parsed = {"raw": body}
    return status, parsed, elapsed


def health_check() -> bool:
    try:
        status, _, _ = request_json("GET", "/health")
        return status == 200
    except Exception as exc:
        print(f"Health check failed: {exc}", file=sys.stderr)
        return False


def create_intent(i: int) -> Tuple[str, float, int]:
    text = INTENTS[i % len(INTENTS)]
    status, body, elapsed = request_json("POST", "/api/v1/intents", {"text": text})
    intent_id = body.get("id", "unknown") if status == 200 else f"error-{i}"
    return intent_id, elapsed, status


def parse_intent(i: int) -> Tuple[int, float]:
    text = INTENTS[i % len(INTENTS)]
    status, _, elapsed = request_json("POST", "/api/v1/intents/parse", {"text": text})
    return status, elapsed


def run_creates(total: int, concurrency: int) -> List[Tuple[str, float, int]]:
    results: List[Tuple[str, float, int]] = []
    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = {executor.submit(create_intent, i): i for i in range(total)}
        for future in as_completed(futures):
            results.append(future.result())
    return results


def run_parses(total: int, concurrency: int) -> List[Tuple[int, float]]:
    results: List[Tuple[int, float]] = []
    with ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = {executor.submit(parse_intent, i): i for i in range(total)}
        for future in as_completed(futures):
            results.append(future.result())
    return results


def summarize(name: str, latencies: List[float], statuses: List[int]) -> None:
    if not latencies:
        print(f"{name}: no samples")
        return
    latencies.sort()
    n = len(latencies)
    ok = sum(1 for s in statuses if s == 200)
    def pct(p: float) -> float:
        idx = int((p / 100.0) * (n - 1))
        return latencies[idx]
    print(f"\n{name}")
    print(f"  samples: {n}")
    print(f"  success: {ok}/{n} ({100.0 * ok / n:.1f}%)")
    print(f"  min:     {latencies[0]:.3f}s")
    print(f"  p50:     {pct(50):.3f}s")
    print(f"  p95:     {pct(95):.3f}s")
    print(f"  p99:     {pct(99):.3f}s")
    print(f"  max:     {latencies[-1]:.3f}s")
    print(f"  mean:    {statistics.mean(latencies):.3f}s")


def main() -> int:
    parser = argparse.ArgumentParser(description="Otter API load test")
    parser.add_argument("--intents", type=int, default=DEFAULT_INTENTS, help="number of intents to create")
    parser.add_argument("--concurrency", type=int, default=DEFAULT_CONCURRENCY, help="concurrent workers")
    parser.add_argument("--skip-parse", action="store_true", help="skip the parse-only warmup")
    args = parser.parse_args()

    print(f"Target API: {api_base()}")
    if not health_check():
        print("API is not healthy. Start it with `cargo run -p interfaces --bin metis_api` or Docker.", file=sys.stderr)
        return 1

    if not args.skip_parse:
        print(f"\nWarmup: parsing {args.intents} intents...")
        parse_results = run_parses(args.intents, args.concurrency)
        summarize(
            "Parse latency",
            [lat for _, lat in parse_results],
            [status for status, _ in parse_results],
        )

    print(f"\nCreating {args.intents} intents with concurrency {args.concurrency}...")
    create_results = run_creates(args.intents, args.concurrency)
    ids, latencies, statuses = zip(*create_results) if create_results else ([], [], [])
    summarize("Create-intent latency", list(latencies), list(statuses))

    print("\nSample created intent IDs:")
    for intent_id in list(ids)[:5]:
        print(f"  {intent_id}")

    print("\nLoad test complete.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
