# 3. FTS5 y cobertura

Fecha: 2026-10-24. Estado: verificado. [Índice](README.md).

Ejecutar:

```sh
cargo test --offline -p gennie-memory-retrieval isolation_tests
cargo test --offline -p gennie-memory-retrieval product::tests
```

| Control | Expectativa y resultado |
| --- | --- |
| `phrase_queries_keep_order_accents_and_treat_operators_as_data` | Frase contigua, acentos, ambos modos, comilla sin cerrar y sintaxis hostil: pasa |
| `bm25_weights_change_ranking_but_not_eligibility` | Cambiar pesos invierte el orden esperado; pesos inválidos se rechazan: pasa |
| `coverage_separates_eligible_indexed_matched_limited_and_empty_queries` | 3 elegibles, 3 proyectados, 2 coincidencias, 1 resultado; diferencia límite, vacío e índice parcial: pasa |
| `fts_projection_updates_deletes_and_current_revision_are_consistent` | Inserción, actualización y borrado conservan integridad FTS; revisión antigua excluida: pasa |
| `hybrid_fusion_returns_one_current_memory_per_identity_despite_two_ranks` | El ID/revisión aparece una vez con ambos rangos: pasa |
| `missing_text_projection_degrades_product_before_classification` | Proyección ausente degrada sin llamar clasificador: pasa |
| `isolation_filters_reach_product_and_empty_normalization_is_not_no_candidates` | Vacío normalizado informa `not_evaluated`: pasa |

Iteraciones: se corrigió el conteo SQLite leyendo enteros `i64` antes de convertirlos a contadores Rust. Se añadió el trigger de actualización que faltaba en la proyección FTS. Conteos y candidatos usan la misma instantánea de lectura; el híbrido conserva esa instantánea durante ambos rankings.

La cobertura del índice se refiere a la proyección de revisiones elegibles. La integridad física del FTS se comprobó con `integrity-check` en los controles de actualización/borrado; no se ejecuta una auditoría física completa en cada consulta. Ninguna cobertura textual prueba completitud semántica, verdad o cobertura de la respuesta final.
