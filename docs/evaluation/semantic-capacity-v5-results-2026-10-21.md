# Perfil de capacidad V5: mejora conocida, puerta de calidad pendiente

Paso 3 del objetivo activo. **Se completaron las 46 respuestas válidas: conserva 35/35 controles fieles y bloquea 7/11 construcciones no aceptables, pero acepta indebidamente cuatro. Persisten 16 discrepancias de 184 dimensiones. No se promueve ni se abre V6.** [Protocolo](semantic-capacity-v5-protocol-2026-10-21.md), [reporte Rust](../../outputs/iromem/evaluation/semantic-capacity-v5-2026-10-21/report.json) y [diagnóstico](../../outputs/iromem/evaluation/semantic-capacity-v5-2026-10-21/failure-diagnosis.json).

## Qué cambió y qué se probó

El perfil usa Claude Opus 5.5 por la ruta exclusiva Anthropic de OpenRouter. Conserva los 46 inputs/IDs, etiquetas V1, mensajes y esquema estricto V4, traducción exacta y validador V1. Solo cambian modelo, proveedor, omisión de temperature —no anunciado por la ruta— y límite de salida 2048→8192 para dejar margen al razonamiento y contenido. No se fija una política de razonamiento no verificada. Es una intervención conjunta de perfil, no una comparación causal exclusivamente de modelos.

La crítica estática no encontró bloqueos. Pasaron 508 pruebas con y sin HTTP2. Se compararon todos los payloads contra V4 congelado; antes de HTTP se reprodujeron preparación, inputs, anotaciones, paquete/mapa, fuentes y binario. Los cuatro bancos manuales, diez informes de extracción y seis auditores/reportes locales anteriores se reproducen idénticos. Ambos reportes nuevos también se reproducen. El catálogo confirmó la ruta disponible y sus parámetros. La revisión interna del alias no está comprobada independientemente; no se afirma negociación HTTP2 observada.

## Resultado completo, sin sustituir denominadores

| Medida | V5 |
| --- | --- |
| Previstas / HTTP200 / válidas / recibos conocidos | 46 / 46 / 46 / 46 |
| No enviadas / reintentos / fallos técnicos | 0 / 0 / 0 |
| Controles fieles conservados | 35/35 |
| Construcciones no aceptables bloqueadas | 7/11 |
| Aceptaciones indebidas / rechazos de fieles | 4 / 0 |
| Dimensiones concordantes / previstas | 168/184 |
| Discrepancias | 16 |
| Entrada / salida informadas | 416.764 / 32.904 tokens |
| Razonamiento informado, incluido en salida | 14.561 tokens; disponible en 46/46 recibos |
| Coste nuevo / operaciones sin coste conocido | USD 2,325136 / 0 |
| Intento terminal p50 / p95 | 10.694 / 15.896 ms |
| Duración real del runner | 164,181 s |

Las once construcciones no aceptables incluyen nueve defectuosas y dos inciertas. Aceptar las inciertas como fieles también falla el criterio predeclarado; no se las elimina del denominador. Los 35 controles son representaciones manuales conocidas, no recuerdos recuperados automáticamente. Pasar esquema, citar literalmente y completar HTTP no demuestra verdad semántica.

## Comparación histórica y coste de la ganancia

| Mismo banco conocido y etiquetas | V1 histórico, GPT-4.1 | V5, perfil Opus |
| --- | --- | --- |
| Respuestas válidas | 46 | 46 |
| Controles fieles conservados | 35/35 | 35/35 |
| Construcciones no aceptables bloqueadas | 1/11 | 7/11 |
| Aceptaciones indebidas | 10 | 4 |
| Discrepancias de dimensión | 31/184 | 16/184 |
| Coste informado | USD 0,210604 | USD 2,325136 |

Hay seis bloqueos adicionales y ninguna regresión de aceptación respecto a V1; por dimensión se corrigen 15 y ninguna antes correcta empeora. Es una mejora diagnóstica observada en este banco conocido, con coste 11,04×. Además del modelo/ruta cambian presentación, esquema y capacidades respecto a V1; no hay control contemporáneo que aísle deriva o un único factor. No es una comparación de recuperación con baseline ni una puerta de recursos aprobada. [Comparación íntegra por ID y dimensión](../../outputs/iromem/evaluation/semantic-capacity-v5-2026-10-21/historical-comparison.json).

