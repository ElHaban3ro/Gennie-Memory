# 5. Regresiones y cierre acotado

Estado: completado el subpaso de presupuestos de tokens en recall-v1, 2026-10-24. [Resultados y hashes de fuentes](verification.json).

| Comprobación | Resultado |
| --- | --- |
| `cargo fmt --all --check` | Correcto |
| `git diff --check` | Correcto |
| `cargo test --offline --workspace` | 1100 aprobadas, 0 fallos, 25 ignoradas |
| `cargo test --offline --workspace --features local-embeddings` | 1112 aprobadas, 0 fallos, 25 ignoradas |

Nueve pruebas nuevas, además del mock HTTP existente ampliado con contabilidad del cuerpo exacto. Las suites incluyen validación de producto, preservación literal, transporte/deadlines, CLI/JSONL, evaluadores y runtime de embeddings optativo. Las ignoradas mantienen su condición; no se habilitó inferencia de proveedor. Persisten avisos de compilación sobre imports/funciones existentes no usados, registrados en el JSON; no se ejecutó una auditoría integral de dependencias, clippy ni despliegue.

Iteraciones: la primera prueba de semilla omitida descubrió una marca de inclusión incorrecta para equivalentes. Se separó selección para composición de inclusión publicada y se corrigieron ambas ramas. Las pruebas paralelas descubrieron una colisión del sandbox basado únicamente en reloj; se añadió una secuencia atómica. También se corrigieron errores de compilación de los nuevos mocks antes de ejecutar las suites finales. Los resultados finales corresponden a las fuentes enlazadas y sus hashes.

Se preservaron cambios anteriores, rama y worktree; no hubo commit, push, modificación de base personal o nuevo servicio persistente. Los mocks propios y sandboxes se cierran/retiran al terminar. E5 queda como investigación, sin integración automática.

Este paso limita contexto según una convención local; no demuestra el tokenizer nativo, facturación, calidad semántica, veracidad de la fuente ni relevancia calibrada. La primera carga del contador se midió aparte y está documentada en el paso 4. El hito completo de recall permanece abierto: sigue verificación de fuentes, después aplicabilidad/relevancia y validación calibrada; el híbrido solo requiere integración si vuelve a activarse para producto.
