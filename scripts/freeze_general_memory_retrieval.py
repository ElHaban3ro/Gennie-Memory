#!/usr/bin/env python3
"""Validate and freeze entity-disjoint hybrid retrieval screens."""

import argparse
import hashlib
import json
import statistics
from collections import defaultdict
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def write_jsonl(path: Path, data: list[dict]) -> None:
    path.write_text("".join(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n"
                    for row in data))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    cases = rows(args.queries)
    results = rows(args.results)
    if len(cases) != 3300 or len(results) != len(cases) or [row["id"] for row in cases] != [row["id"] for row in results]:
        raise ValueError("General-memory search inputs differ")
    by_scope = defaultdict(list)
    for case, result in zip(cases, results):
        for field in ("split", "segment", "expected_ids", "expected_fact", "expected_condition",
                      "forbidden_ids", "label_source"):
            if result[field] != case[field]:
                raise ValueError(f"Mismatched field {field}: {case['id']}")
        source = case["expected_source"]
        if result["expected_source"] != ({**source, "locator": source.get("locator")} if source else None):
            raise ValueError(f"Mismatched source: {case['id']}")
        response = result["response"]
        if response["method"] != "fts5_semantic_rrf" or response["semantic_coverage"] != "complete":
            raise ValueError(f"Incomplete hybrid result: {case['id']}")
        ids = [item["memory_id"] for item in response["candidates"]]
        if len(ids) != len(set(ids)) or set(ids) & set(case["forbidden_ids"]):
            raise ValueError(f"Invalid candidates: {case['id']}")
        by_scope[(case["split"], case["segment"].split("/")[0])].append((case, result))
    args.output.mkdir(parents=True, exist_ok=True)
    split_hashes = {}
    for split in ("development", "final"):
        split_cases = [{**case, "expected_source":
                        ({**case["expected_source"], "locator": case["expected_source"].get("locator")}
                         if case["expected_source"] else None)}
                       for case in cases if case["split"] == split]
        split_results = [result for result in results if result["split"] == split]
        query_file = args.output / f"{split}-queries.jsonl"
        result_file = args.output / f"{split}-results.jsonl"
        seed_result_file = args.output / f"{split}-seed-results.jsonl"
        write_jsonl(query_file, split_cases)
        write_jsonl(result_file, split_results)
        write_jsonl(seed_result_file, [{**result, "hybrid_request": case["request"]}
                                       for case, result in zip(split_cases, split_results)])
        split_hashes[split] = {"queries": digest(query_file), "results": digest(result_file)}
    stats = {}
    for (split, scope), pairs in sorted(by_scope.items()):
        positives = [(case, result) for case, result in pairs if case["expected_ids"]]
        negatives = [(case, result) for case, result in pairs if not case["expected_ids"]]
        stats[f"{split}/{scope}"] = {
            "positive_cases": len(positives), "negative_cases": len(negatives),
            "positive_recall": {str(cap): sum(case["expected_ids"][0] in [item["memory_id"]
                                      for item in result["response"]["candidates"][:cap]]
                                      for case, result in positives) for cap in (1, 5, 10, 20)},
            "negative_with_candidates": sum(bool(result["response"]["candidates"])
                                            for _, result in negatives),
            "search_p50_ms": statistics.median(result["elapsed_ms"] for _, result in pairs),
        }
    summary = {"kind": "general_memory_v1_hybrid_retrieval", "synthetic_only": True,
               "queries_sha256": digest(args.queries), "results_sha256": digest(args.results),
               "split_sha256": split_hashes, "statistics": stats}
    (args.output / "retrieval-summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"cases": len(cases), "splits": split_hashes}, ensure_ascii=False))


if __name__ == "__main__":
    main()
