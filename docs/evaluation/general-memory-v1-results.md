# Recuerdos generales y personales v1: nueva evaluación sintética

Estado: campaña local y remota completada el 2026-10-15 en esta Mac. No se activó un hook, servicio ni memoria real. [Datos, resultados por caso y manifiestos](../../outputs/iromem/evaluation/general-memory-v1/).

## Conclusión

En el nuevo banco de preguntas sobre personas, proyectos, conversaciones y agentes, la configuración provisional de la campaña anterior **no alcanzó el objetivo observado del 95 % de positivos con semilla correcta**. Con búsqueda híbrida y máximo veinte candidatos, OpenAI GPT-6 Luna Decisions acertó **433/460 (94,13 %)** y dejó vacíos **80/80** negativos; Jev 1.13 acertó **414/460 (90,0 %)** y dejó vacíos **79/80**. Son las mismas preguntas y las mismas listas de candidatos. Las 460 positivas están agrupadas por 16 sujetos sintéticos y comparten plantillas; estas proporciones no estiman por sí solas el rendimiento en conversaciones humanas.

La búsqueda incluyó el recuerdo correcto en **459/460** positivos finales dentro de los primeros veinte. OpenAI omitió 21 recuerdos necesarios y eligió otro en 6; Jev omitió 41 y eligió otro en 5. El único fallo de cobertura del buscador está incluido en esos totales. La mayor pérdida ocurre **después** de generar candidatos, sobre todo en recuerdos del ámbito `conversation`.

Laya local 0.3.5 sí cargó en **MPS** con el mismo hash de pesos y tokenizer de la campaña anterior. Con una formulación directa eligió correctamente **96/460 (20,87 %)** positivos y dejó vacíos **20/80** negativos. Una instrucción más explícita subió a **125/460 (27,17 %)** positivos, pero bajó los negativos vacíos a **7/80**. El problema observado es de selección y aplicabilidad con distractores, no una ausencia de aceleración ni un error de formato de la respuesta. Este snapshot de Laya no es un clasificador adecuado para este recorrido tal como se ha probado.

## Diseño y separación de datos

- El [fixture sintético](../../tests/fixtures/general-memory-v1/README.md) contiene 2.800 recuerdos activos y 3.300 preguntas con ámbito dado: 2.800 positivas y 500 negativas. Cubre `user`, `project`, `conversation` y `agent`, doce tipos de recuerdo y múltiples temas. Las 140 preguntas adicionales sin ámbito explícito se excluyeron: iROMEM todavía no resuelve contexto ni permisos de un host.
- Se asignó **un sujeto por ámbito a desarrollo** (115 positivos, 20 negativos), **cuatro sujetos distintos por ámbito a final** (460 positivos, 80 negativos) y se mantuvieron los demás como reserva. No se reutilizó un sujeto entre desarrollo y final. Los datos y etiquetas proceden de reglas deterministas; no hubo revisión humana.
- Todas las preguntas usaron filtros de ámbito explícitos y la misma recuperación híbrida: FTS5, vector del encoder multilingüe de Laya, coseno exacto y RRF. El índice contiene los 2.800 recuerdos; el perfil de embedding sigue siendo experimental. El lote local generó 2.800 vectores de recuerdos y 3.300 de consultas en MPS en **33,28 s**, aparte de cargar el modelo. La búsqueda se ejecutó una vez y sus candidatos se congelaron antes de clasificar.
- En desarrollo se compararon instrucciones `direct` y `grounded` para los dos proveedores remotos. La instrucción `direct` dio mejores positivos en ambos y quedó fijada antes de leer las decisiones finales. Laya probó además una instrucción en español y topes 5, 10 y 20 en desarrollo; en final se midieron `direct` y `grounded` con tope veinte. En ningún caso se modificó la lista de candidatos según el modelo.

## Cobertura y tiempo de búsqueda

| Ámbito final | Positivos | Correcto en top 1 | Top 5 | Top 10 | Top 20 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Usuario | 100 | 69 | 87 | 92 | 100 |
| Proyecto | 120 | 62 | 94 | 104 | 119 |
| Conversación | 120 | 51 | 97 | 103 | 120 |
| Agente | 120 | 59 | 97 | 103 | 120 |
| **Total** | **460** | **241** | **375** | **402** | **459** |

Las 80 negativas finales también recibieron candidatos; abstenerse no fue una consecuencia automática de una lista vacía. En los 540 casos finales, búsqueda p50 **18,61 ms**, p95 **21,51 ms**. Las cifras son de búsqueda sobre vectores ya calculados; no incluyen generar el vector de una pregunta en vivo. [Resumen de las 3.300 búsquedas](../../outputs/iromem/evaluation/general-memory-v1/retrieval-summary.json).

## Clasificación y semillas finales

