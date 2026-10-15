#!/usr/bin/env python3
"""Verify published synthetic pilot files, labels, pairing and summaries."""

import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path


def read_jsonl(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def verify(bundle: Path) -> dict:
    manifest = json.loads((bundle / "manifest.json").read_text())
    if manifest["version"] != 1 or manifest["synthetic_only"] is not True or manifest["split"] != "development":
        raise ValueError("Invalid synthetic development manifest")
    benchmark = json.loads((bundle / manifest["benchmark_manifest"]).resolve().read_text())
    if benchmark["version"] != manifest["benchmark_version"] or benchmark["synthetic"] is not True:
        raise ValueError("Benchmark manifest mismatch")
    expected_files = {"corpus.json", "forms.json", "queries.jsonl"}
    if not expected_files <= set(benchmark["files"]):
        raise ValueError("Benchmark manifest is incomplete")
    benchmark_root = (bundle / manifest["benchmark_manifest"]).resolve().parent
    if hashlib.sha256((benchmark_root / "queries.jsonl").read_bytes()).hexdigest() != benchmark["files"]["queries.jsonl"]:
        raise ValueError("Versioned query corpus changed")
    canonical = {row["id"]: row for row in read_jsonl(benchmark_root / "queries.jsonl")}
    if len(canonical) != benchmark["query_count"]:
        raise ValueError("Benchmark IDs or count changed")

    listed = set(manifest["files_sha256"])
    referenced = {name for group in manifest["sample_groups"].values()
                  for name in [group["queries"], *group["reports"]]}
    if listed != referenced:
        raise ValueError("Published files differ from manifest")
    for name, expected_hash in manifest["files_sha256"].items():
        path = bundle / name
        payload = path.read_bytes()
        if hashlib.sha256(payload).hexdigest() != expected_hash:
            raise ValueError(f"Hash mismatch: {name}")
        if b"sk-or-v1-" in payload or b"IROMEM_OPENROUTER_API_KEY" in payload:
            raise ValueError(f"Potential secret: {name}")

    samples = {}
    reports = 0
    for group_name, group in manifest["sample_groups"].items():
        cases = read_jsonl(bundle / group["queries"])
        identities = [case["id"] for case in cases]
        if (len(identities) != 60 or len(set(identities)) != 60 or
            any(case != canonical.get(case["id"]) or case["split"] != "development" or
                case["label_source"] != "deterministic_fixture_rule" for case in cases)):
            raise ValueError(f"Invalid or changed cases in group {group_name}")
        if len(Counter(case["segment"] for case in cases)) != 12 or any(
            count != 5 for count in Counter(case["segment"] for case in cases).values()
        ):
            raise ValueError(f"Unbalanced sample: {group_name}")
        samples[group_name] = {case["id"]: case for case in cases}
        for report_name in group["reports"]:
            report = json.loads((bundle / report_name).read_text())
            rows = report["individual_results"]
            if (report["split"] != "development" or report["cases"] != len(rows) or
                report["positive"] + report["negative"] != len(rows) or
                report["quantile_method"] != "linear_interpolation"):
                raise ValueError(f"Invalid report summary: {report_name}")
            result_ids = [row["id"] for row in rows]
            if len(set(result_ids)) != len(result_ids) or not set(result_ids) <= set(samples[group_name]):
                raise ValueError(f"Unpaired results: {report_name}")
            if len(rows) != (2 if "two-cases" in report_name else 60):
                raise ValueError(f"Unexpected report size: {report_name}")
            for row in rows:
                case = samples[group_name][row["id"]]
                if row["positive"] != bool(case["expected_ids"]) or row["segment"] != case["segment"]:
                    raise ValueError(f"Label mismatch: {report_name}/{row['id']}")
                if row["forbidden_leak"] or row["chosen_id"] in case["forbidden_ids"]:
                    raise ValueError(f"Forbidden memory: {report_name}/{row['id']}")
                if row["exact"] != (row["chosen_id"] in case["expected_ids"] if case["expected_ids"]
                                    else row["chosen_id"] == "none"):
                    raise ValueError(f"Selection label mismatch: {report_name}/{row['id']}")
            if (report["positive_fact_fidelity"] != sum(row["fact_fidelity"] for row in rows if row["positive"]) or
                report["negative_empty"] != sum(row["fact_fidelity"] for row in rows if not row["positive"]) or
                report["forbidden_leaks"] != 0):
                raise ValueError(f"Aggregate mismatch: {report_name}")
            reports += 1
    if any(set(samples[left]) & set(samples[right]) for left, right in (("a", "b"), ("a", "c"), ("b", "c"))):
        raise ValueError("Sample query IDs overlap")
    older_entities = {case["request"].get("entity_id") for name in ("a", "b")
                      for case in samples[name].values() if case["request"].get("entity_id") is not None}
    new_entities = {case["request"].get("entity_id") for case in samples["c"].values()
                    if case["request"].get("entity_id") is not None}
    if older_entities & new_entities:
        raise ValueError("Sample C shares entities with prior samples")
    return {"valid": True, "samples": len(samples), "reports": reports,
            "queries_per_sample": 60, "entity_overlap_with_c": 0}


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.bundle.resolve())))


if __name__ == "__main__":
    main()
