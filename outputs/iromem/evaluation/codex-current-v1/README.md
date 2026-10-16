# Evidencia del piloto Codex con la ruta vigente

Este paquete conserva seis preguntas sintéticas del banco reservado 010–013. `selection.json` fijó los IDs antes de inspeccionar las respuestas. `codex-seed/`, `codex-engram/` y `codex-control/` contienen un `summary.json` y eventos JSONL por turno; `paired-flow-summary.json` empareja tiempos y tokens y distingue la suma calculada de una integración medida. El [informe](../../../../docs/evaluation/codex-current-v1.md) documenta resultados y límites.

La semilla procede de `../keyword-prefix-v1/holdout-010-013-decisions-network.jsonl`; las preguntas de `../keyword-prefix-v1/holdout-010-013-queries.jsonl`. Engram 3.4.0 utilizó el almacén sintético aislado ya creado para el benchmark de esa cohorte. Codex CLI fue `0.162.0-alpha.17.2` con `gpt-6.1-sol`. No se hicieron llamadas nuevas a OpenRouter; Codex informó tokens, pero no coste monetario.

La figura se regenera sin red desde los resúmenes guardados:

```sh
cargo run --offline -p iromem-eval -- render-codex-current \
  outputs/iromem/evaluation/codex-current-v1/codex-seed/summary.json \
  outputs/iromem/evaluation/codex-current-v1/codex-engram/summary.json \
  outputs/iromem/evaluation/codex-current-v1/paired-flow-summary.json \
  docs/evaluation/assets
```

Los eventos y las respuestas reflejan procesos reales de Codex con datos sintéticos. El suministro automático de semilla al producto continúa pendiente.
