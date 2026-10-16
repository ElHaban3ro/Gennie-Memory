#!/usr/bin/env python3
"""Compare three typed-choice prompts on frozen synthetic candidate matrix screens."""

import argparse
import hashlib
import json
import math
import os
import time
from pathlib import Path

from classifier_prompts import CHOICE_VARIANTS, choice

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


def sha256(path: Path) -> str:
    value = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            value.update(chunk)
    return value.hexdigest()


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def quantile(values: list[float], fraction: float) -> float:
    values = sorted(values)
    if not values:
        return 0.0
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def append(path: Path, row: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as stream:
        stream.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")
        stream.flush()
        os.fsync(stream.fileno())


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--journal", type=Path, required=True)
    parser.add_argument("--summary", type=Path, required=True)
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    parser.add_argument("--max-seconds", type=float, default=1800)
    args = parser.parse_args()
    if not math.isfinite(args.max_seconds) or args.max_seconds <= 0:
        parser.error("max-seconds must be positive")
    model = Path(json.loads(args.soc_runtime.read_text())["model"]).resolve()
    files = (model / "model.safetensors", model / "tokenizer/tokenizer.json")
    if not all(path.is_file() for path in files):
        raise ValueError("Laya snapshot missing")
    inputs = {}
    for size in (100, 200, 2000):
        for scope in ("scoped", "unscoped"):
            root = args.root / str(size)
            query_path = root / f"screen-{scope}.jsonl"
            result_path = root / f"screen-{scope}-results.jsonl"
            queries = {row["id"]: row for row in rows(query_path)}
            results = rows(result_path)
            if len(queries) != 28 or len(results) != 28 or set(queries) != {row["id"] for row in results}:
                raise ValueError("Screen input mismatch")
            inputs[f"{size}/{scope}"] = (queries, results, sha256(query_path), sha256(result_path))
    manifest = {"version": 1, "kind": "laya_candidate_matrix_choice",
                "model_sha256": sha256(files[0]), "tokenizer_sha256": sha256(files[1]),
                "prompt_sha256": sha256(Path(__file__).with_name("classifier_prompts.py")),
                "runner_sha256": sha256(Path(__file__)),
                "inputs": {key: {"queries": value[2], "results": value[3]}
                           for key, value in inputs.items()},
                "caps": [5, 10, 15, 20], "variants": list(CHOICE_VARIANTS)}
    if args.journal.exists():
        events = rows(args.journal)
        if not events or events[0] != {"phase": "manifest", "value": manifest}:
            raise ValueError("Journal manifest differs from current inputs")
    else:
        append(args.journal, {"phase": "manifest", "value": manifest})
        events = [{"phase": "manifest", "value": manifest}]
    completed = {}
    for event in events[1:]:
        if event.get("phase") != "decision":
            raise ValueError("Invalid journal event")
        row = event["value"]
        key = (row["scenario"], row["cap"], row["variant"], row["case_id"])
        if key in completed:
            raise ValueError("Duplicate decision event")
        completed[key] = row
    import laya
    from transformers.initialization import no_init_weights

    started = time.monotonic()
    with no_init_weights():
        agent = laya.load(str(model), device="mps")
    load_seconds = time.monotonic() - started
    if str(agent.device) != "mps":
        raise ValueError("Expected MPS")
    warmup_started = time.monotonic()
    warm_question, _ = choice([{"memory_id": f"warm_{index}",
                                "summary": f"Synthetic memory {index}", "conditions": None}
                               for index in range(20)], "plain")
    agent.predict({"user_prompt": "Synthetic warmup question", "scope": "warmup"},
                  {"relevance": warm_question})
    warmup_ms = (time.monotonic() - warmup_started) * 1000
    decision_started = time.monotonic()
    for scenario, (queries, results, _query_hash, _result_hash) in inputs.items():
        for cap in (5, 10, 15, 20):
            for variant in CHOICE_VARIANTS:
                for result in results:
                    key = (scenario, cap, variant, result["id"])
                    if key in completed:
                        continue
                    if time.monotonic() - decision_started >= args.max_seconds:
                        raise RuntimeError("Local decision time cap reached; journal is resumable")
                    case = queries[result["id"]]
                    candidates = result["response"]["candidates"][:cap]
                    if candidates:
                        question, mapping = choice(candidates, variant)
                        state = {"user_prompt": case["request"]["search"]["query"],
                                 "scope": case["request"]["search"]["scope_id"]}
                        call_started = time.monotonic()
                        answer = agent.predict(state, {"relevance": question})
                        elapsed_ms = (time.monotonic() - call_started) * 1000
                        decision = answer["answers"]["relevance"]
                        probabilities = decision["probabilities"]
                        if (decision["choice"] not in mapping or set(probabilities) != set(mapping) or
                                any(isinstance(value, bool) or not isinstance(value, (int, float)) or
                                    not math.isfinite(value) or not 0 <= value <= 1
                                    for value in probabilities.values()) or
                                abs(sum(probabilities.values()) - 1) > .02):
                            raise ValueError("Invalid Laya decision")
                        chosen = mapping[decision["choice"]]
                        tokens = answer["usage"]["input_tokens"]
                        status = "complete"
                    else:
                        chosen, elapsed_ms, tokens, status = "none", 0.0, 0, "skipped_empty"
                    row = {"scenario": scenario, "cap": cap, "variant": variant,
                           "case_id": result["id"], "split": case["split"],
                           "segment": case["segment"], "status": status,
                           "chosen_id": chosen, "candidate_ids": [item["memory_id"] for item in candidates],
                           "expected_ids": case["expected_ids"],
                           "correct": chosen in case["expected_ids"] if case["expected_ids"] else chosen == "none",
                           "elapsed_ms": elapsed_ms, "input_tokens": tokens}
                    append(args.journal, {"phase": "decision", "value": row})
                    completed[key] = row
    aggregates = {}
    for scenario in inputs:
        for cap in (5, 10, 15, 20):
            for variant in CHOICE_VARIANTS:
                group = [row for row in completed.values() if row["scenario"] == scenario and
                         row["cap"] == cap and row["variant"] == variant]
                positive = [row for row in group if row["expected_ids"]]
                negative = [row for row in group if not row["expected_ids"]]
                if len(group) != 28:
                    raise ValueError("Incomplete screen group")
                timings = [row["elapsed_ms"] for row in group if row["status"] == "complete"]
                aggregates[f"{scenario}/{cap}/{variant}"] = {
                    "positive_exact": sum(row["correct"] for row in positive),
                    "positive_cases": len(positive),
                    "negative_empty": sum(row["correct"] for row in negative),
                    "negative_cases": len(negative),
                    "decision_ms": {"p50": quantile(timings, .5), "p95": quantile(timings, .95)},
                    "input_tokens": sum(row["input_tokens"] or 0 for row in group),
                }
    args.summary.parent.mkdir(parents=True, exist_ok=True)
    args.summary.write_text(json.dumps({"manifest": manifest, "device": str(agent.device),
                                        "model_version": laya.__version__,
                                        "load_seconds": load_seconds, "warmup_ms": warmup_ms,
                                        "decisions": len(completed), "groups": aggregates},
                                       ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"decisions": len(completed), "groups": len(aggregates),
                      "load_seconds": load_seconds, "warmup_ms": warmup_ms}))


if __name__ == "__main__":
    main()
