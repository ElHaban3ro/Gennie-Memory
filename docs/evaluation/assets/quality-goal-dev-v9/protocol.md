# V9 development bank protocol

Status: independently reviewed once, locally executed for retrieval only. Synthetic development only. This bank is disjoint by case text and corpus from V6, V7, and V8. It is not a holdout and must not be described as confirmatory. See `blind-adjudication.md` for review scope and the explicit PO-48R/PO-NS-12 boundary.

The bank has 16 questions across four synthetic conversations and four relation classes (four each): `supports_answer`, `refutes_premise`, `pending_or_partial`, and `unrelated`. It deliberately includes queries without the stable ID repeated in the question, two-slot questions, explicit negation, preliminary values, corrections, and same-person/same-conversation distractors. `blind-review.jsonl` contains only question and candidate texts. `campaign-cases.jsonl` is the campaign adapter; `search-batch-cases.jsonl` is the local FTS preflight input. The key is kept separately in `answer-key.json`; do not send labels, expected IDs, or rationales to any classifier.

Before any provider call, a reviewer must independently label all rows from `blind-review.jsonl`, with relation, useful memory IDs, covered slots, and ambiguity. Reconcile disagreements prospectively without changing the source text. Freeze all four data hashes and the reviewer output. If any row is ambiguous or mislabeled, retain the original package and create a separately versioned correction; do not tune on the blind reviewer output.

`expected_ids` names evidence that contributes to the requested relation/slot, including explicit partials and refutations; topical similarity alone does not qualify. A separate order, person, location, or time is not a match unless the text connects it to the requested one. The independent reviewer flagged one PO contrast; the adjudication retains only the record about the exact PO named by each query. The first gate is local: verify JSON structure, 16 unique IDs, four rows per relation, all expected IDs present in the corpus, no cross-scope IDs, and search candidate recall. Test selector behavior separately from extractor behavior. A test-created trace can establish only selector structure. It cannot establish that a model can produce a faithful trace. Do not run `slot_evidence_v3` campaigns unless its existing three-consecutive-smoke gate is passed on a separate smoke set; the previously failed V3 smokes remain failed.

Potential next campaign, only after the local gate: Decisions + `baseline` against one frozen candidate configuration on these development cases, with labels omitted from requests, complete receipts, a hard call/cost limit, and no automatic retries. This remains development. A separate fresh blind cohort is required for step 4. Preserve every loss, extra, unresolved case, and technical failure; do not adopt without the pre-existing zero-loss/zero-extra quality gate and resource limits.

SHA-256:

| File | SHA-256 |
| --- | --- |
| `memory-corpus.json` | `12a874f5d9f39fdd55239e75cfc8ca6f0786e884c4d4627d1f47b569500e04b8` |
| `queries.jsonl` | `ec348d4e2711896175d72bfe673fe44d9003c9c50158872ae8e660d14c0be4b4` |
| `blind-review.jsonl` | `ddf83c34de67ece7c207a3e08762ef9e4d9ebb049b6ee7b610e9c10da2331524` |
| `answer-key.json` | `90bd04f8ae08f0f85205bf1edf17ddebf9cea535b2064ad50dc989694d8c5376` |
| `campaign-cases.jsonl` | `cbd0d2cb95f54b4351532a5e24abc8e004167459ef8611101fdbfa01f56b2581` |
| `search-batch-cases.jsonl` | `ec348d4e2711896175d72bfe673fe44d9003c9c50158872ae8e660d14c0be4b4` |
