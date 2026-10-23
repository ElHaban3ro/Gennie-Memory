# V9 independent blind review and adjudication

The reviewer received only `blind-review.jsonl` at SHA-256 `ddf83c34de67ece7c207a3e08762ef9e4d9ebb049b6ee7b610e9c10da2331524`. It contained question text and candidate IDs/text, but no relation labels, expected IDs, rationale, or model results. The reviewer did not use a provider or edit the bank. This was one Codex subagent review from the same model family, not a human review or independent-family replication.

## Case-level outcome

| Case | Adjudicated useful evidence | Relation and requested slots |
| --- | --- | --- |
| `v9_field_q01` | `v9_field_01` | Supports altitude at Alta station (2,140 m); query omits the stable person ID. |
| `v9_field_q02` | `v9_field_02` | Refutes that Diego calibrated the sensor. |
| `v9_field_q03` | `v9_field_03` | Partial: 30 cm forecast; actual snow measurement is absent. |
| `v9_field_q04` | None | Unrelated to the requested West Bridge altitude. |
| `v9_proc_q01` | `v9_proc_01` | Supports supplier named on PO-48R (Cedarline Works). |
| `v9_proc_q02` | `v9_proc_02` | Refutes whether PO-NS-12 was signed with Northstar Supply. |
| `v9_proc_q03` | `v9_proc_03` | Partial: estimated 18 November; date not confirmed. |
| `v9_proc_q04` | None | Invoice currency is absent. |
| `v9_transit_q01` | `v9_transit_01` | Supports both requested slots: route 6B and Alder Street stop assigned to Nia. |
| `v9_transit_q02` | `v9_transit_02` | Refutes taxi booking; names the shared shuttle instead. |
| `v9_transit_q03` | `v9_transit_03` | Partial: 08:40 is a conditional prediction, not actual arrival. |
| `v9_transit_q04` | None | Return-trip playlist is absent; a window-seat preference does not answer it. |
| `v9_incident_q01` | `v9_incident_01` | Supports both slots: Mara restored power to panel K-2 at 14:10. |
| `v9_incident_q02` | `v9_incident_02` | Refutes Mara resetting breaker B-4; Omar did it. |
| `v9_incident_q03` | `v9_incident_03` | Partial: loose terminal is a preliminary, unconfirmed cause. |
| `v9_incident_q04` | None | Warning-light color is absent. |

The review supports the frozen key's primary relation and direct evidence IDs for all 16 queries. It also raised a possible partial-context interpretation for `v9_proc_02` on `v9_proc_q01`: that memory excludes Northstar from a *different* order, PO-NS-12. The frozen query asks for the supplier named on PO-48R, so the adjudication treats the distinct purchase-order ID as a scope boundary and does not count the Northstar statement as evidence for PO-48R. Conversely, PO-48R does not establish whether PO-NS-12 was signed. This boundary must remain explicit in any later scorer; do not infer that vendor exclusivity applies across purchase orders.

All four negative controls were judged to have no useful candidate. The review also confirmed that `v9_transit_q04` is a clean near-topic negative after it was changed to ask for a playlist. The bank has one same-memory two-slot query; it does not test combining multiple memories to fill multiple slots.

## Scope and limitations

This establishes a blind, single-review adjudication for a small directed development set. It does not estimate population quality. The 16 cases use English, four synthetic domains, one principal relation per query, and several repeated four-case patterns. No model has been evaluated, and no labels were provided to a model. Local FTS retrieves all 12 expected useful IDs in top 5 and top 1, while returning candidates on all four unrelated controls; this tests only retrieval coverage and motivates a selector gate.

Do not use V9 to tune V3's holdout. It may be used as development after its file hashes are rechecked. A future fresh cohort remains necessary for confirmation.
