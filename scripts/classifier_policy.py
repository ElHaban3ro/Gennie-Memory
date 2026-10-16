"""Validated classifier profile for synthetic evaluation runners."""

import json
from pathlib import Path


DEFAULT_PROFILE = Path(__file__).resolve().parents[1] / "config/classifier-profile-v1.json"
SUPPORTED_MODELS = {"openai/gpt-6-luna-decisions", "typesafe/jev-1.13"}


def load_profile(path: Path = DEFAULT_PROFILE) -> dict:
    profile = json.loads(path.read_text())
    expected = {"version", "retrieval_method", "embedding_encoder", "max_candidates",
                "question_type", "prompt_variant", "allow_none", "primary_model",
                "secondary_model", "secondary_enabled", "automatic_fallback"}
    if not isinstance(profile, dict) or set(profile) != expected:
        raise ValueError("Invalid classifier profile fields")
    if (type(profile["version"]) is not int or profile["version"] != 1 or
            profile["retrieval_method"] != "fts5_semantic_rrf" or
            profile["embedding_encoder"] != "laya_local" or
            type(profile["max_candidates"]) is not int or
            not 1 <= profile["max_candidates"] <= 20 or
            profile["question_type"] != "choice" or
            profile["prompt_variant"] != "direct" or
            profile["allow_none"] is not True or
            profile["primary_model"] not in SUPPORTED_MODELS or
            profile["secondary_model"] not in SUPPORTED_MODELS or
            profile["primary_model"] == profile["secondary_model"] or
            type(profile["secondary_enabled"]) is not bool or
            profile["automatic_fallback"] is not False):
        raise ValueError("Unsupported classifier profile")
    return profile


def selected_model(profile: dict, role: str) -> str:
    if role == "primary":
        return profile["primary_model"]
    if role == "secondary" and profile["secondary_enabled"]:
        return profile["secondary_model"]
    raise ValueError("Secondary classifier is disabled or role is invalid")
