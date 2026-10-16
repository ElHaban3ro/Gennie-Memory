# Evidencia de la evaluación Rust

Este paquete contiene únicamente datos sintéticos. El [informe](../../../../docs/evaluation/rust-migration-v1.md) explica método, resultados, límites y corridas descartadas.

| Archivo | Contenido |
| --- | --- |
| `general-memory-final-keywords.jsonl` y `.manifest.json` | 540 búsquedas de iROMEM en la ruta rápida, con candidatos y tiempos por caso. |
| `final-queries.jsonl` y `development-queries.jsonl` | Entradas exactas congeladas de ambas cohortes. Incluyen vectores históricos que la ruta rápida Rust ignora; se conservan para que coincidan las huellas de los diarios. |
| `general-memory-final-engram.jsonl` y `.manifest.json` | Las mismas 540 búsquedas mediante Engram MCP aislado. |
| `general-memory-final-{keywords,engram}-summary.json` | Cobertura y percentiles calculados por `iromem-eval retrieval`. |
| `general-memory-final-{decisions,jev}-network.jsonl` | Diarios completos de 540 llamadas a `recall` por proveedor, con eventos `started`/`finished`, etiquetas, semilla y uso informado. |
| `general-memory-final-{decisions,jev}-summary.json` | Aciertos y percentiles calculados por `iromem-eval campaign-report`. |
| `general-memory-development-decisions-network.jsonl` y su resumen | Semillas Rust de la cohorte de desarrollo utilizadas para los seis turnos Codex. |
| `codex-seed-v2/`, `codex-engram-v3/`, `codex-control-v2/` | Seis turnos por camino: respuesta, uso y eventos JSONL por caso. |
| `paired-flow-summary.json` | Sumas por caso de preparación Rust y turno Codex con semilla, calculadas desde registros separados. |

Las figuras del README se regeneran **sin llamadas de red** desde estos informes:

```sh
cargo run --offline -p iromem-eval -- render-charts \
  outputs/iromem/evaluation/rust-migration-v1/general-memory-final-keywords-summary.json \
  outputs/iromem/evaluation/rust-migration-v1/general-memory-final-engram-summary.json \
  outputs/iromem/evaluation/rust-migration-v1/general-memory-final-decisions-summary.json \
  outputs/iromem/evaluation/rust-migration-v1/general-memory-final-jev-summary.json \
  outputs/iromem/evaluation/rust-migration-v1/codex-seed-v2/summary.json \
  outputs/iromem/evaluation/rust-migration-v1/codex-engram-v3/summary.json \
  docs/evaluation/assets
```

Para repetir las campañas, usa salidas nuevas y las entradas/bases declaradas en los manifiestos. No reanudes estos diarios terminados para crear una medición nueva. El almacenamiento Engram aislado y la base SQLite iROMEM de la prueba viven en `.local/pilots/` y no se publican; los fixtures, las consultas exactas y los resultados por caso sí están versionados. La base se puede reconstruir desde `tests/fixtures/general-memory-v1/memories.json`, pero una base nueva tendrá otra huella y su latencia debe medirse como una corrida nueva.
