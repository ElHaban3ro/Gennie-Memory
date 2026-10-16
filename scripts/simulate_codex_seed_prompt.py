"""Measure Codex with frozen synthetic iROMEM seeds in its initial prompt.

This is an isolated prompt simulation. It does not install a hook or change user
Codex configuration. The previously observed iROMEM selections are reused.
"""

import argparse
import json
import os
import queue
import subprocess
import tempfile
import threading
import time
from pathlib import Path


def exchange(process, op, request):
    process.stdin.write(json.dumps({"op": op, "request": request}, ensure_ascii=False) + "\n")
    process.stdin.flush()
    line = process.stdout.readline()
    if not line:
        raise RuntimeError("iROMEM session exited")
    result = json.loads(line)
    if result["status"] != "complete":
        raise RuntimeError(result["error"])
    return result["response"]


def run_codex(binary, prompt, directory):
    command = [str(binary), "exec", "--json", "--ephemeral", "--ignore-user-config",
               "--ignore-rules", "--skip-git-repo-check", "-s", "read-only",
               "-m", "gpt-6.1-sol", "-C", str(directory), "-"]
    started = time.monotonic()
    process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True, bufsize=1)
    process.stdin.write(prompt)
    process.stdin.close()
    lines = queue.Queue()

    def reader():
        for line in process.stdout:
            lines.put(((time.monotonic() - started) * 1000, line))
        lines.put(None)

    thread = threading.Thread(target=reader, daemon=True)
    thread.start()
    events = []
    final_answer_ms = None
    final_answer = None
    deadline = time.monotonic() + 90
    while True:
        try:
            item = lines.get(timeout=max(0.1, deadline - time.monotonic()))
        except queue.Empty:
            process.kill()
            raise TimeoutError("Codex exceeded 90 seconds")
        if item is None:
            break
        elapsed_ms, line = item
        event = json.loads(line)
        events.append({"elapsed_ms": elapsed_ms, "event": event})
        if event.get("type") == "item.completed" and event.get("item", {}).get("type") == "agent_message":
            final_answer_ms = elapsed_ms
            final_answer = event["item"].get("text")
    process.wait(timeout=5)
    stderr = process.stderr.read()
    process.stdout.close()
    process.stderr.close()
    usage = next((entry["event"]["usage"] for entry in reversed(events)
                  if entry["event"].get("type") == "turn.completed"), None)
    tool_calls = [entry["event"]["item"].get("type") for entry in events
                  if entry["event"].get("type") == "item.completed"
                  and entry["event"].get("item", {}).get("type") in
                  ("mcp_tool_call", "command_execution", "web_search")]
    if process.returncode or final_answer is None or usage is None or tool_calls:
        raise RuntimeError(f"Codex simulation failed: exit={process.returncode}, "
                           f"answer={final_answer is not None}, usage={usage is not None}, "
                           f"tool_calls={tool_calls}, stderr={stderr[:300]}")
    return final_answer_ms, final_answer, usage, events


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--queries", type=Path, required=True)
    parser.add_argument("--prior-direct", type=Path, required=True)
    parser.add_argument("--database", type=Path, required=True)
    parser.add_argument("--iromem", type=Path, required=True)
    parser.add_argument("--codex", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--mask-seed", action="store_true",
                        help="Use the same questions and prompt wrapper without injecting the seed")
    args = parser.parse_args()
    if args.output_dir.exists():
        raise ValueError("Use a fresh output directory")
    prior = json.loads(args.prior_direct.read_text())
    cases = {case["id"]: case for line in args.queries.read_text().splitlines()
             if (case := json.loads(line))["id"] in prior["case_ids"]}
    if set(cases) != set(prior["case_ids"]):
        raise ValueError("Frozen cases do not match")
    selections = {row["case_id"]: row["selected_ids"] for row in prior["rows"]}
    env = {**os.environ, "IROMEM_DATABASE_PATH": str(args.database.resolve())}
    iromem = subprocess.Popen([str(args.iromem.resolve()), "session"],
                              stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, text=True, env=env)
    args.output_dir.mkdir(parents=True)
    rows = []
    try:
        with tempfile.TemporaryDirectory(prefix="iromem-codex-seed-") as temp:
            for case_id in prior["case_ids"]:
                case = cases[case_id]
                request = case["request"]
                found = exchange(iromem, "hybrid_search", request)
                presented = [{"memory_id": c["memory_id"], "revision": c["revision"]}
                             for c in found["candidates"]]
                selected = [c for c in presented if c["memory_id"] in selections[case_id]]
                if len(selected) != len(selections[case_id]):
                    raise ValueError(f"Frozen selection missing for {case_id}")
                seed = exchange(iromem, "compose_seed", {
                    "search": None, "hybrid": request, "presented": presented,
                    "selected": selected, "classifier": "frozen_prior_decisions",
                    "classification_status": "complete",
                })
                seed_text = seed["seed_text"] or ""
                if args.mask_seed:
                    seed_text = ""
                prompt = (
                    "Pregunta del usuario: " + request["search"]["query"] + "\n\n"
                    "Memoria semilla preparada antes de este turno:\n"
                    + (seed_text if seed_text else "No se encontró una memoria pertinente.")
                    + "\n\nResponde brevemente usando solo los hechos de la semilla. "
                    "Si la semilla no contiene el dato, dilo. No uses herramientas ni archivos."
                )
                answer_ms, answer, usage, events = run_codex(args.codex.resolve(), prompt, temp)
                (args.output_dir / f"{case_id}-events.jsonl").write_text(
                    "".join(json.dumps(event, ensure_ascii=False) + "\n" for event in events)
                )
                rows.append({
                    "case_id": case_id, "expected_ids": case["expected_ids"],
                    "selected_ids": selections[case_id],
                    "seed_text_utf8_bytes": len(seed_text.encode("utf-8")),
                    "seed_text_words": len(seed_text.split()),
                    "prompt_utf8_bytes": len(prompt.encode("utf-8")),
                    "final_answer_ms": answer_ms, "final_answer": answer, "usage": usage,
                })
                print(json.dumps({"case_id": case_id, "final_answer_ms": answer_ms,
                                  "input_tokens": usage.get("input_tokens")}), flush=True)
    finally:
        iromem.stdin.close()
        iromem.wait(timeout=10)
        iromem.stdout.close()
        iromem.stderr.close()
    (args.output_dir / "summary.json").write_text(json.dumps({
        "kind": "codex_initial_seed_prompt_simulation",
        "seed_injected": not args.mask_seed,
        "codex_model": "gpt-6.1-sol", "case_ids": prior["case_ids"],
        "source": str(args.prior_direct), "rows": rows,
    }, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
