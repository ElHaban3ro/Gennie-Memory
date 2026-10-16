# Evidencia de la expansión léxica

Las consultas sintéticas de desarrollo y final están congeladas en el paquete [rust-migration-v1](../rust-migration-v1/). Las variantes de esta carpeta usaron la misma base SQLite de 2.800 recuerdos. Los `*.jsonl` contienen resultados por caso; los `*-summary.json` se calcularon con `iromem-eval retrieval` o `campaign-report`. El [informe](../../../../docs/evaluation/keyword-prefix-v1.md) explica la selección en desarrollo y el resultado final.

| Archivo | Contenido |
| --- | --- |
| `development-keywords-{baseline,prefix}.jsonl` y sus resúmenes | Contraste local de 135 búsquedas. |
| `development-prefix-{oldprompt,prompt}-decisions-network.jsonl` y sus resúmenes | 135 llamadas a Decisions por variante de instrucción, con eventos `started`/`finished`. |
| `final-prefix-keywords.jsonl`, manifiesto y resumen | 540 búsquedas con la variante congelada, hashes de binario, base y consultas. |
| `final-prefix-prompt-decisions-network.jsonl` y resumen | 540 llamadas de `recall` con la variante congelada; incluye respuestas, etiquetas, tiempos y uso informado. |
| `holdout-010-013-queries.jsonl` | 540 consultas de sujetos antes reservados, derivadas con `select-cohort` del fixture versionado. |
| `holdout-010-013-keywords.jsonl`, manifiesto y resumen | Búsqueda Rust sobre esos 540 casos. |
| `holdout-010-013-engram.jsonl`, manifiesto y resumen | Búsqueda Engram MCP 3.4.0 sobre las mismas 540 preguntas y el mismo contenido sintético importado a un almacén aislado. |
| `holdout-010-013-decisions-network.jsonl` y resumen | Campaña Rust de Decisions en la cohorte nueva, sin ajustar la variante después. |

Ejemplo de regeneración local, sin llamadas remotas:

```sh
cargo run --offline -p iromem-eval -- retrieval \
  outputs/iromem/evaluation/keyword-prefix-v1/final-prefix-keywords.jsonl \
  outputs/iromem/evaluation/rust-migration-v1/final-queries.jsonl
cargo run --offline -p iromem-eval -- campaign-report \
  outputs/iromem/evaluation/keyword-prefix-v1/final-prefix-prompt-decisions-network.jsonl
cargo run --offline -p iromem-eval -- retrieval \
  outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-keywords.jsonl \
  outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-queries.jsonl
cargo run --offline -p iromem-eval -- campaign-report \
  outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-decisions-network.jsonl
```

Una nueva campaña debe usar otro diario y registrar el binario, el banco y la base que realmente ejecute. Los diarios completos no se reetiquetan como mediciones de Codex.

Las dos figuras de la ruta vigente que muestra el README se regeneran desde los resúmenes guardados:

```sh
cargo run --offline -p iromem-eval -- render-current-charts \
  outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-keywords-summary.json \
  outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-decisions-network-summary.json \
  outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-engram-summary.json \
  docs/evaluation/assets
```

Las figuras comparativas históricas se regeneran por separado:

```sh
cargo run --offline -p iromem-eval -- render-keyword-charts \
  outputs/iromem/evaluation/rust-migration-v1/general-memory-final-keywords-summary.json \
  outputs/iromem/evaluation/rust-migration-v1/general-memory-final-decisions-summary.json \
  outputs/iromem/evaluation/keyword-prefix-v1/final-prefix-keywords-summary.json \
  outputs/iromem/evaluation/keyword-prefix-v1/final-prefix-prompt-decisions-network-summary.json \
  outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-keywords-summary.json \
  outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-decisions-network-summary.json \
  docs/evaluation/assets
```
