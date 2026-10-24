# 1. Filtros anteriores al límite

Fecha: 2026-10-24. Estado: verificado. [Índice](README.md).

Objetivo: ningún candidato de otro actor, ámbito, entidad, tipo o revisión supera el filtro aunque tenga mejor puntuación que el candidato permitido. Un intervalo incluye `valid_from` y excluye `valid_until`.

Ejecutar:

```sh
cargo test --offline -p gennie-memory-retrieval isolation_tests
```

Controles en `apps/retrieval/src/isolation_tests.rs`:

- `actor_type_and_scope_filters_precede_both_rankings_and_scope_list`: distractores textuales y semánticos, límite 1, listado local y actor desconocido.
- `composition_cannot_reintroduce_a_foreign_actor_or_type`: selección externa no puede reintroducir un ID excluido.
- `fixture_revisions_cannot_change_actor_and_failed_batch_rolls_back`: identidad inmutable y rollback del lote.
- `validation_and_time_boundaries_do_not_expand_the_result`: fecha exacta y rechazo de filtros vacíos.

Resultado: pasan los cuatro controles; pasan también las regresiones previas de entidad, revisión y vigencia. `product/tests.rs` comprueba propagación de actor/tipo y ausencia de llamadas ante una búsqueda vacía.

Iteración: la primera ejecución falló en 11 pruebas antiguas por crear manualmente un esquema sin la migración nueva. Se actualizó su preparación; después pasaron las 112 pruebas existentes de retrieval. Los filtros son simulados: el operador local puede elegir el actor, por lo que esto no acredita ACL multiusuario.
