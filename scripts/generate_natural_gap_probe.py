#!/usr/bin/env python3
"""Turn the v5 lexical-gap cases into complete synthetic user requests."""

import argparse
import hashlib
import json
from pathlib import Path


PROMPTS = {
    "lexical_gap_remote": (
        "For an upcoming video call, which saved preparation rule should I follow?",
        "Before a webcam session with this team, remind me of the established setup preference.",
        "I am joining by video link. What standing preparation instruction applies?",
        "During an online gathering, what prior working preference should guide my setup?",
        "I am planning a teleconference. Which saved instruction is relevant?",
        "For a camera based discussion, what established preparation habit matters?",
        "We will connect through screens. Which retained setup rule should I use?",
        "Ahead of a virtual team session, remind me what preparation was agreed.",
        "What saved guidance applies when everyone joins through a video link?",
        "I will host an online group call. Which existing setup preference applies?",
    ),
    "lexical_gap_onsite": (
        "For our next face to face gathering, which saved preparation rule applies?",
        "When everyone is in the same room, what established setup preference should I recall?",
        "Before an in person session, remind me of the relevant working instruction.",
        "I am planning an onsite gathering. What prior rule should guide the opening?",
        "At a physical group session, which stored preparation preference matters?",
        "What saved guidance applies when the whole team shares one room?",
        "We will gather around a table. Which existing setup rule should I follow?",
        "Ahead of a face to face team session, remind me what preparation was agreed.",
        "For an in person workshop, which retained working preference applies?",
        "I will host everyone at the office. What standing setup instruction matters?",
    ),
    "lexical_gap_mode_unspecified": (
        "I have not chosen between a video call and being in the same room. Which conditional rule applies now?",
        "The format is undecided: webcam or face to face. Should any stored setup preference be used yet?",
        "We might connect online or meet at the office. Which saved instruction is applicable before deciding?",
        "No channel has been selected. Can you already apply either preparation rule?",
        "The team could gather physically or through screens. What guidance applies while that remains open?",
        "I have two possible formats but have selected neither. Is any conditional setup note valid now?",
        "Before choosing virtual versus onsite, which existing preparation preference should be followed?",
        "Whether we use a camera link or one room is still unknown. Does a specific rule apply?",
        "The meeting format is still pending. Should you recall either conditional instruction?",
        "We have not decided how to gather. Is there an applicable saved setup rule already?",
    ),
    "lexical_gap_explicit_no_recall": (
        "Please ignore saved memories for this reply and write a neutral sentence.",
        "Do not consult stored preferences; answer only from my current message.",
        "Leave prior notes aside for now. Give a generic response without recalling them.",
        "For this turn, skip memory lookup and respond without archived guidance.",
        "I explicitly do not want saved instructions used in your next answer.",
        "Set aside retained context and write only a neutral reply.",
        "Please avoid all prior project notes while answering this request.",
        "Answer without retrieving anything previously stored about this work.",
        "Do not bring earlier preferences into the response I am requesting now.",
        "Ignore historical context; produce a short general sentence instead.",
    ),
}


def derive(source_dir: Path) -> dict[str, list[dict]]:
    result = {}
    for split in ("development", "calibration"):
        positives = [json.loads(line) for line in (source_dir / f"{split}.jsonl").read_text().splitlines()]
        negatives = [json.loads(line) for line in
                     (source_dir / f"negatives-{split}.jsonl").read_text().splitlines()]
        rows = positives + negatives
        entities = sorted({case["request"]["entity_id"] for case in rows})
        if len(rows) != 200 or len(entities) != 50 or any(case["split"] != split for case in rows):
            raise ValueError("Expected 200 labeled source cases on 50 entities per split")
        position = {entity: index for index, entity in enumerate(entities)}
        output = []
        for source in rows:
            segment = source["segment"]
            if segment not in PROMPTS:
                raise ValueError(f"Unexpected segment: {segment}")
            entity = source["request"]["entity_id"]
            mode_prompt = PROMPTS[segment][position[entity] % len(PROMPTS[segment])]
            output.append({**source, "id": source["id"].replace("lexgap_", "natgap_", 1),
                           "segment": segment.replace("lexical_gap_", "natural_gap_", 1),
                           "request": {**source["request"], "query": mode_prompt},
                           "v5_source_case": source["derived_from_case"],
                           "derived_from_case": source["id"]})
        output.sort(key=lambda case: case["id"])
        if len({case["id"] for case in output}) != 200:
            raise ValueError("Duplicate derived case ID")
        result[split] = output
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    if args.source_dir.resolve() == args.output_dir.resolve():
        parser.error("Output directory must differ from source")
    rows = derive(args.source_dir)
    args.output_dir.mkdir(parents=True, exist_ok=True)
    hashes = {}
    for split, cases in rows.items():
        payload = "".join(json.dumps(case, ensure_ascii=False, separators=(",", ":")) + "\n"
                          for case in cases)
        (args.output_dir / f"{split}.jsonl").write_text(payload)
        hashes[split] = hashlib.sha256(payload.encode()).hexdigest()
    manifest = {"version": 1, "kind": "v5_natural_lexical_gap_probe",
                "cases_per_split": 200, "positive_per_split": 100, "negative_per_split": 100,
                "source_sha256": {name: hashlib.sha256((args.source_dir / name).read_bytes()).hexdigest()
                                  for name in ("development.jsonl", "calibration.jsonl",
                                               "negatives-development.jsonl",
                                               "negatives-calibration.jsonl")},
                "case_sha256": hashes,
                "label_provenance": "Derived from v5 deterministic labels via the lexical-gap probe; natural English paraphrases authored for this probe",
                "prompts": PROMPTS}
    (args.output_dir / "manifest.json").write_text(
        json.dumps(manifest, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"cases": sum(len(value) for value in rows.values()),
                      "case_sha256": hashes}))


if __name__ == "__main__":
    main()
