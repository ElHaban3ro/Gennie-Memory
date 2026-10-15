#!/usr/bin/env python3
"""Check labels, scope isolation, partitions, and hashes of a generated benchmark."""

import argparse
import hashlib
import json
import re
from collections import defaultdict
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    root = args.directory
    manifest = json.loads((root / "manifest.json").read_text())
    assert manifest["synthetic"] is True
    for name, digest in manifest["files"].items():
        assert hashlib.sha256((root / name).read_bytes()).hexdigest() == digest, name
    corpus = json.loads((root / "corpus.json").read_text())
    cases = [json.loads(line) for line in (root / "queries.jsonl").read_text().splitlines()]
    assert len(corpus) == manifest["fixture_revision_count"]
    assert len(cases) == manifest["query_count"]
    assert len({row["id"] for row in cases}) == len(cases)
    assert len({(row["id"], row["revision"]) for row in corpus}) == len(corpus)
    assert len({row["id"] for row in corpus}) == manifest["unique_memories"]
    assert sum(bool(case["expected_ids"]) for case in cases) == manifest["positive_queries"]
    assert sum(not case["expected_ids"] for case in cases) == manifest["negative_queries"]
    revisions = defaultdict(list)
    for row in corpus:
        revisions[row["id"]].append(row)
        if manifest["version"] in (4, 5):
            assert re.fullmatch(r"m_[0-9a-f]{20}", row["id"])
        else:
            assert row["id"].startswith(("synthetic_", "shadow_", "expired_"))
    current = {identity: max(rows, key=lambda row: row["revision"])
               for identity, rows in revisions.items()}
    if manifest["version"] in (2, 3):
        from generate_benchmark import NAMES, TASKS
        from generate_benchmark_v2 import value_for

        forms = json.loads((root / "forms.json").read_text())
        assert len(forms) == manifest["prompt_form_count"] == 1000
        assert [form["id"] for form in forms] == list(range(1000))
        assert len({form["template"] for form in forms}) == 1000
        used_forms = set()
        positive_counts = defaultdict(int)
        negative_counts = defaultdict(int)
        for case in cases:
            pieces = case["id"].split("_")
            if pieces[0] == "q2":
                number, form_id = int(pieces[1]), int(pieces[2])
                assert 1 <= number <= manifest["entity_count"]
                assert 0 <= form_id < 1000
                form = forms[form_id]
                index = number - 1
                task, task_en, _, _ = TASKS[index % len(TASKS)]
                expected_query = form["template"].format(
                    project=f"Proyecto {NAMES[index % len(NAMES)]} {number:04d}",
                    alias=f"PX-{number:04d}", task=task, task_en=task_en)
                assert case["request"]["query"] == expected_query
                assert case["segment"] == form["segment"]
                assert case["expected_ids"] == [f"synthetic_{number:04d}"]
                assert case["expected_fact"] == value_for(index)
                if manifest["version"] == 3:
                    assert case["request"]["entity_id"] == f"project_{number:04d}"
                used_forms.add(form_id)
                positive_counts[number] += 1
            else:
                number = int(pieces[-1])
                assert 1 <= number <= manifest["entity_count"]
                assert case["segment"] in ("absent_entity", "wrong_scope", "wrong_task", "explicit_no_recall")
                assert case["expected_ids"] == [] and case["expected_fact"] is None
                if manifest["version"] == 3:
                    assert case["request"]["entity_id"] == (
                        f"absent_{number:04d}" if case["segment"] == "absent_entity"
                        else f"project_{number:04d}")
                negative_counts[number] += 1
        assert len(used_forms) == manifest["used_prompt_forms"]
        assert all(positive_counts[number] == manifest["positive_variants_per_entity"]
                   and negative_counts[number] == 4
                   for number in range(1, manifest["entity_count"] + 1))
        assert {name: sum(case["segment"] == name for case in cases)
                for name in manifest["segments"]} == manifest["segments"]
        assert {str(task_index): len({value_for(index)
                for index in range(task_index, manifest["entity_count"], len(TASKS))})
                for task_index in range(len(TASKS))} == manifest["distinct_values_by_task"]
        if manifest["version"] == 3:
            assert manifest["structured_entity_context"] == "provided_by_synthetic_host"
            assert all(row["entity_id"] == f'project_{int(row["id"].split("_")[-1]):04d}'
                       for row in corpus)
    elif manifest["version"] == 4:
        from generate_benchmark import NAMES, TASKS
        from generate_benchmark_v2 import value_for
        from generate_benchmark_v4 import alternate_value, opaque_id, task_slots

        forms = json.loads((root / "forms.json").read_text())
        assert len(forms) == manifest["prompt_form_count"] == 1000
        assert [form["id"] for form in forms] == list(range(1000))
        assert len({form["template"] for form in forms}) == 1000
        assert manifest["memories_per_entity"] == 3
        assert manifest["structured_entity_context"] == "provided_by_synthetic_host"
        assert len(corpus) == 4 * manifest["entity_count"] + manifest["entity_count"] // 5
        assert len(current) == 4 * manifest["entity_count"] + manifest["entity_count"] // 10
        active_by_entity = defaultdict(list)
        for row in current.values():
            assert row["entity_id"].startswith("project_")
            if row["state"] == "active" and row["id"] not in {
                opaque_id("shadow", int(row["entity_id"].split("_")[-1]),
                          (int(row["entity_id"].split("_")[-1]) - 1) % len(TASKS))
            }:
                active_by_entity[row["entity_id"]].append(row)
        assert all(len(active_by_entity[f"project_{number:04d}"]) == 3
                   for number in range(1, manifest["entity_count"] + 1))
        used_forms = set()
        positive_counts = defaultdict(int)
        negative_counts = defaultdict(int)
        task_counts = defaultdict(set)
        for case in cases:
            pieces = case["id"].split("_")
            number = int(pieces[1] if pieces[0] == "q2" else pieces[-1])
            assert 1 <= number <= manifest["entity_count"]
            index = number - 1
            project = f"Proyecto {NAMES[index % len(NAMES)]} {number:04d}"
            alias = f"PX-{number:04d}"
            if pieces[0] == "q2":
                form_id = int(pieces[2])
                assert 0 <= form_id < 1000
                digest = hashlib.sha256(f"iromem-v4-target-{number}-{form_id}".encode()).digest()
                slot = int.from_bytes(digest[:4], "big") % 3
                task_index = task_slots(index)[slot]
                task, task_en, _, _ = TASKS[task_index]
                assert case["request"]["query"] == forms[form_id]["template"].format(
                    project=project, alias=alias, task=task, task_en=task_en)
                assert case["segment"] == forms[form_id]["segment"]
                expected_id = (opaque_id("synthetic", number, index % len(TASKS)) if slot == 0
                               else opaque_id("task", number, task_index))
                expected_fact = value_for(index) if slot == 0 else alternate_value(index, task_index)
                assert case["expected_ids"] == [expected_id]
                assert case["expected_fact"] == expected_fact
                assert expected_fact not in case["request"]["query"]
                used_forms.add(form_id)
                positive_counts[number] += 1
                task_counts[number].add(task_index)
            else:
                assert case["segment"] in ("absent_entity", "wrong_scope", "wrong_task", "explicit_no_recall")
                assert case["expected_ids"] == [] and case["expected_fact"] is None
                negative_counts[number] += 1
            assert case["request"]["entity_id"] == (
                f"absent_{number:04d}" if case["segment"] == "absent_entity"
                else f"project_{number:04d}")
            assert all(identity in current for identity in case["forbidden_ids"])
        assert len(used_forms) == manifest["used_prompt_forms"]
        assert all(positive_counts[number] == manifest["positive_variants_per_entity"]
                   and negative_counts[number] == 4 and len(task_counts[number]) == 3
                   for number in range(1, manifest["entity_count"] + 1))
        assert {name: sum(case["segment"] == name for case in cases)
                for name in manifest["segments"]} == manifest["segments"]
    elif manifest["version"] == 5:
        from generate_benchmark import NAMES, split_for
        from generate_benchmark_v2 import FRAMES
        from generate_benchmark_v5 import MODES, opaque_id, values_for

        forms = json.loads((root / "forms.json").read_text())
        assert len(forms) == manifest["prompt_form_count"] == 1000
        assert [form["id"] for form in forms] == list(range(1000))
        assert len({form["template"] for form in forms}) == 1000
        assert manifest["memories_per_entity"] == 2
        assert manifest["structured_entity_context"] == "provided_by_synthetic_host"
        assert len(corpus) == 3 * manifest["entity_count"] + 2 * ((manifest["entity_count"] + 9) // 10)
        assert len(current) == 3 * manifest["entity_count"]
        used_forms = set()
        positive_counts, negative_counts = defaultdict(int), defaultdict(int)
        modes_by_entity = defaultdict(set)
        for case in cases:
            parts = case["id"].split("_")
            number = int(parts[1] if parts[0] == "q5" else parts[-1])
            assert 1 <= number <= manifest["entity_count"]
            index = number - 1
            project = f"Proyecto {NAMES[index % len(NAMES)]} {number:04d}"
            alias = f"PX-{number:04d}"
            assert case["split"] == split_for(index)
            request = case["request"]
            assert request["scope_kind"] == "project" and request["as_of"] == manifest["as_of"]
            assert request["entity_id"] == (
                f"absent_{number:04d}" if case["segment"] == "absent_entity"
                else f"project_{number:04d}")
            assert opaque_id("shadow", number, "remote") in case["forbidden_ids"]
            if parts[0] == "q5":
                form_id = int(parts[2])
                assert 0 <= form_id < 1000
                mode_digest = hashlib.sha256(f"iromem-v5-mode-{number}-{form_id}".encode()).digest()
                mode = "remote" if mode_digest[0] % 2 == 0 else "onsite"
                config = MODES[mode]
                assert request["query"] == forms[form_id]["template"].format(
                    project=project, alias=alias,
                    task=config["task"], task_en=config["task_en"])
                assert case["segment"] == forms[form_id]["segment"]
                assert case["expected_ids"] == [opaque_id("rule", number, mode)]
                assert case["expected_fact"] == values_for(index)[mode]
                assert case["expected_fact"] not in request["query"]
                assert case["expected_condition"] == config["condition"]
                assert case["expected_source"] == {
                    "kind": "document", "ref": f"synthetic:meeting-guide/{number:04d}@v2",
                    "locator": mode,
                }
                assert case["expected_ids"][0] not in case["forbidden_ids"]
                positive_counts[number] += 1
                modes_by_entity[number].add(mode)
                used_forms.add(form_id)
            else:
                assert case["segment"] in (
                    "mode_unspecified", "mode_unknown", "absent_entity",
                    "wrong_scope", "wrong_task", "explicit_no_recall")
                assert case["expected_ids"] == []
                assert case["expected_fact"] is None
                assert case["expected_condition"] is None
                assert case["expected_source"] is None
                if case["segment"] == "wrong_scope":
                    assert all(opaque_id("rule", number, mode) in case["forbidden_ids"]
                               for mode in MODES)
                negative_counts[number] += 1
            assert all(identity in current for identity in case["forbidden_ids"])
        for number in range(1, manifest["entity_count"] + 1):
            assert positive_counts[number] == manifest["positive_variants_per_entity"]
            assert negative_counts[number] == 6
            assert modes_by_entity[number] == set(MODES)
            index = number - 1
            assert values_for(index)["remote"] != values_for(index)["onsite"]
            for mode, config in MODES.items():
                row = current[opaque_id("rule", number, mode)]
                assert row["state"] == "active"
                assert row["conditions"] == config["condition"]
                assert row["source"] == {
                    "kind": "document", "ref": f"synthetic:meeting-guide/{number:04d}@v2",
                    "locator": mode,
                }
                assert values_for(index)[mode] in row["summary"]
                assert (row["scope_id"], row["entity_id"]) == (
                    f"laboratorio_{index % 20:02d}", f"project_{number:04d}")
        assert len(used_forms) == manifest["used_prompt_forms"]
        assert {name: sum(case["segment"] == name for case in cases)
                for name in manifest["segments"]} == manifest["segments"]
    else:
        assert manifest["version"] == 1
    partitions = defaultdict(set)
    for case in cases:
        entity = case["id"].split("_")[-2] if case["id"].startswith(("q_", "q2_", "q5_")) else case["id"].split("_")[-1]
        partitions[entity].add(case["split"])
        assert case["label_source"] == "deterministic_fixture_rule"
        assert not set(case["expected_ids"]) & set(case["forbidden_ids"])
        for identity in case["expected_ids"]:
            row = current[identity]
            request = case["request"]
            assert row["state"] == "active"
            assert (row["scope_kind"], row["scope_id"]) == (request["scope_kind"], request["scope_id"])
            if manifest["version"] in (3, 5):
                assert row["entity_id"] == request["entity_id"]
            assert row["valid_from"] is None or row["valid_from"] <= request["as_of"]
            assert row["valid_until"] is None or row["valid_until"] > request["as_of"]
            assert case["expected_fact"] in row["summary"]
            if manifest["version"] == 5:
                assert row["conditions"] == case["expected_condition"]
                assert row["source"] == case["expected_source"]
    assert all(len(splits) == 1 for splits in partitions.values())
    assert {name: sum(case["split"] == name for case in cases)
            for name in ("development", "calibration", "final")} == manifest["partitions"]
    print(json.dumps({"valid": True, "fixture_revisions": len(corpus), "queries": len(cases),
                      "entity_partitions": len(partitions)}))


if __name__ == "__main__":
    main()