| Selector, tope 20 | Positivos correctos | Negativos vacíos | Decisión p50 / p95 | Coste remoto informado |
| --- | ---: | ---: | ---: | ---: |
| Jev 1.13, `direct` | 414/460 · 90,0 % | 79/80 | 517 / 737 ms | US$0,02117 |
| OpenAI GPT-6 Luna Decisions, `direct` | 433/460 · 94,13 % | 80/80 | 492 / 705 ms | US$0,02885 |
| Laya local, `direct` | 96/460 · 20,87 % | 20/80 | 95 / 174 ms | Sin llamada remota |
| Laya local, `grounded` | 125/460 · 27,17 % | 7/80 | 95 / 165 ms | Sin llamada remota |

| Ámbito | Jev positivos | OpenAI positivos |
| --- | ---: | ---: |
| Usuario | 100/100 | 99/100 |
| Proyecto | 114/120 | 119/120 |
| Conversación | 88/120 | 97/120 |
| Agente | 112/120 | 118/120 |

En desarrollo, `direct` dio **105/115** positivos y **20/20** negativos a Jev, y **109/115** y **20/20** a OpenAI. `grounded` bajó los positivos a **47/115** y **85/115** respectivamente, sin cambiar los negativos. La instrucción más larga no ayudó a los modelos remotos en este banco. [Informes de desarrollo](../../outputs/iromem/evaluation/general-memory-v1/remote-development-direct-report.json) y [final](../../outputs/iromem/evaluation/general-memory-v1/remote-final-direct-report.json).

`compose-seed-batch` reejecutó la búsqueda y comprobó los IDs y revisiones presentados. Conservó **414/460** y **433/460** positivos correctos, y **79/80** y **80/80** semillas vacías en negativos para Jev y OpenAI. En **todos los aciertos positivos**, la semilla contenía el resumen esperado y la fuente sintética declarada; las condiciones declaradas se conservaron en **34/34** aciertos etiquetados de Jev y **36/36** de OpenAI. La fuente `synthetic://` no fue verificada externamente. La composición midió p50 **12,32 ms** y **12,37 ms** en corridas separadas. [Resumen Jev](../../outputs/iromem/evaluation/general-memory-v1/final-jev-seed-summary.json) · [resumen OpenAI](../../outputs/iromem/evaluation/general-memory-v1/final-openai-seed-summary.json).

## Diagnóstico de Laya

El mismo checkpoint y tokenizer usados antes pasaron la verificación de hash; Laya respondió en MPS. En una prueba pequeña y controlada de **8 positivos y 4 negativos de desarrollo**, el recuerdo correcto se insertó deliberadamente entre candidatos del mismo ámbito. Con la instrucción `direct`, Laya acertó **6/8** positivos con máximo cinco opciones y **2/8** con veinte. En veinte opciones, entre los seis errores positivos eligió otro recuerdo cuatro veces y `none` dos veces. Este sondeo aísla el clasificador del orden de la búsqueda, pero es demasiado pequeño para estimar una tasa general. [Registro del diagnóstico](../../outputs/iromem/evaluation/general-memory-v1/laya-oracle-diagnostic.json).

En el desarrollo híbrido completo, las mejores combinaciones locales fueron `grounded` con cinco candidatos (**43/115** positivos, **14/20** negativos vacíos; techo de cobertura 94/115) y `grounded` con veinte (**35/115**, **5/20**; cobertura 115/115). La variante en español tampoco resolvió el problema. Dar más opciones recuperó el recuerdo correcto, pero empeoró la selección. En final, los resultados `direct` y `grounded` de la tabla anterior confirman el intercambio entre acierto positivo y falsos recuerdos. [Resultados locales por caso](../../outputs/iromem/evaluation/general-memory-v1/laya-final-journal.jsonl).

## Coste, fallos y alcance de la conclusión

Las dos pantallas remotas de desarrollo y la fase final sumaron **1.626 intentos** para **1.620 decisiones** planificadas, con **US$0,08143415** informados o reservados. La fase final registró 1.084 intentos para 1.080 decisiones: un HTTP 429 de Jev y varios HTTP 502 de OpenAI se conservaron como intentos separados y se resolvieron con reintentos dirigidos; no quedaron tareas fallidas ni inciertas. El primer intento restringido de desarrollo falló por DNS antes de llegar a OpenRouter. El importe reservado para intentos sin respuesta no equivale necesariamente a un cargo facturado. No hubo tope monetario para la investigación.

Los percentiles de búsqueda, decisión y composición son **mediciones separadas**; no constituyen latencia punta a punta de Codex. No hay hook ni evidencia de ahorro neto de tiempo para el agente. Las plantillas sintéticas, el ámbito ya resuelto, la ausencia de identidades ambiguas o permisos reales y las fuentes solo declaradas impiden trasladar la tasa de acierto a uso humano. El siguiente experimento debe concentrarse en los errores de `conversation` y en mejorar la decisión tras top veinte, manteniendo un nuevo grupo de sujetos intacto para validación. [Resumen estructurado](../../outputs/iromem/evaluation/general-memory-v1/final-summary.json).
