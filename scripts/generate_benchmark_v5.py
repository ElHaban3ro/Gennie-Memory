#!/usr/bin/env python3
"""Generate a large conditional-memory benchmark with two rules per project."""

import argparse
import hashlib
import json
from pathlib import Path

from generate_benchmark import AS_OF, NAMES, split_for, write_json, write_jsonl
from generate_benchmark_v2 import FRAMES, VALUE_BANKS, forms

MODES = {
    "remote": {
        "task": "preparar reuniones remotas",
        "task_en": "prepare remote meetings",
        "condition": "Solo para reuniones remotas",
    },
    "onsite": {
        "task": "preparar reuniones presenciales",
        "task_en": "prepare in-person meetings",
        "condition": "Solo para reuniones presenciales",
    },
}


def opaque_id(kind: str, number: int, mode: str) -> str:
    value = hashlib.sha256(f"iromem-v5-{kind}-{number}-{mode}".encode()).hexdigest()
    return f"m_{value[:20]}"


def values_for(index: int) -> dict[str, str]:
    bank = VALUE_BANKS[4]
    digest = hashlib.sha256(f"iromem-v5-values-{index}".encode()).digest()
    first = int.from_bytes(digest[:4], "big") % len(bank)
    step = 1 + int.from_bytes(digest[4:8], "big") % (len(bank) - 1)
    return {"remote": bank[first], "onsite": bank[(first + step) % len(bank)]}


