# Verificación local del ciclo de vida v1

Fecha: 2026-10-24. Alcance: SQLite y CLI/JSONL en esta Mac, con datos sintéticos aislados. [Contrato](../../architecture/memory-lifecycle.md) y [decisión 0055](../../decisions/0055-local-memory-lifecycle.md).

## Controles de comportamiento

- Cinco pruebas `lifecycle::tests`: exclusión/restauración en búsqueda, revisiones de control, rechazo de revisión/ámbito incorrectos, calendario, generación de trabajo, concesión abandonada, rollback completo del borrado con fallo inyectado, limpieza de propuestas/evidencias/checks/recibos y bloqueo de reintento de escrituras borradas.
- Cuatro pruebas `local_embeddings::queue_tests` con codificador falso: publicación completa y ámbito vacío, cancelación entre preparación y publicación con rollback, concesión vencida, recuperación con otro token y rechazo de una generación vieja aunque el ámbito vuelva al mismo contenido.
- Una prueba `memory_lifecycle_cli`: persistencia entre procesos, cola/estado, idempotencia, retirada/restauración con revisiones esperadas, borrado y recuperación de una sesión JSONL tras entrada inválida.

Los montajes de once pruebas heredadas de servicio/embeddings se actualizaron para aplicar el registro real de migraciones, en lugar de esquemas SQL parciales. El primer workspace conjunto identificó la ausencia de la columna de ciclo de vida en esos montajes; el enfoque posterior de retrieval confirmó la corrección.

## Evidencia ejecutada

- `cargo test --offline -p gennie-memory-retrieval --lib`: 170 aprobadas, una ignorada, cero fallidas (snapshot compartido con fuentes/reconciliación). Incluye las cinco pruebas lifecycle finales. Log local temporal: `/tmp/gennie-write-retrieval.log`.
- `cargo test --offline -p gennie-memory --test source_verification_cli --test memory_lifecycle_cli --test write_reconciliation_cli`: cinco aprobadas, cero fallidas. Log local temporal: `/tmp/gennie-write-cli.log`.
- `cargo test --offline -p gennie-memory-retrieval --features local-embeddings --lib queue_tests`: cuatro aprobadas, cero fallidas en el enfoque previo a la revisión de control. La suite final debe incluir ese control con el estado compartido definitivo.

Regresión conjunta final del código compartido (incluidos controles de ciclo de vida, fuentes, reconciliación y cobertura):

| Comando | Aprobadas | Fallidas | Ignoradas |
| --- | ---: | ---: | ---: |
| `cargo test --offline --workspace` | 1.153 | 0 | 25 |
| `cargo test --offline --workspace --features gennie-memory/local-embeddings` | 1.169 | 0 | 25 |

Ambas suites finalizaron con código 0. `cargo fmt --all --check` y `git diff --check` finales también terminaron con código 0. Logs locales temporales: `/tmp/gennie-write-workspace.log` y `/tmp/gennie-write-feature.log`. La variante con feature incluye las cuatro pruebas de cola con el estado compartido definitivo y las cinco pruebas lifecycle tras las revisiones de control. Los enfoques anteriores documentan su snapshot; las cifras de workspace verifican el conjunto final. Las pruebas ignoradas no se cuentan como aprobadas. Los logs temporales no son artefactos publicados ni contienen datos personales de usuario.

## Límites

El codificador falso verifica publicación, transacciones y fencing, no calidad semántica, rendimiento o carga real de E5. No se ejecutaron llamadas de proveedor, descarga de modelos, servicios/puertos, benchmarks sostenidos, despliegue, commit ni push para este cambio. La cola funciona por demanda; no hay daemon. Recuperar una concesión después de un crash no acredita continuidad de contexto tras compactación en Codex. El borrado sólo alcanza el almacén activo definido en el contrato.
