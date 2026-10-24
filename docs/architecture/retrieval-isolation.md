# Contrato de recuperación y aislamiento local

Fecha: 2026-10-24. Implementación: `apps/retrieval`; adaptadores CLI/JSONL: `bootstrap`. [Plan y pruebas](retrieval-isolation-plan.md).

## Consulta

`SearchRequest` conserva sus campos y añade:

| Campo | Contrato |
| --- | --- |
| `actor_id` | Opcional; omitido/null significa `local`. ID opaco de 1–128 bytes sin controles. Es un filtro simulado del operador, no autenticación. |
| `memory_types` | Hasta 32 tipos no vacíos de 1–128 bytes; lista omitida/vacía no restringe tipos. Los tipos se comparan literalmente, también los de fixtures históricos. |
| `bm25_weights` | Dos números finitos entre 0 y 100, alguno positivo, en orden original/resumen. Omitido/null usa `[1.0,1.5]`. |

Actor, ámbito, entidad opcional, tipo, revisión vigente, estado activo y vigencia se filtran antes de limitar FTS5 o coseno/RRF. `list_scope` y composición conservan los mismos filtros. La conexión es compartida; conteos y resultados usan una instantánea SQLite consistente.

`natural` conserva términos ordinarios unidos por OR y `keywords` mantiene stopwords/prefijos existentes. En ambos modos, comillas dobles agrupan frases literales: `"azul cobalto"` no equivale a dos términos OR. La comilla abierta se cierra al final. Cada unidad se convierte a una cadena FTS entrecomillada y SQL recibe parámetros; operadores escritos por el usuario no se ejecutan como sintaxis. El índice conserva `unicode61 remove_diacritics 2` y posiciones. Un resultado aparece una vez por memoria/revisión; dos identidades con el mismo contenido se conservan por separado.

## Cobertura

`SearchResponse.coverage` añade conteos `eligible_memories`, `indexed_memories` (documentos proyectados), `matched_memories`, `returned_memories` y `truncated`. `fts_index` distingue `complete|partial|missing`; cero elegibles tiene cobertura vacíamente completa. `query` distingue `executed|empty_after_normalization`. `retrieval` distingue `complete|limited|incomplete_index|not_evaluated`.

Un índice incompleto produce `status:degraded`. `recall-v1` detiene la clasificación, devuelve `fts_index_incomplete` y no llama al proveedor. Si la consulta no conserva términos buscables, devuelve `not_evaluated` con `query_empty_after_normalization`, en lugar de afirmar ausencia de recuerdos. El contrato heredado conserva sus campos; consultar `coverage.query` para interpretar su lista vacía.

El híbrido añade `text_coverage`, `rank_limit:20` y `candidate_pool_truncated`, conservando los conteos semánticos elegibles/indexados y la fusión única por `(memory_id,revision)`. La completitud del índice no mide cobertura de evidencia, relevancia ni verdad. Estos conteos no auditan corrupción arbitraria del índice FTS físico.

## Identidades locales

`entity-resolve <request.json>` y session `op:entity_resolve` aceptan:

```json
{"schema_version":1,"operation_id":"123e4567-e89b-42d3-a456-000000000001","scope":{"kind":"project","id":"demo"},"kind":"person","label":"Ana","source":{"kind":"document","ref":"synthetic-people"}}
```

Tipos: `person|place|event|company|subject`. El nombre admite 1–200 caracteres sin controles; para comparar alias se colapsan espacios y se pasa a minúsculas, sin equivalencias fonéticas o eliminación de acentos. `actor_id` es opcional con el mismo valor local predeterminado. La salida incluye `schema_version`, `status`, `entity_id`, `candidates`, `replayed` y `provenance_status`.

- Sin coincidencias: `created`, con ID generado y persistido.
- Con uno o varios candidatos del mismo tipo/actor/ámbito: `ambiguous`, sin ID elegido. Nunca se fusiona por nombre.
- Con `entity_id` explícito disponible y tipo compatible: `resolved`; el nombre aportado queda como alias de esa identidad.
- `create_new:true` declara otra identidad y permite un homónimo; no puede combinarse con `entity_id`.

Máximo 100 candidatos; superar ese máximo devuelve `entity_candidates_limit` sin lista parcial ni commit. La operación UUID se registra con hash y respuesta confirmada; repetirla reproduce el recibo, cambiarla produce `idempotency_conflict`. Resolver y escribir no pueden reutilizar el mismo UUID. Alias, identidad, evidencia declarada y recibo se confirman en una transacción. Los recibos ambiguos son históricos: volver a evaluar un registro modificado requiere otro UUID.

La migración 8 conserva IDs anteriores como `subject` con su ID por nombre, sin inventar un nombre real o tipo. Fixtures posteriores registran sus IDs con esa misma política. No hay extracción automática de referencias en la pregunta; el llamante usa el ID resuelto como filtro.

## Escritura y conservación

`mem-write` admite `entity_id` opcional en el nivel superior. Debe existir para actor `local` y el ámbito de escritura. `revise` conserva la entidad si se omite; cambiarla explícitamente falla. No acepta actor por JSON: la escritura personal sigue vinculada a `local`, sin ACL de host.

Las huellas exactas incorporan actor/entidad sin modificar los hashes históricos de operaciones sin entidad. Las claves derivadas anteriores se marcan con versión 0 y se renuevan al escribir; también se retiran claves de revisiones obsoletas y se indexan memorias importadas sin recalcular las ya vigentes. Las fuentes adicionales y el historial permanecen separados de la identidad y no se fusionan semánticamente.

La migración es aditiva y conserva las versiones 1–7. Añade actor, tablas de entidades/alias/evidencia/operaciones, versión de claves e índices y trigger de actualización de FTS. No añade dependencia, recurso de red, secreto, servidor ni segunda conexión. Rendimiento sostenido y autenticación futura requieren validación propia.
