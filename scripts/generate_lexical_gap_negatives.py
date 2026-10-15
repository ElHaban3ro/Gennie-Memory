#!/usr/bin/env python3
"""Add labeled no-recall and unknown-mode controls to the v5 lexical-gap probe."""

import argparse
import hashlib
import json
from pathlib import Path


PHRASES = {
    "mode_unspecified": (
        "Could be video call or room gathering; which saved setup applies before choosing?",
        "Maybe webcam session, maybe office meetup; no format selected yet. Any setup note?",
        "Which preparation rule applies while online versus onsite remains undecided?",
        "Should I use a virtual huddle or shared room? Format still open.",
        "Before choosing camera link or physical room, what setup guidance is valid?",
        "Either teleconference or face to face is possible; no decision has been made.",
        "Channel undecided: webcam or same room. Which note applies?",
        "Could be telepresence or office workshop; still unconfirmed.",
        "Plan has no channel yet: video link or lobby gathering.",
        "Unsure whether screen sharing or chairs in a room; what setup note?",
    ),
    "explicit_no_recall": (
        "Ignore archived setup notes for this request; use current text only.",
        "Do not retrieve saved preferences; answer without stored context.",
        "Leave memory aside; respond using only this message.",
        "No saved instructions please; rely solely on present input.",
        "Skip historical notes; write a generic response.",
        "Do not consult retained guidance; continue without it.",
        "Treat all stored reminders as unavailable for this reply.",
        "Bypass memory lookup; use only what I just wrote.",
        "Please avoid prior project notes; provide a neutral sentence.",
        "Exclude archived context and saved rules from this reply.",
    ),
}


def derive(positives: dict[str, Path], all_queries: Path) -> dict[str, list[dict]]:
    wanted = {}
    for split, path in positives.items():
        cases = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
        entities = sorted({case["request"]["entity_id"] for case in cases})
        if (len(cases) != len(entities) * 2 or
            any(case["split"] != split or not case["expected_ids"] for case in cases)):
            raise ValueError("Positive probe must contain two labeled cases per entity")
        wanted[split] = entities
    indexed = {}
    for line in all_queries.read_text().splitlines():
        if not line.strip():
            continue
        case = json.loads(line)
        split = case["split"]
        if (split not in wanted or case["request"].get("entity_id") not in wanted[split] or
            case["segment"] not in PHRASES):
            continue
        key = (split, case["request"]["entity_id"], case["segment"])
        if key in indexed or case["expected_ids"] or case["label_source"] != "deterministic_fixture_rule":
            raise ValueError("Negative source case is duplicate or mislabeled")
        indexed[key] = case
    result = {}
    for split, entities in wanted.items():
        result[split] = []
        for position, entity in enumerate(entities):
            for segment in PHRASES:
                source = indexed.get((split, entity, segment))
                if source is None:
                    raise ValueError(f"Missing negative source for {split}/{entity}/{segment}")
                case = {**source, "id": f"lexgap_{split}_{entity}_{segment}",
                        "segment": f"lexical_gap_{segment}",
                        "request": {**source["request"],
                                    "query": PHRASES[segment][position % len(PHRASES[segment])]},
                        "derived_from_case": source["id"]}
                result[split].append(case)
        result[split].sort(key=lambda case: case["id"])
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--development", type=Path, required=True)
    parser.add_argument("--calibration", type=Path, required=True)
    parser.add_argument("--all-queries", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    if args.output_dir.resolve() in {path.resolve() for path in
                                     (args.development, args.calibration, args.all_queries)}:
        parser.error("Output directory must not overwrite an input")
    positives = {"development": args.development, "calibration": args.calibration}
    result = derive(positives, args.all_queries)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    hashes = {}
    for split, cases in result.items():
        payload = "".join(json.dumps(case, ensure_ascii=False, separators=(",", ":")) + "\n"
                          for case in cases)
        (args.output_dir / f"negatives-{split}.jsonl").write_text(payload)
        hashes[split] = hashlib.sha256(payload.encode()).hexdigest()
    manifest = {"version": 1, "kind": "v5_lexical_gap_negative_controls",
                "cases_per_split": {split: len(cases) for split, cases in result.items()},
                "source_sha256": {name: hashlib.sha256(path.read_bytes()).hexdigest()
                                  for name, path in {**positives, "all_queries": args.all_queries}.items()},
                "case_sha256": hashes,
                "label_provenance": "Copied from deterministic v5 mode_unspecified and explicit_no_recall cases by split and entity; paraphrases authored for this probe",
                "query_phrases": PHRASES}
    (args.output_dir / "negative-manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"cases": sum(len(cases) for cases in result.values()),
                      "case_sha256": hashes}))


if __name__ == "__main__":
    main()
