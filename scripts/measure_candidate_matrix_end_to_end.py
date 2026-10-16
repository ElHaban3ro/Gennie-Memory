#!/usr/bin/env python3
"""Measure warm, single-query hybrid-to-seed latency on synthetic cases."""

import argparse
import hashlib
import json
import os
import subprocess
import time
from pathlib import Path

import classifier_prompts
import probe_openrouter_decisions as probe

os.environ.update(HF_HUB_OFFLINE="1", TRANSFORMERS_OFFLINE="1", USE_TF="0",
                  TOKENIZERS_PARALLELISM="false")


def quantile(values: list[float], fraction: float) -> float:
    values = sorted(values)
    index = (len(values) - 1) * fraction
    lower = int(index)
    upper = min(lower + 1, len(values) - 1)
    return values[lower] + (values[upper] - values[lower]) * (index - lower)


def exchange(process: subprocess.Popen, op: str, request: dict) -> dict:
    process.stdin.write(json.dumps({"op": op, "request": request},
                                   ensure_ascii=False, separators=(",", ":")) + "\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        raise RuntimeError(f"iROMEM session exited during {op}")
    result = json.loads(line)
    if result["status"] != "complete":
        raise RuntimeError(f"iROMEM {op}: {result['error']}")
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--database", type=Path, required=True)
    parser.add_argument("--binary", type=Path, default=Path("target/debug/iromem"))
    parser.add_argument("--journal", type=Path, required=True)
    parser.add_argument("--summary", type=Path, required=True)
    parser.add_argument("--model", choices=probe.MODELS,
                        default="openai/gpt-6-luna-decisions")
    parser.add_argument("--soc-runtime", type=Path,
                        default=Path.home() / "Library/Application Support/soc/decisions/runtime.json")
    args = parser.parse_args()
    source = args.queries.read_bytes()
    cases = [json.loads(line) for line in source.splitlines() if line.strip()]
    if not cases or any(case["label_source"] != "deterministic_fixture_rule"
                        for case in cases):
        raise ValueError("Only frozen synthetic cases are accepted")
    if args.journal.exists():
        raise ValueError("Choose a new journal; end-to-end runs are immutable")
    if not args.binary.is_file() or not args.database.is_file():
        raise ValueError("Binary or database missing")
    model = Path(json.loads(args.soc_runtime.read_text())["model"]).resolve()
    import numpy as np
    import torch
    import laya
    from laya.shortlist import embed_fn_from_agent
    from transformers.initialization import no_init_weights

    torch.set_num_threads(2)
    started = time.perf_counter()
    with no_init_weights():
        agent = laya.load(str(model), device="mps")
    load_seconds = time.perf_counter() - started
    if str(agent.device) != "mps":
        raise ValueError("Expected MPS")
    embed = embed_fn_from_agent(agent, max_length=512, batch_size=1)
    embed(["Synthetic warmup query"])
    key = probe.read_key()
    environment = {**os.environ, "IROMEM_DATABASE_PATH": str(args.database.resolve())}
    process = subprocess.Popen([str(args.binary.resolve()), "session"],
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, env=environment)
    args.journal.parent.mkdir(parents=True, exist_ok=True)
    manifest = {"phase": "manifest", "queries_sha256": hashlib.sha256(source).hexdigest(),
                "model": args.model, "prompt_variant": "plain", "max_candidates": 20,
                "local_model_version": laya.__version__, "local_load_seconds": load_seconds}
    with args.journal.open("w") as stream:
        stream.write(json.dumps(manifest, ensure_ascii=False) + "\n")
        stream.flush()
        os.fsync(stream.fileno())
    completed = []
    try:
        for case in cases:
            query = case["request"]["search"]["query"]
            request = dict(case["request"])
            total_started = time.perf_counter()
            embed_started = time.perf_counter()
            vector = embed([query])
            embedding_ms = (time.perf_counter() - embed_started) * 1000
            if vector.shape[0] != 1 or not np.isfinite(vector).all():
                raise ValueError("Invalid query vector")
            request["query_vector"] = vector[0].astype(float).tolist()
            retrieval_started = time.perf_counter()
            found = exchange(process, "hybrid_search", request)
            retrieval_ms = (time.perf_counter() - retrieval_started) * 1000
            candidates = found["response"]["candidates"][:20]
            decision_ms = 0.0
            cost = 0.0
            chosen = "none"
            if candidates:
                question, mapping = classifier_prompts.choice(candidates, "plain")
                decision_started = time.perf_counter()
                answer = probe.call(key, args.model,
                                    {"user_prompt": query,
                                     "scope": case["request"]["search"]["scope_id"]},
                                    question["criteria"], instructions=question["instructions"])
                decision_ms = (time.perf_counter() - decision_started) * 1000
                chosen = mapping[answer["chosen_id"]]
                cost = answer["reported_cost_usd"]
            by_id = {row["memory_id"]: row for row in candidates}
            presented = [{"memory_id": row["memory_id"], "revision": row["revision"]}
                         for row in candidates]
            selected = ([{"memory_id": chosen, "revision": by_id[chosen]["revision"]}]
                        if chosen != "none" else [])
            composition_started = time.perf_counter()
            composed = exchange(process, "compose_seed", {
                "search": None, "hybrid": request, "presented": presented,
                "selected": selected, "classifier": args.model,
                "classification_status": "complete"})
            composition_ms = (time.perf_counter() - composition_started) * 1000
            total_ms = (time.perf_counter() - total_started) * 1000
            memories = composed["response"]["memories"]
            exact = [memory["memory_id"] for memory in memories] == case["expected_ids"]
            row = {"phase": "case", "case_id": case["id"], "split": case["split"],
                   "expected_ids": case["expected_ids"], "chosen_id": chosen,
                   "seed_ids": [memory["memory_id"] for memory in memories],
                   "correct": exact, "candidate_covered": bool(set(case["expected_ids"]) &
                                                       set(by_id)),
                   "embedding_ms": embedding_ms, "retrieval_ms": retrieval_ms,
                   "decision_ms": decision_ms, "composition_ms": composition_ms,
                   "total_ms": total_ms, "reported_cost_usd": cost,
                   "source_and_condition_faithful": bool(exact and memories and
                      memories[0]["conditions"] == case.get("expected_condition") and
                      memories[0]["source"] == case.get("expected_source") and
                      case["expected_fact"] in (composed["response"]["seed_text"] or ""))}
            with args.journal.open("a") as stream:
                stream.write(json.dumps(row, ensure_ascii=False) + "\n")
                stream.flush()
                os.fsync(stream.fileno())
            completed.append(row)
    finally:
        process.stdin.close()
        process.wait(timeout=10)
        process.stdout.close()
        process.stderr.close()
    if len(completed) != len(cases):
        raise RuntimeError("End-to-end pilot incomplete")
    positive = [row for row in completed if row["expected_ids"]]
    negative = [row for row in completed if not row["expected_ids"]]
    summary = {"kind": "candidate_matrix_end_to_end_pilot", "synthetic_only": True,
               "queries_sha256": manifest["queries_sha256"], "model": args.model,
               "cases": len(completed), "positive_exact": sum(row["correct"] for row in positive),
               "positive_cases": len(positive), "negative_empty": sum(row["correct"] for row in negative),
               "negative_cases": len(negative),
               "faithful_positive": sum(row["source_and_condition_faithful"] for row in positive),
               "load_seconds": load_seconds,
               "timing_ms": {field: {"p50": quantile([row[field] for row in completed], .5),
                                     "p95": quantile([row[field] for row in completed], .95)}
                             for field in ("embedding_ms", "retrieval_ms", "decision_ms",
                                           "composition_ms", "total_ms")},
               "reported_cost_usd": sum(row["reported_cost_usd"] for row in completed)}
    args.summary.parent.mkdir(parents=True, exist_ok=True)
    args.summary.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"cases": summary["cases"], "positive_exact": summary["positive_exact"],
                      "negative_empty": summary["negative_empty"],
                      "total_p50_ms": summary["timing_ms"]["total_ms"]["p50"],
                      "reported_cost_usd": summary["reported_cost_usd"]}))


if __name__ == "__main__":
    main()
