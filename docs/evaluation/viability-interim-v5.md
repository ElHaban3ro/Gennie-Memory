# Viabilidad provisional de recuerdos semilla — v5

Estado: evidencia local y remota del 2026-10-15; **no es la decisión final del MVP**. Los conjuntos de desarrollo y calibración se evaluaron en esta Mac con Laya local y datos sintéticos. Jev y OpenAI Decisions se midieron por OpenRouter en 1.600 casos principales de desarrollo y 400 peticiones adversariales repartidas entre desarrollo y calibración.

## Pregunta y criterio de lectura

El experimento comprueba dos decisiones distintas: si el recuerdo relevante llega a los candidatos y si la política compone la semilla correcta o se abstiene. Una semilla exacta requiere elegir el recuerdo etiquetado en un positivo, o entregar semilla vacía en un negativo. La fidelidad de hecho, condición y fuente se verificó en los positivos acertados. La exactitud agregada se presenta junto a positivos y negativos porque una política que siempre responde o siempre se abstiene puede parecer competitiva según la mezcla del corpus.

## Corpus principal v5

El corpus contiene 42.400 cadenas de consulta únicas organizadas en 1.000 formas normalizadas. Las campañas usaron 1.600 casos de desarrollo (1.000 positivos, 600 negativos) y 1.480 de calibración (1.000 positivos, 480 negativos), sin entidades ni IDs compartidos. El recuerdo esperado apareció entre los cuatro candidatos híbridos en los 2.000 positivos. Los métodos vieron los mismos candidatos y vectores por caso. [Desarrollo](../../outputs/iromem/evaluation/v5-campaign-development/README.md) y [calibración](../../outputs/iromem/evaluation/v5-campaign-calibration/README.md) contienen manifiestos, informes por caso y límites de medición.

| Política híbrida | Desarrollo: positivo / negativo | Calibración: positivo / negativo | p50 desarrollo / calibración |
| --- | ---: | ---: | ---: |
| Top‑1, sin decisión Laya | 719/1.000 · 200/600 | 701/1.000 · 160/480 | 20,04 / 20,12 ms |
| Laya `choice` | 136/1.000 · 568/600 | 149/1.000 · 453/480 | 52,72 / 52,92 ms |
| Laya binario condicionado, umbral 0,5 | 494/1.000 · 200/600 | 456/1.000 · 160/480 | 86,61 / 87,11 ms |

En este corpus, la [ablación FTS5 frente a híbrida](retrieval-ablation-v5.md) produjo exactamente los mismos candidatos y elecciones top‑1 en 3.080 casos; FTS5 fue más rápido. El corpus principal no ejercita una brecha léxica, por lo que ese empate no demuestra que el índice semántico sea prescindible.

La [campaña remota principal](../../outputs/iromem/evaluation/v5-campaign-remote/README.md) reutilizó los candidatos híbridos congelados en los 1.600 casos de desarrollo. Jev obtuvo **1.000/1.000 positivos y 570/600 negativos** (1.570/1.600 semillas exactas), sin fallos del proveedor. OpenAI obtuvo **992/1.000 y 526/600** (1.518/1.600), incluidos ocho HTTP 504 como errores. Jev costó US$0,03015 informado; OpenAI US$0,03943 informado o reservado. La p50 de decisión remota, sin búsqueda ni composición, fue 475,91 y 482,07 ms. Jev supera a las variantes locales en exactitud en esta muestra, con mayor latencia y coste externo por consulta. La comparación no mide un tiempo integrado remoto ni ahorro neto del agente.

## Peticiones sin coincidencia textual

Se probaron aparte [400 peticiones sintéticas completas](../../outputs/iromem/evaluation/v5-natural-gap/README.md), 200 positivas y 200 negativas, repartidas por igual entre desarrollo y calibración. Son 40 frases en inglés reutilizadas con entidades diferentes y redactadas para eliminar coincidencias FTS5. FTS5 devolvió cero candidatos; la búsqueda híbrida incluyó el recuerdo correcto entre dos candidatos en los 200 positivos.

Una comprobación cruzada de las 400 etiquetas contra `corpus.json` confirmó que cada positivo apunta a una revisión activa del mismo ámbito y entidad, con hecho, condición y fuente coincidentes; los 200 negativos carecen de memoria esperada y todas las listas de IDs esperados y prohibidos son disjuntas. También verificó 50 casos de cada segmento por partición. Esto valida la coherencia de los fixtures, pero **no** sustituye una revisión humana de si las 40 frases representan preguntas reales ni prueba que la interpretación de cada frase sea inequívoca.

