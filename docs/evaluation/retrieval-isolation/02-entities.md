# 2. Entidades y escritura

Fecha: 2026-10-24. Estado: verificado. [Índice](README.md).

Objetivo: registrar identidades persistentes por actor/ámbito; conservar alias y fuente declarada, no fusionar homónimos y enlazar un ID resuelto desde `mem-write`. Nombre coincidente, incluso único, devuelve `ambiguous`; elegir `entity_id` es una declaración explícita del operador. `create_new:true` permite registrar otro homónimo.

Ejecutar:

```sh
cargo test --offline -p gennie-memory-retrieval isolation_tests
```

Controles relevantes:

- `entity_aliases_homonyms_and_operation_replays_preserve_identity`: alta, reintento, alias, homónimos, conflicto y evidencia.
- `entity_ids_and_candidates_never_cross_actor_scope_or_kind`: rechaza IDs ajenos; no expone candidatos fuera del filtro.
- `resolved_entity_round_trip_and_exact_dedup_keep_distinct_homonyms`: resolver → escribir → buscar; misma entidad deduplica, entidades distintas conservan memorias distintas; revisar conserva identidad.
- `write_rejects_unknown_entities_and_other_actor_without_side_effects`: fallo cerrado y escritura personal `local`.
- `operation_ids_cannot_be_reused_across_entity_and_memory_writes`: UUID compartido por dos clases de operación se rechaza.
- `entity_failure_rolls_back_identity_alias_operation_and_evidence`: fallo inducido al insertar evidencia; quedan cero filas en las cuatro tablas.
- `legacy_write_receipts_keep_their_request_hash_and_replay_after_upgrade`: serialización anterior conservada cuando falta entidad.
- `fixture_import_after_a_write_cannot_leave_stale_exact_content_keys`: importación posterior también participa en deduplicación.

Resultado: pasan los ocho controles. Se conservan las pruebas previas de nuevas fuentes, negación, ámbitos e historial. Las huellas incluyen actor/entidad; las anteriores se renuevan de forma perezosa y transaccional. Solo se recalcula contenido que no tenga clave vigente.

Límites: el origen permanece `declared_unverified`; no hay extracción automática de nombres, correferencia de pronombres, verificación externa ni fusión semántica.
