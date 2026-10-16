"""Compare the frozen hybrid baseline with a keyword-first local retrieval path.

All labels and vectors come from an existing synthetic cohort. No provider is called.
"""

import argparse
import json
import os
import statistics
import subprocess
import time
from pathlib import Path


class Session:
    def __init__(self, executable: Path, database: Path):
        env = os.environ.copy()
        env["IROMEM_DATABASE_PATH"] = str(database)
        self.process = subprocess.Popen(
            [str(executable), "session"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            env=env,
        )

    def call(self, op: str, request: dict) -> tuple[dict, float]:
        started = time.perf_counter_ns()
        self.process.stdin.write(json.dumps({"op": op, "request": request}) + "\n")
        self.process.stdin.flush()
        line = self.process.stdout.readline()
        elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000
        if not line:
            raise RuntimeError(f"session exited: {self.process.stderr.read()}")
        result = json.loads(line)
        if result["status"] != "complete":
            raise RuntimeError(result["error"])
        return result["response"], elapsed_ms

    def close(self):
        self.process.stdin.close()
        self.process.wait(timeout=10)
        if self.process.returncode:
            raise RuntimeError(self.process.stderr.read())
        self.process.stdout.close()
        self.process.stderr.close()


def percentile(values: list[float], fraction: float) -> float:
    values = sorted(values)
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def summarize(rows: list[dict], key: str) -> dict:
    positive = [r for r in rows if r["expected_ids"]]
    negative = [r for r in rows if not r["expected_ids"]]
    times = [r[key]["elapsed_ms"] for r in rows]
    return {
        "p50_ms": round(statistics.median(times), 3),
        "p95_ms": round(percentile(times, 0.95), 3),
        "top1": sum(r[key]["candidate_ids"][:1] == r["expected_ids"] for r in positive),
        "top5": sum(bool(set(r[key]["candidate_ids"][:5]) & set(r["expected_ids"])) for r in positive),
        "top20": sum(bool(set(r[key]["candidate_ids"][:20]) & set(r["expected_ids"])) for r in positive),
        "positives": len(positive),
        "negatives_with_candidates": sum(bool(r[key]["candidate_ids"]) for r in negative),
        "negatives": len(negative),
        "fallbacks": sum(r[key].get("fallback", False) for r in rows),
        "candidate_count_p50": statistics.median(len(r[key]["candidate_ids"]) for r in rows),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--before", type=Path, required=True)
    parser.add_argument("--after", type=Path, required=True)
    parser.add_argument("--database", type=Path, required=True)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    cases = [json.loads(line) for line in args.queries.read_text().splitlines() if line]
    before = Session(args.before.resolve(), args.database.resolve())
    after = Session(args.after.resolve(), args.database.resolve())
    try:
        for case in cases[:10]:
            before.call("hybrid_search", case["request"])
            search = {**case["request"]["search"], "query_mode": "keywords"}
            after.call("search", search)
        rows = []
        for case in cases:
            request = case["request"]
            baseline, before_ms = before.call("hybrid_search", request)
            keyword_request = {**request["search"], "query_mode": "keywords"}
            keyword, keyword_ms = after.call("search", keyword_request)
            keyword_hybrid, keyword_hybrid_ms = after.call(
                "hybrid_search", {**request, "search": keyword_request}
            )
            fallback = not keyword["candidates"]
            adaptive = keyword
            adaptive_ms = keyword_ms
            if fallback:
                adaptive, second_ms = after.call("hybrid_search", request)
                adaptive_ms += second_ms
            rows.append({
                "case_id": case["id"],
                "segment": case["segment"],
                "expected_ids": case["expected_ids"],
                "before_hybrid": {
                    "elapsed_ms": before_ms,
                    "candidate_ids": [c["memory_id"] for c in baseline["candidates"]],
                },
                "after_keywords": {
                    "elapsed_ms": keyword_ms,
                    "candidate_ids": [c["memory_id"] for c in keyword["candidates"]],
                },
                "after_hybrid_keywords": {
                    "elapsed_ms": keyword_hybrid_ms,
                    "candidate_ids": [c["memory_id"] for c in keyword_hybrid["candidates"]],
                },
                "after_adaptive": {
                    "elapsed_ms": adaptive_ms,
                    "candidate_ids": [c["memory_id"] for c in adaptive["candidates"]],
                    "fallback": fallback,
                },
            })
        output = {
            "inputs": {"before": str(args.before), "after": str(args.after),
                       "database": str(args.database), "queries": str(args.queries)},
            "summary": {key: summarize(rows, key) for key in
                        ("before_hybrid", "after_keywords", "after_hybrid_keywords", "after_adaptive")},
            "rows": rows,
        }
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(output, ensure_ascii=False, indent=2) + "\n")
        print(json.dumps(output["summary"], ensure_ascii=False, indent=2))
    finally:
        before.close()
        after.close()


if __name__ == "__main__":
    main()
