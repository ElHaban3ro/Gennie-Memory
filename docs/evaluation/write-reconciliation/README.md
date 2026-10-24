# Escritura y reconciliación local

Fecha: 2026-10-24. Paso del roadmap implementado para operador personal `local`. [Contrato de comandos](../../architecture/write-reconciliation.md) · [Decisión 0054](../../decisions/0054-local-write-reconciliation.md).

## Resultado

- `mem-write`/JSONL conserva creación/revisión e incorpora confirmación, fuente estricta optativa, tema único y propuestas pendientes fuera de FTS. Las entidades/alias conservan su resolución explícita sin fusión por nombre y comparten las restricciones de UUID entre superficies.
- Huellas exactas preservan actor/entidad/ámbito, original/síntesis, negación, condiciones y fechas. Fuentes distintas se conservan como evidencia. Un recuerdo retirado exige restauración explícita antes de reutilizarlo por duplicación.
- `relation_provider` habilita una sola petición tipada optativa sobre hasta 20 candidatos y una cota de 128 KiB. Expone las seis relaciones y la incertidumbre, con recibo/diagnóstico; el juicio nunca sobrescribe. La llamada ocurre sin transacción y las revisiones se revalidan al volver.
- `mem-proposal-get`, `mem-resolve`, `mem-audit` y `mem-operation-get` tienen equivalentes JSONL. Resolver exige el conjunto exacto de destinos vigentes, evidencia literal de decisión/contenido y commit atómico; purga el texto pendiente. Conflictos y revisión vencida se muestran como incertidumbre derivada en recuperación/semilla, sin modificar revisiones históricas.

## Pruebas significativas

Trece casos de `reconciliation::tests` comprueban recibos/idempotencia y separación de actores/ámbitos, fuentes adicionales/negación, propuestas estrictas fuera de FTS, aceptación tras registrar snapshot, tema único y todos los destinos, revisión obsoleta, confirmación que conserva contenido/fuente, agenda omitida frente a `null`, citas inventadas, rollback completo e incertidumbre hasta descarte. La prueba de carrera simula un proceso externo con una segunda conexión exclusivamente en el test: escribe durante la etapa del clasificador, demuestra que no hay una transacción retenida y exige rechazo de la decisión obsoleta. El producto conserva una sola conexión.

Una prueba del adaptador construye el payload real y comprueba clasificación tipada/roster completo y conservación del recibo en salida inválida, sin red. Dos pruebas CLI ejecutan propuestas/resolución/confirmación/auditoría/consulta de recibos, reapertura y recuperación JSONL tras una línea inválida. Usan SQLite temporal y credencial vacía. Las pruebas compartidas de fuentes, cobertura y ciclo de vida también entran en las suites finales.

La regresión adicional `audit_accepts_exact_evidence_bytes_written_by_lifecycle` reprodujo un fallo real de integración: el lector volvía a serializar JSON y podía confundir orden de campos con corrupción. Ahora compara SHA-256 de los bytes almacenados y conserva el hash observado; sigue rechazando evidencia alterada. También verifica el rechazo de duplicación sobre recuerdo retirado. Otra iteración corrigió una colisión de directorios de pruebas paralelas con una secuencia atómica; no cambió producción por ese fallo.

## Verificación final

| Verificación | Resultado |
| --- | --- |
| `cargo test --offline --workspace` | 1153 aprobadas, 0 fallos, 25 ignoradas |
| `cargo test --offline --workspace --features gennie-memory/local-embeddings` | 1169 aprobadas, 0 fallos, 25 ignoradas |
| `cargo fmt --all --check` | Correcto |
| `git diff --check` | Correcto |

Los conteos, comandos, avisos y hashes del código probado se registran en [verification.json](verification.json). Los resultados corresponden a workspace compartido con los otros chats, no exclusivamente a esta entrega. Las pruebas ignoradas siguen ignoradas; no se ejecutaron campañas pagadas ni se modificó la base personal.

## Límites

`report` mantiene fuentes declaradas por compatibilidad; el modo estricto es explícito. Un snapshot del operador acredita presencia literal, no autenticidad externa ni verdad de interpretación. La clasificación remota está implementada pero su calidad/coste/latencia no se evaluaron con un proveedor real en esta entrega; los recibos de mocks no son gasto real. La selección de candidatos FTS5 no es exhaustiva para paráfrasis. No hay ACL multiusuario ni autenticación de host; los metadatos opcionales generales del diseño v0.1 permanecen fuera del contrato local. El ciclo de vida usa su adaptador y alcance propios. No hubo commit, push, despliegue, hook o servidor MCP.
