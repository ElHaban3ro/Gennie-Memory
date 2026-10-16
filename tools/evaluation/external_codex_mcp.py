"""Run the frozen six-case Codex MCP pilot for an external memory server.

The server configuration is explicit and the Codex process is ephemeral. This
measures the whole Codex turn with MCP startup, search, and final answer.
"""

import argparse
import hashlib
import json
import os
import subprocess
import time
from pathlib import Path

from basic_memory_benchmark import keywords


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_case(codex: Path, workdir: Path, prompt: str, config: list[str]) -> dict:
    command = [
        str(codex), "exec", "--json", "--ephemeral", "--ignore-user-config",
        "--ignore-rules", "--skip-git-repo-check", "-s", "read-only",
        "-m", "gpt-6.1-sol", "-C", str(workdir),
    ]
    for setting in config:
        command.extend(["-c", setting])
    command.append("-")
    started = time.perf_counter_ns()
    process = subprocess.run(command, input=prompt, text=True, capture_output=True, timeout=90)
    if process.returncode:
        raise RuntimeError(f"Codex exit {process.returncode}: {process.stderr[-1000:]}")
    events = [json.loads(line) for line in process.stdout.splitlines() if line.strip()]
    answers = [e["item"]["text"] for e in events
               if e.get("type") == "item.completed" and e.get("item", {}).get("type") == "agent_message"]
    calls = [e["item"] for e in events
             if e.get("type") == "item.completed" and e.get("item", {}).get("type") == "mcp_tool_call"]
    forbidden = [e["item"]["type"] for e in events
                 if e.get("type") == "item.completed"
                 and e.get("item", {}).get("type") in ("command_execution", "web_search")]
    if not answers or not calls or forbidden or any(call.get("error") or call.get("status") != "completed" for call in calls):
        raise ValueError(
            f"missing answer/MCP call or forbidden tool: {len(answers)}, {len(calls)}, "
            f"{forbidden}; final={answers[-1][:300] if answers else None}"
        )
    usages = [e.get("usage") for e in events if e.get("type") == "turn.completed"]
    return {
        "final_answer_ms": (time.perf_counter_ns() - started) / 1_000_000,
        "final_answer": answers[-1], "usage": usages[-1] if usages else None,
        "tool_calls": len(calls), "mcp_calls": calls, "events": events,
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("provider", choices=["basic-memory", "lodis"])
    parser.add_argument("cases", type=Path)
    parser.add_argument("selection", type=Path)
    parser.add_argument("codex", type=Path)
    parser.add_argument("server", type=Path)
    parser.add_argument("data_dir", type=Path)
    parser.add_argument("output_dir", type=Path)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=False)
    workdir = args.output_dir / "workdir"
    workdir.mkdir()
    cases = {row["id"]: row for line in args.cases.read_text().splitlines()
             if (row := json.loads(line))}
    ids = json.loads(args.selection.read_text())["case_ids"]
    if len(ids) != len(set(ids)) or any(item not in cases for item in ids):
        raise ValueError("invalid frozen selection")
    if args.provider == "lodis":
        config = [
            'mcp_servers.lodis.command="' + str(args.server) + '"',
            'mcp_servers.lodis.args=["' + str(args.data_dir / "packages/mcp-server/dist/cli.js") + '"]',
            'mcp_servers.lodis.env.LODIS_BENCHMARK_DIR="' + str(args.data_dir / "semantic-data") + '"',
            'mcp_servers.lodis.default_tools_approval_mode="approve"',
        ]
    else:
        config = [
            'mcp_servers.basic_memory.command="' + str(args.server) + '"',
            'mcp_servers.basic_memory.args=["mcp","--project","main"]',
            'mcp_servers.basic_memory.env.BASIC_MEMORY_CONFIG_DIR="' + str(args.data_dir / "config") + '"',
            'mcp_servers.basic_memory.env.BASIC_MEMORY_HOME="' + str(args.data_dir / "benchmark-notes") + '"',
            'mcp_servers.basic_memory.default_tools_approval_mode="approve"',
            'mcp_servers.basic_memory.startup_timeout_sec=60',
            'mcp_servers.basic_memory.required=true',
        ]
    rows = []
    try:
        for case_id in ids:
            case = cases[case_id]
            request = case["request"]["search"]
            scope = f"{request['scope_kind']}:{request['scope_id']}"
            if args.provider == "lodis":
                instruction = f"Busca con memory_search en el dominio exacto {scope} (limit 20, expand false)."
            else:
                instruction = (
                    "Llama a basic_memory.search_notes una vez con project=main, "
                    f"query={json.dumps(keywords(request['query']), ensure_ascii=False)}, "
                    'search_type="text", page_size=20 y metadata_filters='
                    f'{{"scope_kind":"{request["scope_kind"]}","scope_id":"{request["scope_id"]}"}}.'
                )
            mandatory_search = (
                "Consulta la herramienta antes de responder, también si la pregunta parece no tener respuesta. "
                "No concluyas que falta información sin consultar. "
                if args.provider == "basic-memory" else ""
            )
            prompt = (
                f"Pregunta del usuario: {request['query']}\n\n{instruction} "
                f"{mandatory_search}"
                "Responde brevemente solo con la memoria encontrada. Si no hay memoria pertinente, dilo. "
                "No uses archivos, shell ni otras herramientas."
            )
            measured = run_case(args.codex, workdir, prompt, config)
            events = measured.pop("events")
            (args.output_dir / f"{case_id}-events.jsonl").write_text(
                "".join(json.dumps(event, ensure_ascii=False) + "\n" for event in events)
            )
            rows.append({"case_id": case_id, "expected_ids": case["expected_ids"],
                         "expected_fact": case.get("expected_fact"), "prompt": prompt,
                         **measured})
            print(f"{case_id}: {measured['final_answer_ms']:.1f} ms, {measured['tool_calls']} MCP", flush=True)
    finally:
        workdir.rmdir()
    (args.output_dir / "summary.json").write_text(json.dumps({
        "kind": "external_codex_mcp_synthetic", "provider": args.provider,
        "codex_model": "gpt-6.1-sol", "cases_sha256": digest(args.cases),
        "selection_sha256": digest(args.selection), "server_sha256": digest(args.server),
        "case_ids": ids, "rows": rows,
    }, ensure_ascii=False, indent=2) + "\n")


if __name__ == "__main__":
    main()
