# Paquete de comparación completa, 2026-10-16

El [informe](../../../../docs/evaluation/full-comparison-2026-10-16.md) interpreta estos resultados. [suite-manifest.json](suite-manifest.json) fija versiones, hashes, cohorte y orden de ejecución. [aggregate.json](aggregate.json) reúne cifras calculadas de los archivos por caso; los JSONL y los resúmenes Rust son la evidencia primaria.

| Archivos | Contenido |
| --- | --- |
| `holdout-014-017-queries.jsonl`, `selection.json` | 540 preguntas nuevas y seis casos Codex fijados antes de medir. |
| `order-{1,2,3,4}.jsonl`, `run.log` | Permutaciones deterministas y orden real de las ocho corridas locales. |
| `search-{iromem,engram}-{1,2,3,4}.jsonl`, manifiestos y resúmenes | Resultados y tiempos por pregunta; cada par usó el mismo orden. |
| `recall-{decisions,jev}-network.jsonl`, resúmenes | Campañas de 540 llamadas con eventos sincronizados y coste informado. |
| `recall-decisions-restricted-network.jsonl` | Primer intento de una llamada, degradado por `transport_error`, excluido del análisis. |
| `codex-{seed,engram,control}-{1,2}/` | Respuestas, eventos y uso por turno, dos rondas de seis casos. |
| `paired-flow-{1,2}.json` | Preparación de semilla más turno, calculado por caso; no es integración instalada. |

La base iROMEM está en `.local/pilots/full-comparison-2026-10-16/iromem.db`, copia con el mismo SHA-256 que la base sintética validada. Los cuatro almacenes Engram nuevos están en `.local/pilots/full-comparison-2026-10-16/engram-{1,2,3,4}`. Esas rutas son estado local ignorado por Git; no contienen recuerdos personales. El binario Engram 3.4.0 usado estaba en `/private/tmp/iromem-engram-benchmark-20261015/engram` y su hash figura en el manifiesto. Codex se ejecutó con el binario incluido en la app ChatGPT instalada, no con el ejecutable antiguo de `~/.local/bin`.

## Repetir los cálculos sin red

Desde la raíz del repositorio, estos comandos recalculan los resúmenes de una pareja y los informes de clasificación en **rutas nuevas**. Cambia las rutas de salida antes de ejecutarlos: el evaluador protege los resultados existentes.

```sh
target/debug/iromem-eval retrieval \
  outputs/iromem/evaluation/full-comparison-2026-10-16/search-iromem-1.jsonl \
  outputs/iromem/evaluation/full-comparison-2026-10-16/holdout-014-017-queries.jsonl \
  /tmp/iromem-search-summary-check.json
target/debug/iromem-eval retrieval \
  outputs/iromem/evaluation/full-comparison-2026-10-16/search-engram-1.jsonl \
  outputs/iromem/evaluation/full-comparison-2026-10-16/holdout-014-017-queries.jsonl \
  /tmp/engram-search-summary-check.json
target/debug/iromem-eval campaign-report \
  outputs/iromem/evaluation/full-comparison-2026-10-16/recall-decisions-network.jsonl \
  /tmp/iromem-decisions-summary-check.json
target/debug/iromem-eval campaign-report \
  outputs/iromem/evaluation/full-comparison-2026-10-16/recall-jev-network.jsonl \
  /tmp/iromem-jev-summary-check.json
```

La nueva medición usa `search-benchmark`, `engram-benchmark`, `campaign`, `codex-prompt`, `codex-engram` y `paired-flow-report` con la sintaxis de [contract-v1.md](../../../../docs/evaluation/contract-v1.md). No repitas una llamada remota incierta cambiando solo el archivo de salida; los diarios guardan `started`/`finished` para detectar ese estado. Las campañas de OpenRouter y los turnos de Codex consumen servicios remotos.

Las cinco figuras vigentes del README se regeneran **sin red** desde el agregado congelado con:

```sh
cargo run --offline -p iromem-eval -- render-full-comparison \
  outputs/iromem/evaluation/full-comparison-2026-10-16/aggregate.json \
  docs/evaluation/assets
```

La figura adicional de p95 por corrida permanece en `docs/evaluation/assets/full-comparison-p95-2026-10-16.svg`. La figura del recorrido calculado suma `recall` y Codex por caso a partir de ejecuciones separadas; no mide el tiempo de un hook instalado.
