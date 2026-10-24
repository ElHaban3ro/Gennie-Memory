# Escritura y reconciliación local

Fecha: 2026-10-24. Núcleo: `retrieval::reconciliation`; persistencia compatible: `retrieval::write`; adaptadores CLI y JSONL. Esta entrega completa el recorrido local de propuestas y resolución. No implementa autenticación de host, ACL multiusuario ni equivalencia semántica calibrada.

## Superficies

| Comando CLI | Operación JSONL | Contrato |
| --- | --- | --- |
| `mem-write` | `mem_write` | Crear, revisar o confirmar; conserva el JSON de creación/revisión anterior. |
| `mem-write-batch` | — | Hasta 2500 solicitudes; commit independiente por fila, parada ante fallo y reintento idempotente. |
| `mem-resolve` | `mem_resolve` | Resolver propuesta con todas las revisiones esperadas. |
| `mem-proposal-get` | `mem_proposal_get` | Inspeccionar propuesta y juicios antes de decidir. |
| `mem-audit` | `mem_audit` | Evidencia y comprobaciones de una memoria autorizada; máximo 1000 entradas. |
| `mem-operation-get` | `mem_operation_get` | Consultar el recibo persistido dentro del ámbito local. |

Todas las rutas usan la única conexión del proceso, sin puerto. `schema_version:1`; UUID de operación; actor `local` implícito. No se puede enviar `actor_id`, hashes, puntuaciones o relaciones decididas por el cliente en `mem-write`. `entity-resolve` conserva su registro local de identidades y alias; un nombre coincidente devuelve candidatos, nunca fusiona personas. El filtro de actor sintético de herramientas de investigación no equivale a autenticación.

`create` exige `scope`, `memory` y permite `entity_id`, `topic_key`, `source_policy`, `relation_provider`. `revise` exige `memory_id`, `expected_revision`, `memory`; ámbito omitido se obtiene de la identidad existente. Un ámbito explícito debe coincidir. La entidad es inmutable. El contenido sigue siendo el contrato local previo: original, síntesis, tipo, fuente, condición y fechas; el resto de metadatos del diseño v0.1 sigue fuera de esta superficie y se rechaza.

La huella SHA-256 incluye actor/entidad, ámbito, original, síntesis, tipo, condiciones y vigencia. Se colapsan espacios para comparar, conservando mayúsculas, puntuación y negación. Una fuente distinta se agrega como evidencia sin cambiar identidad/contenido. Huella exacta no significa paráfrasis equivalente. `topic_key` es un tema de valor único, normalizado a minúsculas, hasta 160 caracteres, segmentos `/` no vacíos. No se deriva automáticamente de palabras similares. Cambiar el contenido de un tema existente abre propuesta; una revisión explícita conserva ese tema.

## Fuentes y clasificación

`source_policy` conserva `report` por defecto para compatibilidad de importación local: guarda y audita la procedencia como declarada si no puede verificarla. `require_verified` exige coincidencia literal en un snapshot del operador, ligado a actor, ámbito, versión, hash y rango UTF-8. Si falta, crea propuesta `unverified_source` sin memoria ni documento FTS. Registrar un snapshot no verifica un origen externo ni la verdad semántica. [Contrato de fuentes](recall-source-verification-plan.md).

`relation_provider` es optativo: `decisions`, `jev` o `jev114`. Su omisión no llama a red. La comparación se limita a los candidatos FTS5 de igual actor, ámbito, entidad cuando se aporta y tipo (máximo 20), más la identidad de tema explícita. No se presume exhaustividad semántica ni se integra E5. Una sola petición tipada compara el roster completo y conserva el recibo una vez. Sus categorías son `duplicate`, `complement`, `correction`, `exception`, `contradiction`, `unrelated`, además de `undetermined` para incertidumbre. Las citas se vinculan a textos originales; literalidad no garantiza el juicio semántico. Proveedor fallido, roster incompleto o citas inválidas conserva el contenido como propuesta y publica el diagnóstico, sin reintento automático. El adaptador limita el payload a 128 KiB y usa el plazo HTTP existente.

La llamada remota sucede fuera de la transacción. Dentro se revalidan revisiones, entidad y unicidad de tema. Ningún juicio del modelo sobrescribe ni fusiona automáticamente: una pareja candidata deja propuesta inspeccionable y requiere decisión explícita. La búsqueda FTS puede omitir paráfrasis; habilitar este clasificador no convierte la escritura en deduplicación semántica exhaustiva.

## Resolver y confirmar

`mem-resolve` exige UUID nuevo, `proposal_id`, `expected_targets:[{memory_id,revision}]`, `decision` y `evidence:{original_text,source}`. El conjunto debe contener todos los destinos, sin duplicados ni extras. Las versiones pueden actualizarse respecto de la propuesta, pero han de seguir vigentes al confirmar.

- `keep_existing` descarta y purga el contenido pendiente.
- `replace_existing` exige `target_memory_id` perteneciente al conjunto y `memory` completo; publica una revisión y relación de corrección, preservando identidad/entidad/tema.
- `create_separate` exige mismo `scope`, `memory` completo y, si había candidatos, condiciones que distingan el recuerdo. Un tema que colisione debe reemplazarse por uno distinto. La identidad exacta de otro recuerdo no se transforma en alta separada.

Aceptar exige evidencia literal tanto de la decisión como del contenido resultante, incluso si la propuesta usaba `report`. Una propuesta pendiente por fuente se resuelve con destinos vacíos tras registrar la fuente. Se purga la copia pendiente y se transfiere su evidencia al resultado, sin dejar texto en el recibo de operación. Los recibos conservan IDs, estados y hashes de procedencia. El descarte conserva la evidencia de su decisión, no el contenido descartado.

`confirm` exige identidad, revisión esperada y evidencia; rechaza `memory`, entidad, tema y clasificador. Crea revisión nueva copiando contenido y fuente original, añade evidencia separada y actualiza FTS/huella. `review_after` omitido conserva la agenda anterior, `null` la borra y una fecha UTC futura la reprograma. Una confirmación no resuelve una contradicción pendiente ni convierte una fuente declarada en verdad. El ciclo de vida mantiene retirada/restauración/borrado en `mem-lifecycle`; confirmar/revisar una memoria retirada falla.

## Atomicidad y auditoría

La migración aditiva 0011 agrega tema único, propuestas/destinos, relaciones, operaciones de reconciliación y auditoría. No cambia bytes históricos. El registro de entidades y el de reconciliación rechazan colisiones de UUID entre superficies; las escrituras antiguas conservan sus recibos. Reintentar una operación confirmada no vuelve a llamar al proveedor ni agrega evidencia. Un UUID con otro contenido produce `idempotency_conflict`.

Revisión, FTS, huella, relaciones, evidencia, propuesta y resultado se confirman en la misma transacción. Un fallo de auditoría revierte también memoria/FTS/operación. Auditoría almacena cita y fuente, hash de evidencia, actor/ámbito, revisión, acción, fecha UTC y comprobación literal con sus límites. `mem-audit` verifica el hash del payload al leer; no es una firma externa resistente a un administrador que modifique toda la base. El borrado activo de ciclo de vida purga contenido vinculado conforme a su contrato.

## Verificación

Las pruebas sintéticas enfocadas y CLI usan bases en memoria/temporales, proveedor falso o sin credencial y no llaman a red. Las suites del workspace verifican compatibilidad con fuentes, aislamiento y ciclo de vida. Los resultados finales se registran en el [informe de escritura](../evaluation/write-reconciliation/README.md). No se atribuye a estos controles calidad semántica de modelos, fuente externa verificada o despliegue.
