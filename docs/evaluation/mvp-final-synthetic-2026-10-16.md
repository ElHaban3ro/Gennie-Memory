# Cohorte sintética final 018–020: búsqueda y semilla

Estado: campaña ejecutada el 2026-10-16 en esta Mac. La cohorte y los criterios se fijaron [antes de abrir resultados](mvp-final-protocol-2026-10-16.md). Los [casos, diarios, informes y análisis](../../outputs/iromem/evaluation/mvp-final-2026-10-16/) conservan resultados por pregunta. Las 48 etiquetas de la [muestra de auditoría](mvp-final-audit-sample-2026-10-16.md) fueron cotejadas por el asistente con el fixture; la revisión humana independiente sigue pendiente. No hubo ajuste de búsqueda, prompt o umbral después de observar esta cohorte.

## Alcance y resultado

Se cargaron **2.800 recuerdos ficticios** en una SQLite nueva y se ejecutaron **405 preguntas** de doce ámbitos nuevos: 345 positivas y 60 negativas. La ruta fue FTS5 por palabras clave, límite veinte, Decisions por defecto; Jev recibió los mismos casos por una campaña independiente. La búsqueda local encontró el recuerdo esperado en **343/345** positivos (99,42 %), con **314/345** en primera posición, p50 **7,03 ms**, p95 **8,07 ms** y p99 **8,35 ms**. No devolvió IDs prohibidos. Ninguno de los 60 negativos tuvo lista de candidatos vacía; eso no equivale a producir una semilla falsa.

| Métrica de 405 consultas | Decisions | Jev |
| --- | ---: | ---: |
| Positivos con semilla de ID correcto | **331/345 (95,94 %)** | 316/345 (91,59 %) |
| Negativos con semilla vacía | **60/60** | **60/60** |
| Acierto si el esperado estaba entre candidatos | 331/343 (96,50 %) | 316/343 (92,13 %) |
| Consulta → semilla p50 / p95 / p99 | 391 / 1.090 / 1.945 ms | **371 / 478 / 998 ms** |
| Tokens de entrada informados, p50 | **561** | 962 |
| Coste informado de 405 casos | US$0,02231670 | **US$0,01617143** |
| Degradadas / IDs prohibidos | 0 / 0 | 0 / 0 |

El coste informado conjunto fue **US$0,03848813**; es el valor de OpenRouter en los diarios, no una factura auditada. Decisions sirvió `openai/gpt-6-luna-decisions-20261006` y Jev `typesafe/jev-1.13-20260917` en los 405 casos de cada campaña. OpenRouter informó 0 tokens de salida de mediana para Decisions y 166 para Jev; no se reconstruye consumo interno a partir de esos campos. La semilla ocupó 175 y 174 bytes UTF-8 de mediana respectivamente; eso **no es un recuento de tokens de semilla**. Las tarifas publicadas al preparar la campaña constan en el protocolo.

| Tipo de ámbito | Positivos | Decisions correctos | Jev correctos | Negativos vacíos, ambos |
| --- | ---: | ---: | ---: | ---: |
| `user` | 75 | 75 | 75 | 15/15 |
| `project` | 90 | 90 | 89 | 15/15 |
| `conversation` | 90 | **77 (85,56 %)** | 72 (80,00 %) | 15/15 |
| `agent` | 90 | 89 | 80 | 15/15 |

En positivos, ambos acertaron 313 casos; Decisions acertó 18 que Jev falló, Jev acertó tres que Decisions falló y ambos fallaron once. Los dos fallos de cobertura de búsqueda fueron de `conversation`. De los catorce fallos de Decisions, trece son de `conversation`: doce abstenciones y una elección incorrecta; el restante es una abstención de `agent`. En Jev hubo diecisiete abstenciones y una elección incorrecta en `conversation`, diez abstenciones en `agent` y una en `project`. El agregado superior a 95 % **no se sostiene en `conversation`**.

El objetivo previo exigía al menos 328/345 positivos y 60/60 negativos vacíos. Decisions lo cumple **como tasa observada en este banco**; Jev no alcanza la meta positiva. El intervalo Wilson de 95 % para positivos con Decisions es **93,30–97,57 %** y con Jev **88,19–94,08 %**; para 60/60 negativos vacíos es **93,98–100 %** en ambos. Un remuestreo exploratorio de los doce ámbitos completos (10.000 muestras, semilla `20261016`) dio **91,43–99,41 %** y **86,29–96,47 %** para positivos, respectivamente. El límite inferior de Decisions queda bajo 95 % y solo hay tres sujetos por tipo: no se ha demostrado una tasa poblacional de al menos 95 %. Las 405 preguntas comparten plantillas y no son 405 observaciones humanas independientes.

## Fidelidad y límites de medición

En cada memoria incluida en la semilla (332 con Decisions, 317 con Jev), resumen, condición, fuente declarada, ámbito y revisión coincidieron con el fixture, normalizando `locator` omitido a `null`; hubo **cero discrepancias estructurales**. El hecho de que una semilla copie bien `synthetic://` no verifica una fuente externa. Se revisaron todas las discrepancias de ID y los negativos; las etiquetas de la cohorte siguen siendo deterministas. No se midieron contradicciones reales, permisos de un host, resolución de ámbito, tokens exactos de semilla ni calidad de la respuesta del agente.

Los tiempos de `timings.total_ms` incluyen búsqueda, proveedor y composición dentro del servicio, pero excluyen arranque de proceso, apertura de SQLite y preparación del cliente HTTP. El `search-benchmark` sí incluye ida y vuelta por una sesión persistente. Una repetición local de 405 búsquedas consumió 3,09 s reales, 2,85 s de CPU de usuario y 0,14 s de sistema para **evaluador y proceso hijo juntos**; la base SQLite cargada ocupó **2.584.576 bytes**. No se obtuvo pico de RAM ni una medición comparable en frío: `/usr/bin/time -l` no pudo consultar `kern.clockrate` bajo el sandbox. La latencia remota depende de la red y el servicio; en esta corrida Decisions tuvo p95 de 1.090 ms, distinto de los 567 ms de la cohorte 014–017.

Un primer intento en la red restringida devolvió `transport_error` para ambas campañas y está [separado como diagnóstico](../../outputs/iromem/evaluation/mvp-final-2026-10-16/diagnostic-restricted-network/README.md). La red no resolvía OpenRouter; las corridas válidas usaron diarios nuevos fuera de esa restricción. Los diagnósticos no entran en ninguna cifra de acierto o coste.

## Decisión que permite esta evidencia

Decisions sigue siendo el candidato experimental para la ruta rápida: supera el objetivo observado agregado y conserva los 60 negativos vacíos. **No cierro la viabilidad general del MVP**: `conversation` queda en 77/90, el intervalo de incertidumbre no acredita 95 % y faltan preguntas naturales independientes revisadas. El siguiente trabajo de calidad debe estudiar los fallos de selección y cobertura de `conversation` usando desarrollo nuevo, manteniendo esta cohorte congelada. Una integración Codex posterior tendrá que medir desde el mensaje del usuario hasta la respuesta final; ninguna cifra aquí mide ese recorrido.

Las figuras actuales del [README](../../README.md) siguen representando la comparación 014–017 entre motores y el piloto Codex de seis turnos; sus cohortes están etiquetadas allí. Este informe no renombra esas figuras como resultados 018–020 ni proyecta la mejora de Codex a esta cohorte.
