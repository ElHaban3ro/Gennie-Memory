# Campaña local v5 de memorias semilla — desarrollo

Estado: **tres recorridos locales completos** el 2026-10-15 en esta Mac. La [comparación remota posterior](../v5-campaign-remote/README.md) ya se ejecutó sobre esta misma muestra. Corpus: [1.600 consultas sintéticas congeladas](../../../../tests/benchmark/v5/development-campaign-1600.jsonl), SHA-256 `cfcae2f21348a1b640e4b4ff00c7a278a8caf012585eed55132dd63352b35137`: las 1.000 formas positivas aparecen una vez y hay 600 negativos (100 por segmento). Cero consultas o entidades compartidas con los tres pilotos previos. Esta campaña local no usó calibración ni final; posteriormente se [preparó calibración](../../../../docs/evaluation/calibration-v5-preparation.md) y se ejecutaron los tres clasificadores locales sobre ella.

## Resultado local

Los tres procesos usaron MPS, el mismo índice de 1.200 recuerdos activos, el encoder experimental Laya 0.3.5 de 768 dimensiones y los **mismos candidatos en el mismo orden** para cada consulta. La cobertura semántica fue completa en 1.600/1.600 y el recuerdo etiquetado apareció entre los cuatro candidatos en 1.000/1.000 positivos. No hubo fallos de decisión, semillas degradadas ni filtraciones de IDs prohibidos.

| Selector | Positivos exactos / 1.000 | Negativos vacíos / 600 | Semillas exactas / 1.600 | p50 total | p95 total | p99 total | Tokens de clasificación |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Top‑1 sin clasificador | 719 | 200 | 919 | 20,04 ms | 37,90 ms | 40,50 ms | 0 |
| Laya `choice` | 136 | 568 | 704 | 52,72 ms | 86,41 ms | 89,67 ms | 248.901 |
| Laya `noul` con condiciones, umbral 0,5 | 494 | 200 | 694 | 86,61 ms | 135,77 ms | 140,08 ms | 518.588 |

En los positivos correctos, hecho, condición y referencia de fuente quedaron fieles en la semilla: 719, 136 y 494 casos respectivamente. El resto de positivos se perdió por elección «ninguno» o memoria equivocada, no porque faltara el recuerdo entre los candidatos. Top‑1 eligió una memoria equivocada en 281 positivos y añadió memoria a 400 negativos; `choice` omitió 685 positivos, eligió otra memoria en 179 y añadió memoria a 32 negativos; `noul` condicionado omitió 206 positivos, eligió otra memoria en 300 y añadió memoria a 400 negativos.

| Segmento | Casos | Top‑1 | Laya `choice` | Laya `noul` condicionado |
| --- | ---: | ---: | ---: | ---: |
| `absent_entity` | 100 | 100 | 100 | 100 |
| `alias` | 200 | 160 | 29 | 91 |
| `code_switch` | 100 | 65 | 14 | 35 |
| `direct` | 250 | 203 | 46 | 105 |
| `english` | 200 | 102 | 39 | 91 |
| `explicit_no_recall` | 100 | 0 | 99 | 0 |
| `indirect` | 150 | 130 | 7 | 74 |
| `long_context` | 100 | 59 | 1 | 98 |
| `mode_unknown` | 100 | 0 | 100 | 0 |
| `mode_unspecified` | 100 | 0 | 100 | 0 |
| `wrong_scope` | 100 | 100 | 100 | 100 |
| `wrong_task` | 100 | 0 | 69 | 0 |

`choice` evita muchos falsos recuerdos negativos, pero pierde gran parte de los positivos. La variante binaria condicionada recupera más positivos que `choice` y llega a 98/100 en `long_context`, pero conserva los 400 falsos recuerdos negativos de top‑1. Ninguna variante constituye una política lista para producción. Top‑1 también usa el encoder Laya para **búsqueda híbrida**; «sin clasificador» solo significa que no ejecuta la decisión Laya.

## Evidencia y límites

Una [ablación posterior](../../../../docs/evaluation/retrieval-ablation-v5.md) encontró que filtros estructurados + FTS5 devolvieron exactamente los mismos candidatos y 919/1.600 semillas exactas que esta ruta híbrida, con p50 de 5,50 ms frente a 20,04 ms. Este banco no contiene positivos donde FTS5 pierda el recuerdo esperado.

