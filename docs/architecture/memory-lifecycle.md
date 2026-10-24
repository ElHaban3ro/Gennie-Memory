# Ciclo de vida local

Estado: implementación local, 2026-10-24. Propietario: `retrieval::lifecycle`; migración 12. Superficie de operador local, actor `local` implícito, sin autenticación multiusuario. No inicia un daemon ni cambia la ruta predeterminada de recall. Los índices E5 siguen siendo investigación optativa.

## Operaciones

`mem-lifecycle solicitud.json` y session `mem_lifecycle` aceptan:

```json
{"schema_version":1,"operation_id":"123e4567-e89b-42d3-a456-426614174001","action":"withdraw","memory_id":"memory-id","expected_revision":1,"scope":{"kind":"project","id":"demo"},"as_of":"2026-10-24T12:00:00.000Z","next_review_at":null,"evidence":{"original_text":"El operador solicitó retirar este recuerdo.","source":{"kind":"user","ref":"solicitud-local"}}}
```

Acciones: `withdraw`, `restore`, `delete`, `review`. Todas exigen evidencia de la solicitud (`original_text` y `source`); en el borrado sólo queda su hash dentro de la solicitud canónica. Operación UUID idempotente; reutilizarlo con otra solicitud falla. La transacción comprueba ámbito, actor y revisión vigente antes de cambiar nada. Retirada/restauración crean una revisión de control con el mismo contenido/fuente y avanzan el contador; no reescriben las revisiones anteriores: `memories.lifecycle_state` controla visibilidad en FTS, listado, cobertura, híbrido e inventario E5 antes del límite. Revisar una memoria retirada no la reactiva. Revisar contenido con mem-write tampoco puede restaurarla implícitamente. Restaurar conserva estado y fechas originales: no vuelve vigente un recuerdo caducado.

`review` requiere `next_review_at` UTC posterior a `as_of` y evidencia de la revisión. Registra la revisión declarada por el operador para la revisión actual, sin verificar verdad ni aplicabilidad semántica. La procedencia de esa evidencia se comprueba contra snapshots locales y se audita; confirmar contenido mediante una nueva revisión corresponde a `mem-write action=confirm`. Las fechas de validez del contenido se cambian mediante una nueva revisión de escritura. Una revisión de contenido reinicia el calendario; una revisión de control conserva su agenda. Una solicitud antigua con la revisión previa al control falla con `revision_conflict`. `as_of` es el tiempo lógico aportado por el operador, mientras que el recibo usa la hora UTC de SQLite.

`mem-review-list solicitud.json` y session `mem_review_list`:

```json
{"scope":{"kind":"project","id":"demo"},"as_of":"2026-10-24T12:00:00.000Z","limit":20,"after_id":null}
```

Devuelve IDs, revisión, estado, fechas y calendario; sin textos. Incluye recuerdos sin retirada y sin revisión programada, con revisión vencida, validez terminada o reconciliación pendiente. Expone estado derivado, motivos y `applicable_now`; una revisión vencida produce incertidumbre sin declarar falso el dato. Orden estable por ID; `next_after_id` permite continuar, con máximo 100 por solicitud. No es una instantánea persistente entre páginas: cambios concurrentes pueden alterar el conjunto.

## Borrado del almacén activo

`delete` elimina todas las revisiones, documentos FTS, vectores históricos, índices E5 completos que contienen la memoria, comprobaciones de fuente, claves derivadas, evidencias y propuestas vinculadas. Eliminar un índice completo evita anunciar cobertura total tras retirar solo algunos fragmentos. El ámbito suscrito se reencola para reconstruirlo.

Se conserva un recibo mínimo de operación con IDs/acción/revisión/hash y sin texto de memoria. Los snapshots de fuentes independientes permanecen: pueden respaldar otros recuerdos y no son la memoria eliminada. Esta operación no borra archivos de entrada, backups, exportaciones, logs externos ni garantiza sobrescritura forense de páginas SQLite/WAL. La restauración solo aplica a retirada; un recuerdo borrado no se restaura con `restore`.

## Cola persistente E5

`embedding-job solicitud.json` y session `embedding_job` usan:

```json
{"action":"enqueue","scope":{"kind":"project","id":"demo"},"space_id":"espacio-del-perfil-E5"}
```

Acciones: `enqueue`, `get`, `cancel`, `retry`. El ID de trabajo se deriva de actor/ámbito/espacio. `enqueue` suscribe explícitamente el ámbito/perfil y es idempotente; no descarga ni carga modelos. El estado inicial es `pending`. Las altas, nuevas revisiones, retiradas, restauraciones y borrados incrementan la generación y lo dejan pendiente dentro de su misma transacción. Cambios posteriores a una cancelación vuelven a dejar el ámbito pendiente; cancelar una generación no desactiva permanentemente la suscripción.

En `embedding-session` (feature `local-embeddings`, settings y artefactos locales existentes), `queue_run` consume un trabajo:

```json
{"op":"queue_run","request":{"scope":{"kind":"project","id":"demo"},"actor_id":"local","deadline_ms":30000}}
```

El espacio se toma del perfil cargado. Estados: `pending → running → completed|failed`; `cancelled` impide publicación; `retry` explícito permite repetir fallos/cancelaciones/completados. Un worker reclama atómicamente una concesión de diez minutos con token aleatorio e incrementa intentos. Una concesión abandonada se recupera al vencer con otro token. La generación, token, ámbito, actor, espacio y plazo se comprueban en la misma transacción que publica el índice completo. Una cancelación o cambio durante inferencia hace rollback de la publicación; el cómputo CPU ya iniciado termina bajo su plazo existente. No se promete interrupción instantánea del modelo. Un ámbito vacío termina con `completed_empty`, índice nulo y purga de índices anteriores de ese ámbito/espacio.

Los errores conservan un código acotado, no mensajes con contenido ni secretos. La concesión no se renueva: trabajos que superen diez minutos no publican y requieren recuperación. Los fragmentos y vectores se calculan fuera de transacciones; publicación y finalización son atómicas. La cola no procesa automáticamente ni introduce un servicio de fondo. Cerrar stdin finaliza la sesión; al reabrirla persisten los estados y concesiones en SQLite.

## Verificación

Las pruebas locales usan SQLite sintética aislada y un codificador falso para comprobar mecanismos de cola, transacciones y publicación; no acreditan calidad semántica ni rendimiento E5 real. La evidencia ejecutada se registra en el informe de entrega y no se extrapola a despliegue o integración Codex. La continuidad de contexto del agente tras compactación permanece pendiente, separada de la recuperación de trabajos persistentes.

Los UUID de escrituras heredadas borradas se conservan como marcas mínimas sin contenido ni FK a la memoria. Repetir una escritura borrada no vuelve a crear su texto; los recibos de reconciliación afectados se sustituyen por un recibo `content_deleted`. Una nueva escritura explícita con UUID nuevo sigue siendo posible.

## Contención de escritores (2026-10-24)

La escritura reconciliada y las transiciones de ciclo de vida reservan el escritor con `BEGIN IMMEDIATE` antes de sus lecturas transaccionales. Así el busy_timeout puede esperar un escritor ya reservado en lugar de fallar de inmediato al promover una transacción deferred. La llamada remota sigue fuera de transacción; replay/revalidación y commit conservan sus reglas. Las lecturas/colas no recibieron un cambio global de transacción. [Repro rojo, probe y verificación](../evaluation/recall-completion-v1/sqlite-contention-diagnosis.md).
