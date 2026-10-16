"""Import and search the frozen synthetic holdout with a pinned Lodis MCP binary.

The benchmark store must be isolated with LODIS_BENCHMARK_DIR. The pinned Lodis
checkout needs only a local-data-path patch in db.ts, embeddings.ts, reranker.ts.
"""

import argparse
import hashlib
import json
import os
import re
import sqlite3
import subprocess
import time
from pathlib import Path

from basic_memory_benchmark import keywords


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


class Mcp:
    def __init__(self, binary: Path):
        if not os.environ.get("LODIS_BENCHMARK_DIR"):
            raise ValueError("LODIS_BENCHMARK_DIR is required")
        self.process = subprocess.Popen(
            ["node", str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.PIPE, text=True, bufsize=1,
        )
        self.next_id = 0
        self.call("initialize", {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "iromem-eval", "version": "0.1"},
        })

    def call(self, method: str, params: dict) -> dict:
        self.next_id += 1
        self.process.stdin.write(json.dumps({
            "jsonrpc": "2.0", "id": self.next_id, "method": method, "params": params,
        }) + "\n")
        self.process.stdin.flush()
        response = json.loads(self.process.stdout.readline())
        if response.get("id") != self.next_id or "error" in response:
            raise ValueError(f"MCP failed: {response}")
        return response["result"]

    def tool(self, name: str, args: dict) -> dict:
        result = self.call("tools/call", {"name": name, "arguments": args})
        if result.get("isError"):
            raise ValueError(f"{name} failed: {result}")
        return json.loads(next(item["text"] for item in result["content"] if item["type"] == "text"))

    def close(self) -> None:
        self.process.stdin.close()
        self.process.terminate()
        self.process.wait(timeout=10)
        error = self.process.stderr.read()
        if error:
            print(error[-1000:], file=os.sys.stderr)


def import_memories(binary: Path, memories: Path, mapping_path: Path) -> None:
    if mapping_path.exists():
        raise FileExistsError("choose a fresh mapping path")
    items = json.loads(memories.read_text())
    if len({item["id"] for item in items}) != len(items):
        raise ValueError("duplicate memory ID")
    mcp = Mcp(binary)
    mapping = {}
    started = time.perf_counter()
    try:
        for offset in range(0, len(items), 400):
            batch = items[offset:offset + 400]
            entries = [{
                "content": item["summary"], "detail": item["original_text"],
                "domain": f"{item['scope_kind']}:{item['scope_id']}",
                "entityType": "fact", "sourceType": "observed",
                "structuredData": {"source_id": item["id"]},
            } for item in batch]
            result = mcp.tool("memory_bulk_upload", {
                "entries": entries, "sourceAgentId": "iromem-eval",
                "sourceAgentName": "iromem-eval", "skipDedup": True,
            })
            if result["written"] != len(batch) or result["failed"] or result["skipped"]:
                raise ValueError(f"incomplete import at {offset}: {result}")
            for row in result["results"]:
                mapping[row["id"]] = batch[row["index"]]["id"]
            print(f"imported {len(mapping)}/{len(items)}", flush=True)
    finally:
        mcp.close()
    if len(mapping) != len(items):
        raise ValueError("incomplete ID mapping")
    database = Path(os.environ["LODIS_BENCHMARK_DIR"]) / "lodis.db"
    with sqlite3.connect(database) as connection:
        embedded = connection.execute(
            "SELECT COUNT(id) FROM memories WHERE embedding IS NOT NULL"
        ).fetchone()[0]
    mapping_path.write_text(json.dumps({
        "mapping": mapping, "memories_sha256": digest(memories),
        "import_seconds": time.perf_counter() - started,
        "embedded_count": embedded,
    }, indent=2) + "\n")
    print(f"embedded {embedded}/{len(items)}")


def search(binary: Path, queries: Path, mapping_path: Path, output: Path, split: str) -> None:
    if output.exists() or output.with_suffix(".manifest.json").exists():
        raise FileExistsError("choose a fresh output path")
    cases = [json.loads(line) for line in queries.read_text().splitlines() if line.strip()]
    if len({case["id"] for case in cases}) != len(cases) or any(case["split"] != split for case in cases):
        raise ValueError("invalid case cohort")
    imported = json.loads(mapping_path.read_text())
    if imported.get("embedded_count") != len(imported["mapping"]):
        raise ValueError("Lodis embeddings are incomplete; this is not a hybrid run")
    mapping = imported["mapping"]
    mcp = Mcp(binary)

    def query(case: dict) -> list[str]:
        request = case["request"]["search"]
        domain = f"{request['scope_kind']}:{request['scope_id']}"
        result = mcp.tool("memory_search", {
            "query": keywords(request["query"]), "domain": domain,
            "limit": 20, "expand": False,
        })
        ids = []
        for item in result["memories"]:
            if item["domain"] != domain:
                raise ValueError("Lodis returned another domain")
            ids.append(mapping[item["id"]])
        if len(ids) > 20 or len(set(ids)) != len(ids):
            raise ValueError("invalid candidates")
        return ids

    try:
        query(cases[0])
        with output.open("x") as writer:
            for case in cases:
                started = time.perf_counter_ns()
                ids = query(case)
                elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000
                row = {key: value for key, value in case.items() if key != "request"}
                row.update({
                    "elapsed_ms": elapsed_ms,
                    "normalized_query": keywords(case["request"]["search"]["query"]),
                    "response": {"status": "complete", "method": "lodis_hybrid",
                                 "semantic_coverage": "complete",
                                 "candidates": [{"memory_id": item} for item in ids]},
                })
                writer.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")
            writer.flush()
            os.fsync(writer.fileno())
    finally:
        mcp.close()
    output.with_suffix(".manifest.json").write_text(json.dumps({
        "kind": "lodis_mcp_benchmark", "method": "lodis_hybrid", "split": split,
        "cases": len(cases), "cases_sha256": digest(queries),
        "mapping_sha256": digest(mapping_path), "binary_sha256": digest(binary),
        "query": "engram_keywords_with_exact_domain_filter",
    }, indent=2) + "\n")
    print(f"wrote {len(cases)} search results to {output}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    imp = sub.add_parser("import")
    imp.add_argument("binary", type=Path)
    imp.add_argument("memories", type=Path)
    imp.add_argument("mapping", type=Path)
    bench = sub.add_parser("search")
    bench.add_argument("binary", type=Path)
    bench.add_argument("queries", type=Path)
    bench.add_argument("mapping", type=Path)
    bench.add_argument("output", type=Path)
    bench.add_argument("split")
    args = parser.parse_args()
    if args.command == "import":
        import_memories(args.binary, args.memories, args.mapping)
    else:
        search(args.binary, args.queries, args.mapping, args.output, args.split)
