# V9 local retrieval preflight

Status: **development-only retrieval preflight**. One independent blind review confirmed the primary evidence/relation for 16/16 queries; it was a Codex agent of the same model family, not human or independent-family validation. At the time of this search preflight no provider had been called; the later paired selector diagnostic is recorded separately in [V9 provider results](../../quality-goal-dev-v9-provider-diagnostic-2026-10-23.md).

The current Rust CLI loaded 16 synthetic memories into a dedicated SQLite database and ran the 16 frozen FTS5 queries through `search-batch`. The workspace binary was rebuilt from current sources by `cargo run --offline -p iromem -- init-db`; compilation succeeded with three existing dead-code warnings in the experimental slot-evidence adapter.

| Relation | Queries | Expected useful memory IDs | Retrieved top 1 | Retrieved top 5 |
| --- | ---: | ---: | ---: | ---: |
| Supports answer | 4 | 4 | 4/4 | 4/4 |
| Refutes premise | 4 | 4 | 4/4 | 4/4 |
| Pending or partial | 4 | 4 | 4/4 | 4/4 |
| Unrelated | 4 | — | Candidates in 4/4 | — |

All **12/12 draft-key useful IDs** appear in top 5 and top 1. The search call took a median of **0.256 ms**, with a maximum of **0.550 ms** across 16 local cases. These timings exclude classification and composition. FTS returned candidates on every unrelated query, so retrieval alone does not abstain.

This establishes only that the current local search route can supply the draft-key evidence. It does not evaluate generated citations or response fidelity. The full local output is `search-results.jsonl`; the compact counts are in `search-report.json`. V3 remains experimental and its earlier smoke failures remain unchanged.
