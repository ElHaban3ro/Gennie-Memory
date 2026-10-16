"""Typed-choice formulations for synthetic personal and multi-scope memories."""

VARIANTS = ("direct", "grounded", "spanish", "relation", "explicit", "relation_context")
INSTRUCTIONS = {
    "direct": (
        "Select the single saved memory that directly answers the user's question. "
        "Choose none if no listed memory supplies the answer. Do not guess."
    ),
    "grounded": (
        "Identify the fact requested by the user, then choose the one listed memory that "
        "explicitly states that fact about the indicated person, project, conversation or agent. "
        "Choose none when the fact is absent. Do not infer an answer from a related memory."
    ),
    "spanish": (
        "Elige el único recuerdo que responde directamente a la pregunta sobre la persona, "
        "proyecto, conversación o agente indicado. Elige none si ningún recuerdo contiene "
        "el dato solicitado. No inventes ni deduzcas la respuesta."
    ),
    "relation": (
        "Choose the one memory whose stated action or fact answers the question. "
        "Keep mentioned, attached, completed, pending and requested distinct. "
        "Choose none only when no listed memory answers it."
    ),
    "relation_context": (
        "Choose the one memory whose stated action or fact answers the question. "
        "Keep mentioned, attached, completed, pending and requested distinct. "
        "Use a listed condition only to check applicability. Choose none if nothing answers."
    ),
    "explicit": (
        "Match the question to one explicit fact in the memories. "
        "A memory may answer even when the wording differs. "
        "Choose none if every listed fact is about something else."
    ),
}


def choice(candidates: list[dict], variant: str, include_none: bool = True) -> tuple[dict, dict[str, str]]:
    if variant not in VARIANTS or len(candidates) > 20:
        raise ValueError("Invalid general-memory choice configuration")
    ids = [item["memory_id"] for item in candidates]
    if len(ids) != len(set(ids)) or "none" in ids:
        raise ValueError("Duplicate or reserved candidate ID")
    criteria = {}
    mapping = {}
    for index, item in enumerate(candidates):
        label = chr(ord("A") + index)
        condition = item.get("conditions")
        criteria[label] = (f"{item['summary']} [Condition: {condition}]"
                           if variant == "relation_context" and condition else item["summary"])
        mapping[label] = item["memory_id"]
    if include_none:
        criteria["none"] = ("Ningún recuerdo contiene el dato solicitado." if variant == "spanish"
                            else "No listed memory answers this question." if variant in ("relation", "explicit", "relation_context")
                            else "No listed memory contains the requested fact.")
        mapping["none"] = "none"
    return {"type": "choice", "instructions": INSTRUCTIONS[variant], "criteria": criteria}, mapping
