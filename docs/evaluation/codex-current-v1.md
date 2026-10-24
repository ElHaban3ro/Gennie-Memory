# Codex con la ruta vigente de Gennie Memory

Estado: piloto sintético ejecutado el 2026-10-16 en esta Mac. Se ejecutaron turnos reales del CLI de Codex con la semilla de la ruta actual **FTS5 por palabras clave con expansión de prefijos + Decisions**, tomada del [diario Rust congelado](../../outputs/iromem/evaluation/keyword-prefix-v1/holdout-010-013-decisions-network.jsonl). La entrega al prompt la hizo el runner de evaluación: Gennie Memory todavía no tiene una integración automática con Codex.

## Muestra y método

La [selección](../../outputs/iromem/evaluation/codex-current-v1/selection.json) se fijó antes de revisar resultados: primera pregunta positiva de cada ámbito `user`, `project`, `agent` y `conversation`, segunda positiva de `project` y primera negativa de `user`, en el orden del banco reservado 010–013. Son cinco positivas y una negativa de **2.800 recuerdos sintéticos**. El diario de Decisions ya contenía una semilla correcta para cada positiva y ninguna para la negativa; no se generaron nuevas llamadas a OpenRouter para este piloto.

Cada pregunta se ejecutó en un proceso efímero de Codex CLI `0.162.0-alpha.17.2`, modelo `gpt-6.1-sol`, sin configuración ni reglas personales y en sandbox de solo lectura. El runner Rust cronometró hasta el evento de respuesta final. Un camino recibió la semilla en el primer prompt y no usó herramientas; el otro recibió Engram 3.4.0 por MCP contra el almacén sintético aislado de la misma cohorte. Un tercer camino sin semilla permitió medir tokens añadidos por el prompt. Los prompts son distintos porque un camino recibe el contexto y el otro debe buscarlo.

## Resultado observado

| Camino de Codex | Aciertos | Tiempo a respuesta, mediana | Tokens de entrada informados, mediana | Llamadas MCP |
| --- | ---: | ---: | ---: | ---: |
| Gennie Memory: semilla en prompt | 5/5 positivas + 1/1 negativa | **4,38 s** | 16.989 | 0 |
| Engram: búsqueda MCP | 5/5 positivas + 1/1 negativa | 9,97 s | 52.792 | 7 en seis turnos |

Las cinco respuestas positivas se contrastaron con los hechos esperados del fixture y ambas respuestas a la negativa indicaron que no había dato pertinente. Los [resultados por turno](../../outputs/iromem/evaluation/codex-current-v1/) conservan texto, tiempo, uso informado y eventos.

| Caso | Gennie Memory + Codex | Engram + Codex |
| --- | ---: | ---: |
| Nombre de usuario | 5,49 s | 10,27 s |
| Responsable del proyecto | 5,09 s | 8,74 s |
| Idioma del agente | 4,48 s | 10,48 s |
| Tarea de conversación | 4,29 s | 9,66 s |
| Objetivo del proyecto | 4,27 s | 9,16 s |
| Negativa: dato inexistente | 4,12 s | 13,13 s |

La preparación de la semilla registrada en otra corrida Rust tuvo **0,39 s de mediana** en estos seis casos. La suma emparejada por caso de preparación más turno con semilla da **4,76 s de mediana**, calculada a partir de dos ejecuciones separadas; no es una medición de la integración instalada. El control sin semilla indica **48,5 tokens de entrada adicionales de mediana** para el prompt con semilla. Los totales de 16.989 y 52.792 incluyen instrucciones y caché, y **no** aíslan el coste marginal ni son directamente equivalentes a tokens facturados. Codex no informó un coste monetario para estos turnos; el piloto no hizo nuevas llamadas a OpenRouter.

La [figura de este piloto histórico](assets/current-codex-comparison.svg) se regenera desde los tres resúmenes Rust con `gennie-memory-eval render-codex-current`. Seis casos fáciles y sintéticos, una sola corrida por camino, orden fijo, distinta vía de entrega y posible variación de servicio no permiten afirmar una ventaja estadística general. La [comparación vigente](full-comparison-2026-10-16.md) repitió el piloto en otra cohorte; la [figura anterior](assets/speed-comparison-rust.svg) y su [informe](rust-migration-v1.md) se conservan como historia de la ruta previa.
