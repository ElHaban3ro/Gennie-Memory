# Regresión baseline con evidencia explícita: 30 casos

Banco sintético de desarrollo, preparado antes de inferencia: 20 positivos y 10 negativos, con dos búsquedas vacías. Aliso conserva consulta, búsqueda y contexto del primer caso de desarrollo de `product-contract-v1`; los otros 29 casos son nuevos. `corpus.json` contiene 61 revisiones de 60 recuerdos. Los grupos, IDs y etiquetas son solo evaluación; `request` es la única entrada enviada al producto.

`manifest.json` congela los hashes de corpus, política y casos. Etiquetas del mismo agente, sin revisión humana o independiente (`agent_synthetic_unreviewed`). No es reserva final ni reemplaza el banco prospectivo anterior. No ajustar etiquetas después de observar respuestas.

Ejecución pagada explícita, desde la raíz y con directorio de salida nuevo:

```sh
cargo build --offline -p gennie-memory -p gennie-memory-eval
target/debug/gennie-memory-eval run-baseline-regression tests/fixtures/baseline-evidence-30-v1/manifest.json target/debug/gennie-memory outputs/gennie-memory/evaluation/baseline-evidence-30-NUEVO
```

Máximo 30 llamadas / 180 segundos / USD 0,02; reserva contable por operación USD 0,002. Sin reintentos; recibo desconocido detiene el recorrido. La prueba sin credencial usa `GENNIE_MEMORY_OPENROUTER_API_KEY=''` y otro directorio nuevo. No incorporarlo a CI offline. El runner usa su propia SQLite, sesión y capturas y verifica ID, revisión, texto, metadatos, estado y resultado de semilla. Exige 100 % de evidencia útil, cero selecciones ajenas/degradaciones y p95 ≤ 2.000 ms. `unassessed` conserva la ausencia de roles en baseline.

[Resultados y límites](../../../docs/evaluation/baseline-evidence-30-results-2026-10-23.md).
