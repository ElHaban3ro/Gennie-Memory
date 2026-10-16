#!/usr/bin/env python3
"""Run resumable Laya choices against frozen general-memory hybrid candidates."""

import argparse
import hashlib
import json
import math
import os
import time
from pathlib import Path

from general_memory_prompts import VARIANTS, choice

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def append(path: Path, event: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as stream:
        stream.write(json.dumps(event, ensure_ascii=False, separators=(",", ":")) + "\n")
        stream.flush()
        os.fsync(stream.fileno())


def percentile(values: list[float], fraction: float) -> float:
    ordered = sorted(values)
    index = (len(ordered) - 1) * fraction
    low = int(index)
    high = min(low + 1, len(ordered) - 1)
    return ordered[low] + (ordered[high] - ordered[low]) * (index - low)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--journal", type=Path, required=True)
    parser.add_argument("--summary", type=Path, required=True)
    parser.add_argument("--cap", type=int, action="append", required=True)
    parser.add_argument("--variant", choices=VARIANTS, action="append", required=True)
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    args = parser.parse_args()
    if len(set(args.cap)) != len(args.cap) or any(not 1 <= cap <= 20 for cap in args.cap) or len(set(args.variant)) != len(args.variant):
        parser.error("Caps must be distinct 1..20 and variants distinct")
    cases = rows(args.queries)
    results = rows(args.results)
    if not cases or [case["id"] for case in cases] != [row["id"] for row in results]:
        raise ValueError("Input cases and retrieval results differ")
    model = Path(json.loads(args.soc_runtime.read_text())["model"]).resolve()
    manifest = {"kind": "general_memory_laya_choice", "synthetic_only": True,
                "queries_sha256": sha256(args.queries), "results_sha256": sha256(args.results),
                "model_sha256": sha256(model / "model.safetensors"),
                "tokenizer_sha256": sha256(model / "tokenizer/tokenizer.json"),
                "prompt_sha256": sha256(Path(__file__).with_name("general_memory_prompts.py")),
                "runner_sha256": sha256(Path(__file__)),
                "caps": args.cap, "variants": args.variant}
    if args.journal.exists():
        events = rows(args.journal)
        if not events or events[0] != {"phase": "manifest", "value": manifest}:
            raise ValueError("Local journal manifest differs from inputs")
    else:
        append(args.journal, {"phase": "manifest", "value": manifest})
        events = [{"phase": "manifest", "value": manifest}]
    completed = {}
    for event in events[1:]:
        if event["phase"] != "decision":
            raise ValueError("Unexpected journal event")
        row = event["value"]
        key = (row["case_id"], row["cap"], row["variant"])
        if key in completed:
            raise ValueError("Duplicate local decision")
        completed[key] = row
    import torch
    import laya
    from transformers.initialization import no_init_weights

    torch.set_num_threads(2)
    started = time.perf_counter()
    with no_init_weights():
        agent = laya.load(str(model), device="mps")
    load_seconds = time.perf_counter() - started
    if str(agent.device) != "mps":
        raise RuntimeError("MPS unavailable")
    warm, _ = choice([{"memory_id": "warm", "summary": "Synthetic memory"}], args.variant[0])
    agent.predict({"user_prompt": "Warmup", "scope": "warmup"}, {"relevance": warm})
    for case, result in zip(cases, results):
        for cap in args.cap:
            candidates = result["response"]["candidates"][:cap]
            for variant in args.variant:
                key = (case["id"], cap, variant)
                if key in completed:
                    continue
                if candidates:
                    question, mapping = choice(candidates, variant)
                    state = {"user_prompt": case["request"]["search"]["query"],
                             "scope": case["request"]["search"]["scope_id"]}
                    started = time.perf_counter()
                    answer = agent.predict(state, {"relevance": question})
                    elapsed_ms = (time.perf_counter() - started) * 1000
                    decision = answer["answers"]["relevance"]
                    probabilities = decision["probabilities"]
                    if (decision["choice"] not in mapping or set(probabilities) != set(mapping) or
                            any(isinstance(value, bool) or not isinstance(value, (int, float)) or
                                not math.isfinite(value) or not 0 <= value <= 1
                                for value in probabilities.values()) or
                            abs(sum(probabilities.values()) - 1) > .02):
                        raise ValueError(f"Invalid Laya decision: {case['id']}")
                    chosen = mapping[decision["choice"]]
                    none_probability = probabilities["none"]
                    token_count = answer["usage"]["input_tokens"]
                else:
                    chosen, elapsed_ms, none_probability, token_count = "none", 0.0, 1.0, 0
                expected = case["expected_ids"]
                row = {"case_id": case["id"], "split": case["split"],
                       "scope_kind": case["request"]["search"]["scope_kind"],
                       "segment": case["segment"], "cap": cap, "variant": variant,
                       "candidate_ids": [item["memory_id"] for item in candidates],
                       "expected_ids": expected, "chosen_id": chosen,
                       "correct": chosen in expected if expected else chosen == "none",
                       "none_probability": none_probability, "elapsed_ms": elapsed_ms,
                       "input_tokens": token_count}
                append(args.journal, {"phase": "decision", "value": row})
                completed[key] = row
    groups = {}
    for cap in args.cap:
        for variant in args.variant:
            group = [row for row in completed.values() if row["cap"] == cap and row["variant"] == variant]
            if len(group) != len(cases):
                raise ValueError("Incomplete local group")
            positive = [row for row in group if row["expected_ids"]]
            negative = [row for row in group if not row["expected_ids"]]
            times = [row["elapsed_ms"] for row in group if row["elapsed_ms"] > 0]
            groups[f"{cap}/{variant}"] = {
                "positive_exact": sum(row["correct"] for row in positive), "positive_cases": len(positive),
                "negative_empty": sum(row["correct"] for row in negative), "negative_cases": len(negative),
                "positive_candidate_coverage": sum(row["expected_ids"][0] in row["candidate_ids"]
                                                   for row in positive),
                "decision_p50_ms": percentile(times, .5), "decision_p95_ms": percentile(times, .95),
                "input_tokens": sum(row["input_tokens"] for row in group),
            }
    report = {"manifest": manifest, "device": str(agent.device), "laya_version": laya.__version__,
              "load_seconds": load_seconds, "decisions": len(completed), "groups": groups}
    args.summary.parent.mkdir(parents=True, exist_ok=True)
    args.summary.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"decisions": len(completed), "groups": groups}, ensure_ascii=False))


if __name__ == "__main__":
    main()
