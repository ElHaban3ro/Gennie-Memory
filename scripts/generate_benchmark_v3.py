#!/usr/bin/env python3
"""Add simulated resolved entity context to the synthetic v2 benchmark."""

import argparse
import hashlib
import json
from pathlib import Path

from generate_benchmark import AS_OF, TASKS, write_json, write_jsonl
from generate_benchmark_v2 import generate_v2, value_for


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--entities", type=int, default=400)
    parser.add_argument("--variants-per-entity", type=int, default=100)
    parser.add_argument("--output", type=Path, default=Path("tests/benchmark/v3"))
    args = parser.parse_args()
    if not 10 <= args.entities <= 10000 or not 10 <= args.variants_per_entity <= 1000:
        parser.error("entities must be 10..10000 and variants-per-entity 10..1000")
    corpus, cases, forms = generate_v2(args.entities, args.variants_per_entity)
    for memory in corpus:
        number = int(memory["id"].split("_")[-1])
        memory["entity_id"] = f"project_{number:04d}"
    for case in cases:
        number = int(case["id"].split("_")[-2] if case["id"].startswith("q2_")
                     else case["id"].split("_")[-1])
        case["request"]["entity_id"] = (f"absent_{number:04d}" if case["segment"] == "absent_entity"
                                          else f"project_{number:04d}")
    args.output.mkdir(parents=True, exist_ok=True)
    write_json(args.output / "corpus.json", corpus)
    write_json(args.output / "forms.json", forms)
    write_jsonl(args.output / "queries.jsonl", cases)
    manifest = dict(version=3, synthetic=True, generator="scripts/generate_benchmark_v3.py",
                    structured_entity_context="provided_by_synthetic_host",
                    entity_count=args.entities, prompt_form_count=len(forms),
                    used_prompt_forms=len({int(case["id"].split("_")[-1]) for case in cases
                                           if case["id"].startswith("q2_")}),
                    positive_variants_per_entity=args.variants_per_entity,
                    fixture_revision_count=len(corpus),
                    unique_memories=len({memory["id"] for memory in corpus}), query_count=len(cases),
                    positive_queries=args.entities * args.variants_per_entity,
                    negative_queries=args.entities * 4,
                    distinct_values_by_task={str(task_index): len({value_for(index)
                        for index in range(task_index, args.entities, len(TASKS))})
                        for task_index in range(len(TASKS))},
                    as_of=AS_OF, partitions={name: sum(case["split"] == name for case in cases)
                                          for name in ("development", "calibration", "final")},
                    segments={name: sum(case["segment"] == name for case in cases)
                              for name in sorted({case["segment"] for case in cases})},
                    files={name: hashlib.sha256((args.output / name).read_bytes()).hexdigest()
                           for name in ("corpus.json", "forms.json", "queries.jsonl")})
    write_json(args.output / "manifest.json", manifest)
    print(json.dumps({key: value for key, value in manifest.items() if key != "files"}))


if __name__ == "__main__":
    main()
