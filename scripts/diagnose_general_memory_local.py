#!/usr/bin/env python3
"""Isolate Laya choice behavior with oracle-present synthetic memories."""

import argparse
import hashlib
import json
import os
import statistics
import time
from collections import defaultdict
from pathlib import Path

from general_memory_prompts import VARIANTS, choice

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError("Choose a new output path for this immutable diagnostic")
    memories = json.loads(args.corpus.read_text())
    cases = [json.loads(line) for line in args.queries.read_text().splitlines() if line.strip()]
    by_scope = defaultdict(list)
    for memory in memories:
        by_scope[(memory["scope_kind"], memory["scope_id"])].append(memory)
    selected = []
    for kind in ("user", "project", "conversation", "agent"):
        cohort = [row for row in cases if row["split"] == "development" and
                  row["request"]["scope_kind"] == kind]
        positive = [row for row in cohort if row["expected_ids"]]
        negative = [row for row in cohort if not row["expected_ids"]]
        selected.extend((positive[0], positive[len(positive) // 2], negative[0]))
    model = Path(json.loads(args.soc_runtime.read_text())["model"]).resolve()
    import torch
    import laya
    from transformers.initialization import no_init_weights

    torch.set_num_threads(2)
    started = time.perf_counter()
    with no_init_weights():
        agent = laya.load(str(model), device="mps")
    load_seconds = time.perf_counter() - started
    if str(agent.device) != "mps":
        raise RuntimeError("MPS unavailable; local diagnostic would not match prior runs")
    warm, _ = choice([{"memory_id": "warm", "summary": "Synthetic memory"}], "direct")
    agent.predict({"user_prompt": "Warmup", "scope": "warmup"}, {"relevance": warm})
    rows = []
    for case in selected:
        request = case["request"]
        pool = by_scope[(request["scope_kind"], request["scope_id"])]
        expected = case["expected_ids"]
        ordered = sorted(pool, key=lambda item: hashlib.sha256(
            (case["id"] + item["id"]).encode()).hexdigest())
        if expected:
            ordered = [next(item for item in pool if item["id"] == expected[0])] + [
                item for item in ordered if item["id"] != expected[0]]
        for cap in (1, 5, 20):
            candidates = [{"memory_id": item["id"], "summary": item["summary"]}
                          for item in ordered[:cap]]
            for variant in VARIANTS:
                for include_none in (True, False) if expected and cap in (5, 20) else (True,):
                    question, mapping = choice(candidates, variant, include_none=include_none)
                    state = {"user_prompt": request["query"], "scope": request["scope_id"]}
                    started = time.perf_counter()
                    answer = agent.predict(state, {"relevance": question})
                    elapsed_ms = (time.perf_counter() - started) * 1000
                    decision = answer["answers"]["relevance"]
                    if decision["choice"] not in mapping:
                        raise ValueError("Local model selected an unknown label")
                    chosen = mapping[decision["choice"]]
                    rows.append({"case_id": case["id"], "scope_kind": request["scope_kind"],
                                 "label": "positive" if expected else "negative", "cap": cap,
                                 "variant": variant, "include_none": include_none,
                                 "expected_id": expected[0] if expected else None,
                                 "chosen_id": chosen, "correct": chosen in expected if expected else chosen == "none",
                                 "none_probability": decision["probabilities"].get("none"),
                                 "chosen_probability": decision["probabilities"][decision["choice"]],
                                 "elapsed_ms": elapsed_ms,
                                 "input_tokens": answer["usage"]["input_tokens"]})
    summary = {}
    for cap in (1, 5, 20):
        for variant in VARIANTS:
            for include_none in (True, False):
                group = [row for row in rows if (row["cap"], row["variant"], row["include_none"]) ==
                         (cap, variant, include_none)]
                if group:
                    summary[f"{cap}/{variant}/none={include_none}"] = {
                        "positive_exact": sum(row["correct"] for row in group if row["label"] == "positive"),
                        "positive_cases": sum(row["label"] == "positive" for row in group),
                        "negative_empty": sum(row["correct"] for row in group if row["label"] == "negative"),
                        "negative_cases": sum(row["label"] == "negative" for row in group),
                        "chosen_none": sum(row["chosen_id"] == "none" for row in group),
                        "mean_none_probability": statistics.mean(row["none_probability"] for row in group
                                                                 if row["none_probability"] is not None)
                                                 if include_none else None,
                    }
    report = {"kind": "general_memory_laya_oracle_diagnostic", "synthetic_only": True,
              "model_sha256": digest(model / "model.safetensors"),
              "tokenizer_sha256": digest(model / "tokenizer/tokenizer.json"),
              "corpus_sha256": digest(args.corpus), "queries_sha256": digest(args.queries),
              "device": str(agent.device), "laya_version": laya.__version__,
              "load_seconds": load_seconds, "summary": summary, "results": rows}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"device": report["device"], "load_seconds": load_seconds,
                      "summary": summary}, ensure_ascii=False))


if __name__ == "__main__":
    main()