def generate_v5(entity_count: int, variants_per_entity: int) -> tuple[list[dict], list[dict], list[dict]]:
    prompt_forms = forms()
    corpus, cases = [], []
    for index in range(entity_count):
        number = index + 1
        project = f"Proyecto {NAMES[index % len(NAMES)]} {number:04d}"
        alias = f"PX-{number:04d}"
        entity_id = f"project_{number:04d}"
        scope = f"laboratorio_{index % 20:02d}"
        source_ref = f"synthetic:meeting-guide/{number:04d}@v2"
        values = values_for(index)
        common = {"split": split_for(index), "label_source": "deterministic_fixture_rule"}
        request = {"scope_kind": "project", "scope_id": scope, "entity_id": entity_id,
                   "as_of": AS_OF, "limit": 20}
        for mode, config in MODES.items():
            memory_id = opaque_id("rule", number, mode)
            condition = config["condition"]
            source = {"kind": "document", "ref": source_ref, "locator": mode}
            value = values[mode]
            summary = f"{project} ({alias}): si la reunión es {'remota' if mode == 'remote' else 'presencial'}, {value}."
            original = f"En {project} ({alias}), {condition.lower()}: {value}."
            revision = 2 if index % 10 == 0 else 1
            if revision == 2:
                bank = VALUE_BANKS[4]
                old_value = bank[(bank.index(value) + 1) % len(bank)]
                old_source = dict(source, ref=f"synthetic:meeting-guide/{number:04d}@v1")
                corpus.append({
                    "id": memory_id, "scope_kind": "project", "scope_id": scope,
                    "entity_id": entity_id, "revision": 1, "memory_type": "procedure",
                    "original_text": f"Versión anterior: {original.replace(value, old_value)}",
                    "summary": f"Versión anterior: {summary.replace(value, old_value)}",
                    "conditions": condition, "source": old_source, "state": "corrected",
                    "valid_from": None, "valid_until": None,
                })
            corpus.append({
                "id": memory_id, "scope_kind": "project", "scope_id": scope,
                "entity_id": entity_id, "revision": revision, "memory_type": "procedure",
                "original_text": original, "summary": summary,
                "conditions": condition, "source": source, "state": "active",
                "valid_from": None, "valid_until": None,
            })
        shadow_id = opaque_id("shadow", number, "remote")
        corpus.append({
            "id": shadow_id, "scope_kind": "project",
            "scope_id": f"foreign_{index % 20:02d}", "entity_id": entity_id,
            "revision": 1, "memory_type": "procedure",
            "original_text": f"Otro ámbito de {project}: {values['remote']}.",
            "summary": f"Otro ámbito de {project}: {values['remote']}.",
            "conditions": MODES["remote"]["condition"],
            "source": {"kind": "document", "ref": f"synthetic:foreign-guide/{number:04d}",
                       "locator": "remote"},
            "state": "active", "valid_from": None, "valid_until": None,
        })
        for variant in range(variants_per_entity):
            form_id = (variant % len(FRAMES)) * 50 + ((index * 5 + variant // len(FRAMES)) % 50)
            form = prompt_forms[form_id]
            mode_digest = hashlib.sha256(f"iromem-v5-mode-{number}-{form_id}".encode()).digest()
            mode = "remote" if mode_digest[0] % 2 == 0 else "onsite"
            config = MODES[mode]
            query = form["template"].format(
                project=project, alias=alias,
                task=config["task"], task_en=config["task_en"])
            cases.append({
                "id": f"q5_{number:04d}_{form_id:04d}", "segment": form["segment"],
                "request": dict(request, query=query),
                "expected_ids": [opaque_id("rule", number, mode)],
                "expected_fact": values[mode],
                "expected_condition": config["condition"],
                "expected_source": {"kind": "document", "ref": source_ref, "locator": mode},
                "forbidden_ids": [shadow_id], **common,
            })
        negatives = (
            ("mode_unspecified", f"Para {project}, ¿cómo preparo la reunión? No sé si será remota o presencial.", scope, entity_id),
            ("mode_unknown", f"En {project} la modalidad de la reunión sigue sin decidirse. ¿Qué instrucción aplico ya?", scope, entity_id),
            ("absent_entity", f"¿Qué regla de reuniones aplica a Proyecto Inexistente {number:04d}?", scope, f"absent_{number:04d}"),
            ("wrong_scope", f"¿Cómo preparo reuniones remotas en {project}?", f"unassigned_{index % 20:02d}", entity_id),
            ("wrong_task", f"Para {project}, ¿qué pauta rige al cerrar incidencias?", scope, entity_id),
            ("explicit_no_recall", f"No consultes recuerdos de {project}; escribe una frase genérica.", scope, entity_id),
        )
        for segment, query, query_scope, query_entity in negatives:
            forbidden = [shadow_id]
            if segment == "wrong_scope":
                forbidden.extend(opaque_id("rule", number, mode) for mode in MODES)
            cases.append({
                "id": f"{segment}_{number:04d}", "segment": segment,
                "request": dict(request, query=query, scope_id=query_scope,
                                entity_id=query_entity),
                "expected_ids": [], "expected_fact": None,
                "expected_condition": None, "expected_source": None,
                "forbidden_ids": forbidden, **common,
            })
    return corpus, cases, prompt_forms


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--entities", type=int, default=400)
    parser.add_argument("--variants-per-entity", type=int, default=100)
    parser.add_argument("--output", type=Path, default=Path("tests/benchmark/v5"))
    args = parser.parse_args()
    if not 10 <= args.entities <= 10000 or not 10 <= args.variants_per_entity <= 1000:
        parser.error("entities must be 10..10000 and variants-per-entity 10..1000")
    corpus, cases, prompt_forms = generate_v5(args.entities, args.variants_per_entity)
    args.output.mkdir(parents=True, exist_ok=True)
    write_json(args.output / "corpus.json", corpus)
    write_json(args.output / "forms.json", prompt_forms)
    write_jsonl(args.output / "queries.jsonl", cases)
    manifest = {
        "version": 5, "synthetic": True, "generator": "scripts/generate_benchmark_v5.py",
        "entity_count": args.entities, "memories_per_entity": 2,
        "structured_entity_context": "provided_by_synthetic_host",
        "prompt_form_count": len(prompt_forms),
        "used_prompt_forms": len({int(case["id"].split("_")[-1]) for case in cases
                                  if case["id"].startswith("q5_")}),
        "positive_variants_per_entity": args.variants_per_entity,
        "fixture_revision_count": len(corpus),
        "unique_memories": len({row["id"] for row in corpus}),
        "query_count": len(cases),
        "positive_queries": args.entities * args.variants_per_entity,
        "negative_queries": args.entities * 6, "as_of": AS_OF,
        "partitions": {name: sum(case["split"] == name for case in cases)
                       for name in ("development", "calibration", "final")},
        "segments": {name: sum(case["segment"] == name for case in cases)
                     for name in sorted({case["segment"] for case in cases})},
        "files": {name: hashlib.sha256((args.output / name).read_bytes()).hexdigest()
                  for name in ("corpus.json", "forms.json", "queries.jsonl")},
    }
    write_json(args.output / "manifest.json", manifest)
    print(json.dumps({key: value for key, value in manifest.items() if key != "files"}))


if __name__ == "__main__":
    main()
