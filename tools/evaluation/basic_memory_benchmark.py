"""Run the frozen iROMEM holdout against an isolated Basic Memory local index.

Use the Python interpreter from a pinned Basic Memory checkout. This adapter uses
Basic Memory's local ASGI search API, so its latency includes API serialization
but not MCP stdio. It never mutates the indexed store during timed searches.
"""

import argparse
import asyncio
import hashlib
import json
import os
import re
import time
from pathlib import Path

STOPWORDS = set(
    "a al algo como con cual cuando cuál cuáles cuándo cómo de del donde dónde el ella en es "
    "esta este esto fue ha hay la las le lo los mi mis no nos para por que quien quién qué se ser "
    "si son su sus te tiene un una unas unos y ya".split()
)


def keywords(value: str) -> str:
    return " ".join(
        word for word in re.findall(r"[\w]+", value.lower(), flags=re.UNICODE)
        if len(word) >= 3 and word not in STOPWORDS
    )


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def import_notes(memories: Path, notes: Path) -> None:
    notes.mkdir(parents=True, exist_ok=False)
    data = json.loads(memories.read_text())
    ids = set()
    for memory in data:
        memory_id = memory["id"]
        if memory_id in ids or not re.fullmatch(r"[a-zA-Z0-9_]+", memory_id):
            raise ValueError("duplicate or unsafe memory ID")
        ids.add(memory_id)
        title = json.dumps(memory["summary"], ensure_ascii=False)
        body = (
            f"---\ntitle: {title}\nsource_id: {memory_id}\n"
            f"scope_kind: {memory['scope_kind']}\nscope_id: {memory['scope_id']}\n"
            f"---\n\n{memory['original_text']}\n"
        )
        (notes / f"{memory_id}.md").write_text(body, encoding="utf-8")
    print(f"wrote {len(ids)} notes to {notes}")


async def run(queries: Path, output: Path, split: str) -> None:
    from basic_memory.mcp.async_client import get_client

    if output.exists() or output.with_suffix(".manifest.json").exists():
        raise FileExistsError("choose a fresh output path")
    cases = [json.loads(line) for line in queries.read_text().splitlines() if line.strip()]
    if len({case["id"] for case in cases}) != len(cases):
        raise ValueError("duplicate query ID")
    if any(case["split"] != split for case in cases):
        raise ValueError("unexpected split")

    async with get_client(project_name="main") as client:
        async def search(case: dict) -> list[str]:
            request = case["request"]["search"]
            payload = {
                "text": keywords(request["query"]),
                "metadata_filters": {
                    "scope_kind": request["scope_kind"],
                    "scope_id": request["scope_id"],
                },
                "project_ids": [1],
            }
            response = await client.request(
                "QUERY", "/v2/search/", json=payload, params={"page": 1, "page_size": 20}
            )
            response.raise_for_status()
            ids = []
            for item in response.json()["results"]:
                filename = Path(item["file_path"]).name
                if not re.fullmatch(r"gmv1_[0-9a-f]+\.md", filename):
                    raise ValueError(f"unmapped result: {filename}")
                ids.append(filename[:-3])
            if len(ids) > 20 or len(set(ids)) != len(ids):
                raise ValueError("invalid candidate list")
            return ids

        await search(cases[0])
        with output.open("x", encoding="utf-8") as writer:
            for case in cases:
                started = time.perf_counter_ns()
                ids = await search(case)
                elapsed_ms = (time.perf_counter_ns() - started) / 1_000_000
                row = {key: value for key, value in case.items() if key != "request"}
                row.update({
                    "elapsed_ms": elapsed_ms,
                    "normalized_query": keywords(case["request"]["search"]["query"]),
                    "response": {
                        "status": "complete",
                        "method": "basic_memory_fts",
                        "semantic_coverage": "disabled",
                        "candidates": [{"memory_id": item} for item in ids],
                    },
                })
                writer.write(json.dumps(row, ensure_ascii=False, separators=(",", ":")) + "\n")
            writer.flush()
            os.fsync(writer.fileno())

    manifest = {
        "kind": "basic_memory_local_asgi_benchmark",
        "method": "basic_memory_fts",
        "split": split,
        "cases": len(cases),
        "cases_sha256": digest(queries),
        "config_dir": os.environ.get("BASIC_MEMORY_CONFIG_DIR"),
        "query": "engram_keywords_with_exact_scope_metadata_filter",
        "transport": "in_process_asgi_QUERY",
    }
    output.with_suffix(".manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"wrote {len(cases)} search results to {output}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    imp = sub.add_parser("import-notes")
    imp.add_argument("memories", type=Path)
    imp.add_argument("notes", type=Path)
    bench = sub.add_parser("search")
    bench.add_argument("queries", type=Path)
    bench.add_argument("output", type=Path)
    bench.add_argument("split")
    args = parser.parse_args()
    if args.command == "import-notes":
        import_notes(args.memories, args.notes)
    else:
        asyncio.run(run(args.queries, args.output, args.split))
