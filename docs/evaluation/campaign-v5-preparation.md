# Campaña v5 de desarrollo: preparación

Estado: **muestra congelada y campaña local completada**; [resultados por caso](../../outputs/iromem/evaluation/v5-campaign-development/README.md). La comparación remota posterior ya se ejecutó y se documenta en outputs/iromem/evaluation/v5-campaign-remote/README.md. La regla de [pruebas](../rules/testing.md) exige definir corpus, presupuesto y duración antes de correr un benchmark grande.

## Corpus congelado

- Archivo: [`development-campaign-1600.jsonl`](../../tests/benchmark/v5/development-campaign-1600.jsonl), derivado de [`queries.jsonl`](../../tests/benchmark/v5/queries.jsonl).
- SHA-256: `cfcae2f21348a1b640e4b4ff00c7a278a8caf012585eed55132dd63352b35137`.
- 1.600 consultas sintéticas de `development`: una positiva por cada una de las **1.000 formas** del banco y 100 negativas por cada uno de seis segmentos. Los 1.600 textos son distintos. No hay IDs de consulta ni entidades compartidas con las tres muestras v5 publicadas; contiene 203 entidades distintas.
- Selección: orden estable por SHA-256 de cada ID dentro de cada forma positiva y segmento negativo, excluyendo IDs y entidades de `sample-a`, `sample-b` y `sample-c`. Después se preparó y ejecutó una [muestra de calibración separada](calibration-v5-preparation.md). La partición final permanece cerrada.
- Limitación: las 1.000 formas se componen a partir de marcos, prefacios y cierres; no equivalen a 1.000 intenciones humanas independientes. Falta revisión humana de una muestra de etiquetas.

Auditoría estática del corpus completo: las 1.000 plantillas son distintas incluso tras normalización NFKC, cambio a minúsculas y recorte de espacios; las 42.400 preguntas renderizadas tienen texto distinto y ninguna petición completa duplicada. Esta comprobación mide diversidad superficial y unicidad, no independencia semántica ni corrección humana de las etiquetas.

## Ejecución local realizada

Se compararon, sobre los **mismos 1.600 casos**, `top1`, Laya `choice` y Laya `noul` por candidato con la condición explícita, usando búsqueda híbrida y la misma base con los 1.200 vectores de memorias v5. El umbral de `noul` permaneció en 0,5. Cada selector dejó un diario y un informe completo; se usaron `--journal`, `--limit 1600`, `--require-mps` y `--max-seconds-per-run 300`. Los tres procesos terminaron antes del corte y la revisión posterior confirmó candidatos idénticos, cobertura semántica completa y cero fallos de decisión.

El presupuesto local fijado fue **tres recorridos de 1.600 casos, hasta 15 minutos activos en total** y cero llamadas pagadas. Ninguna invocación llegó a su corte individual de cinco minutos; la suma de tiempos por caso en los informes publicados fue 33,54 s, 81,21 s y 129,96 s, sin incluir arranque, carga ni escritura. Los tres recorridos se repitieron con la misma versión del runner para conservar los vectores híbridos exactos necesarios al componer semillas remotas: coincidieron los 1.600 candidatos, elecciones y aciertos con la primera corrida de cada método. Los informes publicados deduplican los vectores hacia una sola instantánea validada. El runner midió latencias por etapa, cobertura de candidatos, exactitud, fidelidad de hecho/condición/fuente, abstenciones y fallos. MPS estuvo disponible; no se midieron CPU/RAM globales ni ahorro del agente principal.

Antes de la campaña se probó el diario en dos casos top‑1 y dos con Laya en CPU; la corrida completa posterior se hizo en MPS y dejó 1.601 eventos por selector (manifiesto más 1.600 casos).

## Comparación remota posterior

Jev 1.13 y OpenAI Decisions por OpenRouter usaron una instantánea de búsqueda idéntica, con hasta cuatro candidatos por pregunta. El informe completo del recorrido top‑1 preservó `retrieval_snapshot`; `scripts/export_campaign_retrieval.py` validó y exportó esos candidatos a un JSONL aceptado por `run_openrouter_campaign.py prepare`. La exportación y la preparación no llamaron a proveedores. El [plan remoto v5](../../outputs/iromem/evaluation/v5-campaign-development/v5-campaign-openrouter-plan.json) calcula **2.800 llamadas** para dos modelos (1.400 cada uno), reserva US$0,56, fija máximo US$1 y 3.600 segundos activos. El usuario autorizó la ejecución de esa campaña pagada. El plan remoto v4 previo de 1.600 llamadas no corresponde a este corpus v5. La campaña v5 completó 2.800 llamadas intentadas y recompuso 1.600 semillas por proveedor; resultados y límites en outputs/iromem/evaluation/v5-campaign-remote/README.md.

El [procedimiento remoto](remote-v5-runbook.md) conserva el vector de cada consulta hasta `compose-seed-batch` y `evaluate_seeds`; un caso positivo y uno negativo pasaron una prueba de integración con decisiones remotas simuladas, sin llamada a OpenRouter.

Las métricas locales miden resolución de semillas. El ahorro de tiempo o tokens del agente principal requiere una evaluación posterior con un host integrado; no se infiere de estas latencias.
