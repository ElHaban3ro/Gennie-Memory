#!/usr/bin/env python3
"""Evaluate composed seed IDs and fact fidelity against synthetic labels."""

import argparse
import json
from pathlib import Path


def quantile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    return ordered[min(len(ordered) - 1, int((len(ordered) - 1) * fraction))] if ordered else 0.0


def read_jsonl(path: Path) -> dict[str, dict]:
    rows = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    result = {row["id"]: row for row in rows}
    if len(rows) != len(result):
        raise ValueError("Duplicate case IDs")
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("results", type=Path)
    parser.add_argument("--requests", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    requests, results = read_jsonl(args.requests), read_jsonl(args.results)
    if set(requests) != set(results):
        raise ValueError("Seed results are incomplete or contain extra cases")
    rows = []
    for identity, result in results.items():
        source = requests[identity]
        if any(result[key] != source[key] for key in
               ("split", "segment", "expected_ids", "forbidden_ids", "label_source")):
            raise ValueError(f"Label mismatch for {identity}")
        for field in ("expected_condition", "expected_source"):
            if result.get(field) != source.get(field):
                raise ValueError(f"Label mismatch for {identity}: {field}")
        response = result["response"]
        selected = [(m["memory_id"], m["revision"]) for m in response["memories"]]
        proposed = [(m["memory_id"], m["revision"]) for m in source["request"]["selected"]]
        if selected != proposed:
            raise ValueError(f"Selected revision changed for {identity}")
        expected = source["expected_ids"]
        chosen = [memory_id for memory_id, _ in selected]
        complete = response["status"] == "complete"
        selected_memory = response["memories"][0] if len(response["memories"]) == 1 else None
        condition_label = source.get("expected_condition")
        source_label = source.get("expected_source")
        rows.append({"id": identity, "positive": bool(expected), "complete": complete,
                     "correct_ids": complete and chosen == expected,
                     "fact_fidelity": complete and chosen == expected and
                     (source["expected_fact"] in (response["seed_text"] or "") if expected else response["seed_text"] is None),
                     "condition_labeled": bool(expected) and condition_label is not None,
                     "condition_fidelity": (complete and chosen == expected and
                                            selected_memory is not None and
                                            selected_memory.get("conditions") == condition_label and
                                            condition_label in (response["seed_text"] or ""))
                     if condition_label is not None else None,
                     "source_labeled": bool(expected) and source_label is not None,
                     "source_fidelity": (complete and chosen == expected and
                                         selected_memory is not None and
                                         selected_memory.get("source") == source_label and
                                         source_label["ref"] in (response["seed_text"] or ""))
                     if source_label is not None else None,
                     "forbidden_leak": bool(set(chosen) & set(source["forbidden_ids"])),
                     "elapsed_ms": result["elapsed_ms"]})
    positive = [row for row in rows if row["positive"]]
    negative = [row for row in rows if not row["positive"]]
    condition_labeled = [row for row in positive if row["condition_labeled"]]
    source_labeled = [row for row in positive if row["source_labeled"]]
    report = {"method": "seed_composition", "cases": len(rows),
              "positive": len(positive), "negative": len(negative),
              "complete_cases": sum(row["complete"] for row in rows),
              "exact_selection_accuracy": sum(row["correct_ids"] for row in rows) / len(rows),
              "positive_fact_fidelity": sum(row["fact_fidelity"] for row in positive) / len(positive) if positive else None,
              "positive_condition_fidelity": sum(row["condition_fidelity"] for row in condition_labeled) / len(condition_labeled) if condition_labeled else None,
              "positive_source_fidelity": sum(row["source_fidelity"] for row in source_labeled) / len(source_labeled) if source_labeled else None,
              "negative_empty_rate": sum(row["fact_fidelity"] for row in negative) / len(negative) if negative else None,
              "forbidden_leak_count": sum(row["forbidden_leak"] for row in rows),
              "composition_latency_ms": {"p50": quantile([row["elapsed_ms"] for row in rows], .5),
                                         "p95": quantile([row["elapsed_ms"] for row in rows], .95)}}
    encoded = json.dumps(report, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.write_text(encoded)
    else:
        print(encoded, end="")


if __name__ == "__main__":
    main()
