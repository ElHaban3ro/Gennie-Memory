"""Frozen synthetic candidate descriptions and decision questions for matrix pilots."""

import json


CHOICE_VARIANTS = ("plain", "conditional", "structured")
CHOICE_INSTRUCTIONS = {
    "plain": (
        "Choose the one current memory that directly answers the user's project and task. "
        "Choose none when no memory applies or the user says not to use memories. "
        "Use only the listed memories."
    ),
    "conditional": (
        "Choose exactly one memory only when its project, task and every stated condition "
        "are established by the user's request. If the task or condition is unknown, "
        "or the user prohibits recall, choose none. Do not infer a missing condition."
    ),
    "structured": (
        "Evaluate this request in order: first check whether recall is allowed; then match "
        "the project and task; then verify the meeting modality and other stated conditions. "
        "Choose the single applicable option, or none if any required detail is missing. "
        "Return only the typed choice."
    ),
}


def choice(candidates: list[dict], variant: str) -> tuple[dict, dict[str, str]]:
    if variant not in CHOICE_VARIANTS:
        raise ValueError(f"Unknown prompt variant: {variant}")
    if len(candidates) > 20:
        raise ValueError("At most twenty candidates may be presented")
    identifiers = [item["memory_id"] for item in candidates]
    if "none" in identifiers or len(identifiers) != len(set(identifiers)):
        raise ValueError("Duplicate or reserved candidate ID")
    criteria = {}
    mapping = {}
    for index, candidate in enumerate(candidates):
        label = chr(ord("A") + index)
        summary = candidate["summary"]
        condition = candidate.get("conditions")
        if variant == "plain":
            description = summary
        elif variant == "conditional":
            description = summary + (f"\nApplies when: {condition}" if condition else "")
        else:
            description = json.dumps({"memory": summary, "condition": condition},
                                     ensure_ascii=False, separators=(",", ":"))
        criteria[label] = description
        mapping[label] = candidate["memory_id"]
    criteria["none"] = "No listed memory applies to the current request."
    mapping["none"] = "none"
    return {"type": "choice", "instructions": CHOICE_INSTRUCTIONS[variant],
            "criteria": criteria}, mapping
