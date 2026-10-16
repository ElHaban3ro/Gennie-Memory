"""Measure keyword retrieval, Decisions selection and seed composition on frozen synthetic cases."""

import argparse
import json
import os
import subprocess
import time
from pathlib import Path

import general_memory_prompts
import probe_openrouter_decisions as probe


def exchange(process, op, request):
    process.stdin.write(json.dumps({"op": op, "request": request}, ensure_ascii=False) + "\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        raise RuntimeError(f"iROMEM session exited during {op}")
    result = json.loads(line)
    if result["status"] != "complete":
        raise RuntimeError(f"iROMEM {op}: {result['error']}")
    return result["response"]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--case-ids-from", type=Path, required=True)
    parser.add_argument("--database", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError("Use a new output file for each remote measurement")
    selected = json.loads(args.case_ids_from.read_text())["case_ids"]
    if len(selected) != 6 or len(set(selected)) != 6:
        raise ValueError("Expected six unique frozen cases")
    cases = {case["id"]: case for line in args.queries.read_text().splitlines()
             if (case := json.loads(line))["id"] in selected}
    if set(cases) != set(selected) or any(case["split"] != "development" for case in cases.values()):
        raise ValueError("Only the frozen synthetic development cases are accepted")
    key = probe.read_key()
    env = {**os.environ, "IROMEM_DATABASE_PATH": str(args.database.resolve())}
    process = subprocess.Popen([str(args.binary.resolve()), "session"], stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env)
    rows = []
    try:
        for case_id in selected:
            case = cases[case_id]
            search = {**case["request"]["search"], "query_mode": "keywords"}
            total_start = time.perf_counter()
            search_start = time.perf_counter()
            found = exchange(process, "search", search)
            search_ms = (time.perf_counter() - search_start) * 1000
            candidates = found["candidates"]
            decision_ms = 0.0
            chosen = "none"
            decision = None
            if candidates:
                question, mapping = general_memory_prompts.choice(candidates, "direct")
                decision_start = time.perf_counter()
                decision = probe.call(key, "openai/gpt-6-luna-decisions",
                                      {"user_prompt": search["query"], "scope": search["scope_id"]},
                                      question["criteria"], instructions=question["instructions"])
                decision_ms = (time.perf_counter() - decision_start) * 1000
                chosen = mapping[decision["chosen_id"]]
            presented = [{"memory_id": item["memory_id"], "revision": item["revision"]}
                         for item in candidates]
            selected_item = [item for item in presented if item["memory_id"] == chosen]
            compose_start = time.perf_counter()
            seed = exchange(process, "compose_seed", {
                "search": search, "hybrid": None, "presented": presented,
                "selected": selected_item, "classifier": "openai/gpt-6-luna-decisions",
                "classification_status": "complete",
            })
            compose_ms = (time.perf_counter() - compose_start) * 1000
            rows.append({
                "case_id": case_id, "expected_ids": case["expected_ids"],
                "candidate_ids": [item["memory_id"] for item in candidates],
                "chosen_id": chosen,
                "seed_ids": [item["memory_id"] for item in seed["memories"]],
                "correct": [item["memory_id"] for item in seed["memories"]] == case["expected_ids"],
                "search_ms": search_ms, "decision_ms": decision_ms,
                "compose_ms": compose_ms, "total_ms": (time.perf_counter() - total_start) * 1000,
                "served_model": decision["served_model"] if decision else None,
                "decision_input_tokens": decision["input_tokens"] if decision else 0,
                "decision_output_tokens": decision["output_tokens"] if decision else 0,
                "reported_cost_usd": decision["reported_cost_usd"] if decision else 0.0,
                "seed_text_utf8_bytes": len((seed["seed_text"] or "").encode("utf-8")),
                "seed_text_words": len((seed["seed_text"] or "").split()),
            })
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps({"kind": "keyword_direct_end_to_end_synthetic",
                                           "case_ids": selected, "rows": rows},
                                          ensure_ascii=False, indent=2) + "\n")
        print(json.dumps({"cases": len(rows), "correct": sum(row["correct"] for row in rows),
                          "reported_cost_usd": sum(row["reported_cost_usd"] or 0 for row in rows)},
                         ensure_ascii=False))
    finally:
        process.stdin.close()
        process.wait(timeout=10)
        process.stdout.close()
        process.stderr.close()


if __name__ == "__main__":
    main()
