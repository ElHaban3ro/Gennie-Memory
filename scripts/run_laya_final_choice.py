#!/usr/bin/env python3
"""Evaluate local Laya on the frozen final scoped top-20 plain candidates."""

import argparse
import hashlib
import json
import math
import os
import time
from pathlib import Path

from classifier_prompts import choice

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


def rows(path: Path) -> list[dict]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def quantile(values: list[float], fraction: float) -> float:
    values = sorted(values)
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--results", type=Path, required=True)
    parser.add_argument("--journal", type=Path, required=True)
    parser.add_argument("--summary", type=Path, required=True)
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    args = parser.parse_args()
    queries = rows(args.queries)
    results = rows(args.results)
    if (len(queries) != 480 or len(results) != 480 or
            [row["id"] for row in queries] != [row["id"] for row in results] or
            any(row["split"] != "final" for row in queries)):
        raise ValueError("Final inputs differ")
    if args.journal.exists():
        raise ValueError("Choose a new journal; final local runs are immutable")
    model = Path(json.loads(args.soc_runtime.read_text())["model"]).resolve()
    import laya
    from transformers.initialization import no_init_weights

    started = time.perf_counter()
    with no_init_weights():
        agent = laya.load(str(model), device="mps")
    load_seconds = time.perf_counter() - started
    if str(agent.device) != "mps":
        raise ValueError("Expected MPS")
    warmup, _ = choice([{"memory_id": f"warm_{index}",
                         "summary": f"Synthetic memory {index}", "conditions": None}
                        for index in range(20)], "plain")
    agent.predict({"user_prompt": "Synthetic warmup question", "scope": "warmup"},
                  {"relevance": warmup})
    manifest = {"phase": "manifest", "kind": "laya_final_choice",
                "queries_sha256": sha256(args.queries),
                "results_sha256": sha256(args.results),
                "prompt_sha256": sha256(Path(__file__).with_name("classifier_prompts.py")),
                "runner_sha256": sha256(Path(__file__)),
                "model_sha256": sha256(model / "model.safetensors"),
                "tokenizer_sha256": sha256(model / "tokenizer/tokenizer.json"),
                "model_version": laya.__version__, "device": str(agent.device),
                "load_seconds": load_seconds, "max_candidates": 20,
                "prompt_variant": "plain"}
    args.journal.parent.mkdir(parents=True, exist_ok=True)
    with args.journal.open("w") as stream:
        stream.write(json.dumps(manifest, ensure_ascii=False) + "\n")
        stream.flush()
        os.fsync(stream.fileno())
    completed = []
    for case, result in zip(queries, results):
        candidates = result["response"]["candidates"][:20]
        chosen = "none"
        elapsed_ms = 0.0
        tokens = 0
        if candidates:
            question, mapping = choice(candidates, "plain")
            state = {"user_prompt": case["request"]["search"]["query"],
                     "scope": case["request"]["search"]["scope_id"]}
            started = time.perf_counter()
            answer = agent.predict(state, {"relevance": question})
            elapsed_ms = (time.perf_counter() - started) * 1000
            decision = answer["answers"]["relevance"]
            probabilities = decision["probabilities"]
            if (decision["choice"] not in mapping or set(probabilities) != set(mapping) or
                    any(not isinstance(value, (int, float)) or not math.isfinite(value)
                        for value in probabilities.values()) or
                    abs(sum(probabilities.values()) - 1) > .02):
                raise ValueError(f"Invalid Laya decision: {case['id']}")
            chosen = mapping[decision["choice"]]
            tokens = answer["usage"]["input_tokens"]
        row = {"phase": "decision", "case_id": case["id"],
               "expected_ids": case["expected_ids"], "chosen_id": chosen,
               "correct": chosen in case["expected_ids"] if case["expected_ids"]
                          else chosen == "none", "elapsed_ms": elapsed_ms,
               "input_tokens": tokens}
        with args.journal.open("a") as stream:
            stream.write(json.dumps(row, ensure_ascii=False) + "\n")
            stream.flush()
            os.fsync(stream.fileno())
        completed.append(row)
    positive = [row for row in completed if row["expected_ids"]]
    negative = [row for row in completed if not row["expected_ids"]]
    times = [row["elapsed_ms"] for row in completed if row["elapsed_ms"] > 0]
    summary = {"kind": "laya_final_choice", "synthetic_only": True,
               "manifest": manifest, "positive_exact": sum(row["correct"] for row in positive),
               "positive_cases": len(positive),
               "negative_empty": sum(row["correct"] for row in negative),
               "negative_cases": len(negative), "input_tokens": sum(row["input_tokens"] for row in completed),
               "decision_ms": {"p50": quantile(times, .5), "p95": quantile(times, .95)}}
    args.summary.parent.mkdir(parents=True, exist_ok=True)
    args.summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({key: summary[key] for key in ("positive_exact", "positive_cases",
                                                   "negative_empty", "negative_cases",
                                                   "decision_ms")}))


if __name__ == "__main__":
    main()
