#!/usr/bin/env python3
"""Generate a multi-memory-per-entity synthetic retrieval benchmark."""

import argparse
import hashlib
import json
from pathlib import Path

from generate_benchmark import AS_OF, NAMES, TASKS, write_json, write_jsonl
from generate_benchmark_v2 import VALUE_BANKS, generate_v2, value_for


def opaque_id(kind: str, number: int, task_index: int) -> str:
    digest = hashlib.sha256(f"iromem-v4-memory-{kind}-{number}-{task_index}".encode()).hexdigest()
    return f"m_{digest[:20]}"


def alternate_value(index: int, task_index: int) -> str:
    bank = VALUE_BANKS[task_index]
    digest = hashlib.sha256(f"iromem-v4-value-{index}-{task_index}".encode()).digest()
    return bank[int.from_bytes(digest[:4], "big") % len(bank)]


def task_slots(index: int) -> tuple[int, int, int]:
    primary = index % len(TASKS)
    return primary, (primary + 2) % len(TASKS), (primary + 3) % len(TASKS)


def generate_v4(entity_count: int, variants_per_entity: int) -> tuple[list[dict], list[dict], list[dict]]:
    old_corpus, old_cases, forms = generate_v2(entity_count, variants_per_entity)
    corpus = []
    for row in old_corpus:
        kind, suffix = row["id"].split("_", 1)
        number = int(suffix)
        index = number - 1
        updated = dict(row)
        updated["id"] = opaque_id(kind, number, index % len(TASKS))
        updated["entity_id"] = f"project_{number:04d}"
        corpus.append(updated)
    for index in range(entity_count):
        number = index + 1
        project = f"Proyecto {NAMES[index % len(NAMES)]} {number:04d}"
        alias = f"PX-{number:04d}"
        scope = f"laboratorio_{index % 20:02d}"
        for task_index in task_slots(index)[1:]:
            task, _, _, _ = TASKS[task_index]
            value = alternate_value(index, task_index)
            corpus.append(dict(id=opaque_id("task", number, task_index),
                               scope_kind="project", scope_id=scope,
                               entity_id=f"project_{number:04d}", revision=1,
                               memory_type="preference",
                               original_text=f"En {project} (alias {alias}), al {task}, la preferencia vigente es {value}.",
                               summary=f"{project} ({alias}): al {task}, {value}.",
                               state="active", valid_from=None, valid_until=None))
    cases = []
    for old in old_cases:
        case = dict(old)
        request = dict(case["request"])
        case["request"] = request
        number = int(case["id"].split("_")[-2] if case["id"].startswith("q2_")
                     else case["id"].split("_")[-1])
        index = number - 1
        primary = index % len(TASKS)
        request["entity_id"] = (f"absent_{number:04d}" if case["segment"] == "absent_entity"
                                else f"project_{number:04d}")
        if case["id"].startswith("q2_"):
            form_id = int(case["id"].split("_")[-1])
            digest = hashlib.sha256(f"iromem-v4-target-{number}-{form_id}".encode()).digest()
            slot = int.from_bytes(digest[:4], "big") % 3
            task_index = task_slots(index)[slot]
            task, task_en, _, _ = TASKS[task_index]
            project = f"Proyecto {NAMES[index % len(NAMES)]} {number:04d}"
            alias = f"PX-{number:04d}"
            request["query"] = forms[form_id]["template"].format(
                project=project, alias=alias, task=task, task_en=task_en)
            memory_id = (opaque_id("synthetic", number, primary) if slot == 0
                         else opaque_id("task", number, task_index))
            case["expected_ids"] = [memory_id]
            case["expected_fact"] = (value_for(index) if slot == 0
                                     else alternate_value(index, task_index))
        else:
            case["expected_ids"] = []
            case["expected_fact"] = None
        case["forbidden_ids"] = [opaque_id("shadow", number, primary)]
        if case["segment"] == "wrong_scope":
            case["forbidden_ids"].append(opaque_id("synthetic", number, primary))
        if index % 10 == 0:
            case["forbidden_ids"].append(opaque_id("expired", number, primary))
        cases.append(case)
    return corpus, cases, forms


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--entities", type=int, default=400)
    parser.add_argument("--variants-per-entity", type=int, default=100)
    parser.add_argument("--output", type=Path, default=Path("tests/benchmark/v4"))
    args = parser.parse_args()
    if not 10 <= args.entities <= 10000 or not 10 <= args.variants_per_entity <= 1000:
        parser.error("entities must be 10..10000 and variants-per-entity 10..1000")
    corpus, cases, forms = generate_v4(args.entities, args.variants_per_entity)
    args.output.mkdir(parents=True, exist_ok=True)
    write_json(args.output / "corpus.json", corpus)
    write_json(args.output / "forms.json", forms)
    write_jsonl(args.output / "queries.jsonl", cases)
    manifest = dict(version=4, synthetic=True, generator="scripts/generate_benchmark_v4.py",
                    entity_count=args.entities, memories_per_entity=3,
                    structured_entity_context="provided_by_synthetic_host",
                    prompt_form_count=len(forms),
                    used_prompt_forms=len({int(case["id"].split("_")[-1]) for case in cases
                                           if case["id"].startswith("q2_")}),
                    positive_variants_per_entity=args.variants_per_entity,
                    fixture_revision_count=len(corpus),
                    unique_memories=len({memory["id"] for memory in corpus}),
                    query_count=len(cases), positive_queries=args.entities * args.variants_per_entity,
                    negative_queries=args.entities * 4, as_of=AS_OF,
                    partitions={name: sum(case["split"] == name for case in cases)
                                for name in ("development", "calibration", "final")},
                    segments={name: sum(case["segment"] == name for case in cases)
                              for name in sorted({case["segment"] for case in cases})},
                    files={name: hashlib.sha256((args.output / name).read_bytes()).hexdigest()
                           for name in ("corpus.json", "forms.json", "queries.jsonl")})
    write_json(args.output / "manifest.json", manifest)
    print(json.dumps({key: value for key, value in manifest.items() if key != "files"}))


if __name__ == "__main__":
    main()