V4, medido antes, se detuvo con tres citas no literales entre cuatro respuestas. Sus cuatro dimensiones válidas no se comparan como porcentaje completo con184. V5 resolvió la disponibilidad del contrato en la ronda completa, pero eso no subsana las respuestas V4 ni prueba fiabilidad poblacional. Ambos diarios permanecen intactos.

## Revisión de razones y causas pendientes

Un crítico recibió los 46 inputs canónicos y los184 juicios/razones, sin etiquetas de referencia, puntuaciones agregadas, origen o identidad del modelo. Veía las predicciones: es revisión crítica con referencia oculta, no predicción ciega. Pertenece a la misma familia de agentes que elaboró las anotaciones; no es revisión humana ni prueba de verdad externa. Revisó todas las entradas y señaló ocho hallazgos con citas literales y rutas existentes. [Dictamen](../../outputs/iromem/evaluation/semantic-capacity-v5-2026-10-21/rationale-critical-review.json) y [conciliación separada](../../outputs/iromem/evaluation/semantic-capacity-v5-2026-10-21/rationale-review-reconciliation.json).

Dos aceptaciones indebidas tienen problemas concretos de fundamento: s029 conserva la ocurrencia temporal de «se observó», pero permite basis=any y pierde el requisito observacional; s040 atribuye observed_fact a una simple declaración de peso que no afirma medición. El crítico detectó ambos sin etiquetas. Las otras dos, s021 y s039, aprueban un alcance transitivo o inmediato de suministro que el texto no resuelve expresamente; el crítico localiza incertidumbre, sin declarar una falsedad externa comprobada.

También señaló completitud relacional en s038, que el modelo ya bloqueaba por otras dimensiones, y una duda de completitud temporal en s001, igualmente bloqueado. Su revisión no resuelve todos los desacuerdos de dimensiones, incluido s019. El reporte previo conserva su indicador histórico de revisión pendiente; el dictamen posterior se adjunta por separado, sin modificar predicciones, gold, métricas o el resultado fallido de capacidad.

El siguiente mecanismo debe comprobar por separado fundamento epistémico, cobertura de relaciones y alcance de flujo, con controles fieles y toda la cohorte conservados. Las fronteras contractuales genuinamente inciertas deben aclararse de forma prospectiva, sin reescribir etiquetas antiguas para hacer pasar V5. Ese siguiente mecanismo aún no está implementado. Siguen pendientes capacidad completa, extracción/tubería automática, conocidos100/126, baseline, recursos/estabilidad y casos nuevos con revisión ciega.

## Cuenta y límites de uso

La estimación anunciada fue USD 2,39 y 3–8 minutos para hasta 46 peticiones; la ilustración conservadora de bytes como tokens y salida máxima fue USD 11,95, sin garantía de factura. Se cobraron USD 2,325136 con recibos completos, sin reintentos/fallback ni peticiones inciertas. Los 14.561 tokens de razonamiento están incluidos en el consumo de salida informado; no se suman por segunda vez. Estos tokens y tiempos pertenecen al verificador, no a entrada de Codex, recall o recorrido completo.

La tanda V4+V5 suma **50 inferencias y USD 2,391848**. Hubo tres GET públicos de metadatos separados —catálogo V4, descubrimiento de modelos y catálogo V5—. Acumulado: **6.105 operaciones lógicas, 23 campañas emparejadas completas, USD 3,519868508 conocidos más 16 operaciones históricas de coste desconocido**; excluye programación/revisión de Codex. Completar 46 operaciones diagnósticas no añade una campaña emparejada contra baseline.

Snapshots 001–063, configuración vigente, producto, SQL, dashboard, servicios, README/gráficas y reserva V6 se conservan. No hay promoción. El goal íntegro sigue activo en paso 3.
