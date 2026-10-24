# 5. Verificación y cierre acotado

Fecha: 2026-10-24. Estado: completado el subpaso de procedencia literal local. [Resultados, comandos y hashes](verification.json).

| Comprobación | Resultado |
| --- | --- |
| Controles propios de fuentes | 18 aprobados: 12 núcleo, 4 producto, 2 CLI/JSONL |
| Retrieval enfocado, tras integrar helper/uncertain | 170 aprobadas, 0 fallos, 1 ignorada |
| CLI conjunta de fuentes/ciclo de vida/escritura | 5 aprobadas, incluidas las 2 propias de fuentes |
| `cargo test --offline --workspace` final | 1153 aprobadas, 0 fallos, 25 ignoradas |
| `cargo test --offline --workspace --features gennie-memory/local-embeddings` final | 1169 aprobadas, 0 fallos, 25 ignoradas |
| `cargo fmt --all --check` y `git diff --check` | Correctos |

La verificación final fue consolidada por el chat de escritura autorizado por Fernando, en el mismo workspace, sin ejecutar Cargo simultáneo. Incluye los trece controles nuevos del chat de cobertura, los cambios de ciclo de vida y la reconciliación ya integrados. Se comprobaron los logs y presencia de los dieciocho tests propios en ambas suites; los hashes ligan las fuentes y sus dependencias compartidas al estado documentado. No atribuye estos totales únicamente a este cambio ni acredita modificaciones posteriores.

La primera suite workspace conjunta encontró once fallos heredados de service/embedding: sus bases de prueba solo aplicaban migraciones antiguas y las consultas nuevas de ciclo de vida necesitaban lifecycle_state. Se conservan [síntomas y causas](workspace-initial-failure.txt). El chat propietario corrigió los montajes para aplicar el módulo completo, sin reescribir migraciones históricas. Las pruebas propias de fuentes pasaron también en ese intento. La verificación conjunta añadió además controles de auditoría/borrado/reintentos de otros chats antes del pase final.

Este cierre comprueba registro, comparación literal, persistencia, políticas y recorridos locales sobre datos sintéticos. Mantiene las pruebas de proveedor ignoradas y no ejecuta inferencia pagada, base personal, commit, push o despliegue. Persisten warnings de imports/funciones existentes, registrados en JSON. No es auditoría integral de dependencias ni medición de latencia/recursos de fuentes.

Se preservaron rama y cambios previos. E5 sigue como investigación. Permanecen pendientes los conectores/autenticidad de fuentes externas, veracidad y fidelidad semántica, aplicabilidad independiente y relevancia/puntuaciones calibradas. El siguiente subpaso de recall es definir y comprobar aplicabilidad por candidato; este informe no cierra todo el hito.
