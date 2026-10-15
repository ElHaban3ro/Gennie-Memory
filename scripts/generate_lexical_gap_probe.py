#!/usr/bin/env python3
"""Derive a small synthetic paraphrase probe from labeled v5 cases without using final."""

import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path


PHRASES = {
    "remote": (
        "videollamada webcam auriculares", "teleconferencia pantalla enlace",
        "virtual huddle webcam headset", "video call screen sharing",
        "online sync camera headset", "audio link virtual gathering",
        "webcam session digital link", "telepresence camera screen",
        "videoconferencia auriculares webcam", "online meetup webcam audio",
    ),
    "onsite": (
        "caraacara recinto compartido", "face to face room chairs",
        "onsite gathering office lobby", "inperson workshop chairs",
        "physical meetup building room", "sameplace huddle office",
        "encuentro físico mesas sillas", "conversación caraacara recinto",
        "onsite session lobby chairs", "shared room workshop seating",
    ),
}
CONDITIONS = {"Solo para reuniones remotas": "remote",
              "Solo para reuniones presenciales": "onsite"}


def derive(paths: dict[str, Path], entities_per_split: int) -> list[dict]:
    result = []
    used_entities = set()
    for split in ("development", "calibration"):
        grouped = defaultdict(lambda: defaultdict(list))
        for line in paths[split].read_text().splitlines():
            if not line.strip():
                continue
            case = json.loads(line)
            if case["split"] != split or case["label_source"] != "deterministic_fixture_rule":
                raise ValueError("Input must contain labeled cases from only its declared split")
            mode = CONDITIONS.get(case.get("expected_condition"))
            if case["expected_ids"] and mode:
                grouped[case["request"]["entity_id"]][mode].append(case)
        eligible = [entity for entity, modes in grouped.items() if set(modes) == set(PHRASES)]
        eligible.sort(key=lambda entity: (hashlib.sha256(entity.encode()).hexdigest(), entity))
        if len(eligible) < entities_per_split or set(eligible) & used_entities:
            raise ValueError("Insufficient disjoint entities with both memory modes")
        for position, entity in enumerate(eligible[:entities_per_split]):
            for mode in ("remote", "onsite"):
                source = min(grouped[entity][mode], key=lambda case: case["id"])
                case = {**source, "id": f"lexgap_{split}_{entity}_{mode}",
                        "segment": f"lexical_gap_{mode}",
                        "request": {**source["request"],
                                    "query": PHRASES[mode][position % len(PHRASES[mode])]},
                        "derived_from_case": source["id"]}
                result.append(case)
        used_entities.update(eligible[:entities_per_split])
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--development", type=Path, required=True)
    parser.add_argument("--calibration", type=Path, required=True)
    parser.add_argument("--entities-per-split", type=int, default=50)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    if not 1 <= args.entities_per_split <= 100:
        parser.error("entities-per-split must be 1..100")
    if args.output_dir.resolve() in {args.development.resolve(), args.calibration.resolve()}:
        parser.error("Output directory must not overwrite an input")
    cases = derive({"development": args.development, "calibration": args.calibration},
                   args.entities_per_split)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    hashes = {}
    for split in ("development", "calibration"):
        path = args.output_dir / f"{split}.jsonl"
        payload = "".join(json.dumps(case, ensure_ascii=False, separators=(",", ":")) + "\n"
                          for case in cases if case["split"] == split)
        path.write_text(payload)
        hashes[split] = hashlib.sha256(payload.encode()).hexdigest()
    manifest = {"version": 1, "kind": "v5_lexical_gap_paraphrase_probe",
                "entities_per_split": args.entities_per_split,
                "cases_per_split": args.entities_per_split * 2,
                "source_sha256": {name: hashlib.sha256(path.read_bytes()).hexdigest()
                                  for name, path in (("development", args.development),
                                                     ("calibration", args.calibration))},
                "case_sha256": hashes,
                "label_provenance": "Copied from a positive v5 source case by entity and condition; paraphrases authored for this probe",
                "query_phrases": PHRASES}
    (args.output_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"cases": len(cases), "case_sha256": hashes}))


if __name__ == "__main__":
    main()