| Política | Positivos exactos / 200 | Negativos vacíos / 200 | Lectura |
| --- | ---: | ---: | --- |
| FTS5 top‑1 | 0 | 200 | Omite todos los recuerdos necesarios. |
| Híbrida top‑1 | 104 | 0 | Recupera algunos positivos, añade memoria en todos los negativos. |
| Híbrida Laya `choice` | 2 | 188 | Se abstiene a menudo, omite casi todos los positivos. |
| Híbrida Laya binario condicionado (0,5) | 99 | 0 | Selecciona algunos positivos, añade memoria en todos los negativos. |

El barrido exploratorio de umbrales sobre las puntuaciones binarias guardadas alcanzó 100/200 exactos por partición únicamente absteniéndose en todos los casos; no se fijó un umbral. La asimetría por modalidad también importa: en cada partición, el binario condicionado acertó 47/50 y 45/50 positivos de videollamada, pero solo 3/50 y 4/50 presenciales. Hay cobertura de candidatos y un problema posterior de aplicabilidad y orden.

Las puntuaciones por segmento refuerzan ese diagnóstico: la mediana del **máximo** binario fue 0,9309 en los 50 negativos `explicit_no_recall` de desarrollo y 0,9306 en los 50 de calibración. Para los positivos remotos fue 0,8131 y 0,8080; para los presenciales, 0,8328 y 0,8320. Por tanto, el clasificador dio a menudo más confianza a recuerdos que el usuario prohibió consultar que a recuerdos aplicables. Estas cifras son descriptivas de las plantillas del sondeo; no calibran probabilidades.

Como sondeo de diagnóstico adicional, se preguntó al mismo snapshot Laya por un veto **antes** de presentarle candidatos en ocho peticiones de desarrollo (dos por segmento), esta vez en CPU. Una pregunta binaria dio puntuaciones de uso mayores que 0,5 en los ocho casos, incluso en los cuatro negativos. Una elección explícita `use`/`skip` acertó los cuatro positivos y solo uno de los cuatro negativos: 5/8. Los resultados locales están en `.local/pilots/laya-memory-gate-8.json` y `.local/pilots/laya-memory-gate-choice-8.json` (ignorados por Git). La muestra es mínima, comparte frases con el sondeo principal y no establece una política nueva; no se ejecutó una campaña adicional.

La [comparación remota en las 400 peticiones sin coincidencia FTS5](../../outputs/iromem/evaluation/v5-natural-gap-remote/README.md) redujo el optimismo del corpus principal. Jev conservó **147/200 positivos** y dejó vacíos **200/200 negativos**; OpenAI, **69/200 y 199/200**, con un HTTP 504 negativo. Las particiones de entidades disjuntas dieron a Jev 73/100 y 74/100 positivos, y 100/100 negativos cada una. La política de Jev parece más equilibrada en estas plantillas, pero deja 53 positivos sin el recuerdo correcto. No se fijó una política final.

## Conclusión operativa provisional

El núcleo local **sí ejecuta** carga sintética, filtros implementados, FTS5, índice semántico, composición y validación de semillas. La evidencia local **no justifica activar automáticamente** top‑1, Laya `choice` o Laya binario condicionado: top‑1 y el binario condicionado añaden recuerdos en demasiados negativos, mientras `choice` omite demasiados positivos. Un nuevo umbral binario no resolvió ese intercambio. Jev es el proveedor más prometedor de los medidos en estas muestras sintéticas, con más latencia y dependencia externa; aún falla en tarea ajena del corpus principal y omite positivos en el sondeo adversarial.

**Decisión provisional:** vale la pena continuar el experimento con Jev como candidato externo y conservar FTS5 + semántica para cobertura. No hay evidencia suficiente para activar recuperación automática en trabajo real: el banco adversarial todavía deja 53/200 positivos sin semilla correcta, la clasificación remota añade alrededor de medio segundo por llamada completa y no se ha medido ningún ahorro neto del agente. Las dos campañas remotas costaron en conjunto US$0,08782 informados o reservados; esa cifra de prueba no incluye mantenimiento del índice, recursos locales ni facturación final de la cuenta.

Antes de una decisión final hacen falta una política de abstención/aplicabilidad fijada sin reajustarla en `final`, revisión humana de etiquetas y casos menos ligados a las plantillas del generador. También faltan mediciones del coste total de operación, CPU/RAM y una evaluación del conjunto `final` una vez congeladas las decisiones. No existe hook inicial; las latencias medidas no prueban ahorro de tiempo ni tokens de Codex.

La [campaña remota principal](remote-v5-runbook.md) ejecutó 2.800 llamadas reales tras autorización del usuario, con tope US$1 y hasta 60 minutos activos. Informó o reservó US$0,06959; el [sondeo remoto adversarial](../../outputs/iromem/evaluation/v5-natural-gap-remote/README.md) añadió 800 llamadas por US$0,01824. Los diarios y los informes separan respuestas reales de la anterior simulación sin red. La facturación efectiva debe contrastarse con la cuenta del proveedor.