- [Resumen computado](v5-campaign-summary.json): hashes de los tres informes, métricas por segmento y tablas de concordancia por caso.
- [Análisis de errores y umbrales](../../../../docs/evaluation/error-analysis-v5.md): omisiones, memorias equivocadas, falsos recuerdos y límite de las puntuaciones binarias. [Datos del barrido](threshold-analysis.json).
- [Manifiestos de ejecución](execution-manifests.json): huellas de consultas, base, binario, modelo, lote semántico y código del runner; cada diario local contiene un manifiesto y 1.600 casos sincronizados.
- Informes completos por caso: [top‑1](v5-campaign-top1-1600-report.json), [Laya `choice`](v5-campaign-laya-choice-1600-report.json), [Laya condicionado](v5-campaign-laya-conditions-1600-report.json). Cada fila conserva candidatos, estado, fidelidad y latencias. Los vectores idénticos de las tres corridas se almacenan una sola vez en la instantánea híbrida; cada informe registra la huella del informe original completo antes de quitar esa duplicación.
- [Instantánea de búsqueda](v5-campaign-hybrid-results.jsonl): exportación validada del recorrido top‑1, con el vector exacto de cada consulta, para que la comparación remota vea los mismos candidatos y `compose-seed` pueda revalidar la semilla con la misma búsqueda híbrida.
- [Plan OpenRouter v5](v5-campaign-openrouter-plan.json): 1.600 casos, 2.800 llamadas previstas para Jev 1.13 y OpenAI Decisions (1.400 por modelo), reserva de US$0,0002 por llamada, máximo de US$1 y 3.600 segundos activos. El [diario y los resultados reales](../v5-campaign-remote/README.md) ya cerraron las 3.200 tareas. La reserva inicial sumaba US$0,56; el gasto informado o reservado al finalizar fue US$0,06959.
- [Prueba remota simulada](remote-dry-run-summary.json): recorrió las 3.200 tareas del plan, simuló 2.800 respuestas sin red y compuso 1.600 semillas por modelo con cero IDs prohibidos. Verifica integración y formatos; no mide calidad, latencia ni coste de los proveedores.

El 2026-10-15, el catálogo de OpenRouter indicaba [US$0,042/M tokens de entrada para Jev 1.13](https://openrouter.ai/typesafe/jev-1.13/) y [US$0,10/M para GPT-6 Luna Decisions](https://openrouter.ai/openai/gpt-6-luna-decisions), con salida gratuita. Esos precios se comprobaron antes de ejecutar; la facturación efectiva debe contrastarse con la cuenta del proveedor.

Los tres recorridos se repitieron con la misma versión del runner histórico para registrar los vectores: los 1.600 IDs, candidatos, elecciones y aciertos coincidieron exactamente con las primeras corridas; las puntuaciones binarias también coincidieron. Sus latencias publicadas corresponden a las repeticiones. La compactación verificó que cada vector coincidía con la instantánea antes de quitar las copias de los informes públicos. Una prueba de integración con un positivo y un negativo sintéticos convirtió decisiones remotas simuladas en `compose-seed-batch` y evaluación de semillas: 2/2 semillas completas y exactas, fidelidad de hecho/condición/fuente en el positivo, negativo vacío y cero IDs prohibidos. Los scripts de esa campaña fueron retirados; esto no sustituye una prueba real de proveedor.

Las etiquetas proceden de reglas deterministas del generador y faltan revisión humana y escenarios adversariales adicionales. La entidad ya viene resuelta por un host sintético; las referencias de fuente son declaradas, no verificadas. El perfil de embeddings comparte pesos con el clasificador y no es el perfil independiente previsto. Se hizo un recorrido por selector, en orden fijo; la caché del sistema y el estado térmico pueden influir en latencias, y no hay estimación de variación entre repeticiones. No se midieron CPU/RAM del proceso completo ni ahorro de tiempo o tokens del agente principal, ya que no existe hook en este MVP. Estos resultados son de **desarrollo**, con plantillas relacionadas; no son una evaluación final ni prueban generalización a preguntas humanas.
