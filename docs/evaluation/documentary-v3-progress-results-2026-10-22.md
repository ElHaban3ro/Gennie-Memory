# Resultado consolidado del paso 3 — 2026-10-22

Se corrigieron dos defectos reales del mecanismo experimental: una negación con configuración pendiente podía permitir una respuesta completa incorrecta; además, una clase literal registrada en el vocabulario no satisfacía el filtro de contexto cuando `semantic_type` estaba ausente. Las correcciones están implementadas y verificadas, pero **la puerta general sigue fallando**. M5 y el objetivo de cinco pasos no están terminados.

| Medición | Antes | Después | Alcance |
| --- | --- | --- | --- |
| Selecciones útiles del banco completo | 23/39 (59,0 %) | 23/39 (59,0 %) | Mismas 100 entradas automáticas conocidas |
| Selecciones ajenas | 0/60 | 0/60 | Mismo banco; una ambigua separada |
| Selecciones útiles del piloto refinado | 3/8 | 4/8 | Mismas 12 parejas conocidas; diagnóstico |
| Ajenas del piloto refinado | 0/4 | 0/4 | Sin pérdidas; se recupera Olmo |
| Crítica independiente del nuevo mecanismo | — | 28/28 controles | 14 parejas válidas, dos adaptadores |
| Pruebas del workspace | 717 | 722 | 0 fallos; 24 ignoradas |
| Compatibilidad histórica | — | 226/226 idénticos | Informes de rutas anteriores |

La mejora histórica del banco completo fue 16/39→23/39. La corrección actual **no añade ganancias a ese banco**: siguen 16 omisiones, cero respuestas completas y una selección solo condicional. Los 28 controles son controles dirigidos, no un 100 % de calidad automática. El control manual 36/39 tampoco representa el producto actual.

## Qué se implementó y por qué

El getter `configuration_slot` conserva la configuración del mismo hecho y no toma valores de otros hechos. La revisión crítica encontró un contraejemplo: una negación pertinente cuyo contexto estaba pendiente no debía descartarse por no estar completamente resuelto. Ahora conserva esa deuda y bloquea publicar el valor. Una primera corrección demasiado amplia falló un control existente de contexto distinto; se acotó y quedó verificada con 717 pruebas, ocho controles críticos y 326 replays idénticos antes de añadir el siguiente cambio. Se conservan ambos fallos y sus logs.

El filtro contextual nuevo acepta una clase literal únicamente si coincide exactamente con una única fila del vocabulario y con el tipo solicitado. Solo actúa en contexto V2 para hechos positivos en estado, fundamento y scope compatibles, sin binders ni ramas, sin tipo explícito incompatible, anáfora o conflicto de identidad. No reescribe `semantic_type:null`, no prueba identidad y no infiere el origen o valor solicitado. La evidencia seleccionada es parcial y los términos de respuesta permanecen vacíos. `class_match` y las rutas anteriores conservan su contrato.

## Extracción y revisión ciega: resultados que se conservan

El primer piloto canónico tuvo 17/22 textos aceptados por ambos críticos y perdió una selección útil (5/5→4/5); se rechazó. El segundo congeló cuatro omisiones conocidas, cuatro positivos conservados y cuatro negativos: es desarrollo dirigido, no holdout. Dos autores separados y dos críticos nuevos revisaron sus 24 textos sin etiquetas ni resultados de parejas. Obtuvieron 20/24 textos conjuntamente fieles y 4/8 útiles, con una ganancia y una pérdida frente al control de ese subconjunto; también se rechazó.

Una única ronda de refinamiento con feedback semántico modificó tres textos y conservó 21 revisiones previas, incluida una fallida. Llegó a 22/24 aceptados, no a 24/24; no es una nueva medición ciega de todos los textos. La ejecución válida posterior dio 3/8 útiles, y la corrección del tipo contextual la devuelve ahora a 4/8 sin nuevas pérdidas. El caso Malva sigue sin una representación fiel del origen material/exclusividad; Lino conserva una incertidumbre temporal objetada. No se amplió el experimento a las 116 entradas.

La primera ejecución de ese refinamiento usó por error otro namespace en el harness y dio 0/8: **ese informe es inválido para medir calidad**. Se preservó, se corrigió exclusivamente el namespace para coincidir con los frames congelados y se repitió en otra carpeta con comprobaciones explícitas. Los resultados válidos son los del harness corregido y la comparación posterior. También se conserva la clarificación tardía de la guía: autores y críticos utilizaron la misma copia V5A congelada; la posterior V5B no se les atribuye retroactivamente.

## Latencia, coste y alcance

La comparación completa alternó el orden de ambos binarios: p50 CLI 30,924→30,018 ms; p95 61,870→63,370 ms. Excluye extracción, revisión, red y producto; se ejecutaron otros replays locales en paralelo, por lo que estas cifras son descriptivas y no demuestran mejora de latencia. No pasan la puerta de recursos del recorrido completo.

Cero llamadas externas y USD 0 de API nuevos en esta continuación. El coste de los subagentes Codex no está cuantificado. El acumulado externo previo permanece en 6.129 operaciones, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 operaciones con coste desconocido; no incluye Codex. No hay un total local acumulado reconciliado de todos los intentos.

La comprobación final pasó `cargo fmt --all --check`, compilación offline, 722 pruebas (24 ignoradas), 226 informes históricos idénticos y 28 controles críticos. Se preservan fuentes, entradas, binarios por hash, instrucciones, críticas y resultados de intentos fallidos. No se modificaron defaults, README, dashboard, base de datos o credenciales. V6 reservada permanece sin abrir.

## Estado de los cinco pasos

1. Contratos de identidad, premisa y valor definidos; la fidelidad automática todavía falla en casos específicos.
2. Banco conocido, negativos y revisiones independientes ejecutados; el último piloto no supera la puerta semántica.
3. Mecanismo implementado y corregido con evidencia; la calidad completa sigue en 23/39, por debajo de 39/39 requeridos.
4. No corresponde abrir V6 ni atribuir un pase contra producto antes de superar la puerta local.
5. Reporte y coste de esta tanda completos; adopción pendiente porque no existe mejora general suficiente.

La siguiente unidad de trabajo es resolver las dos pérdidas semánticas del contrato canónico y medir un piloto nuevo congelado, con autores y críticos separados. Repetir la misma tanda sin cambiar la causa no constituye progreso. No se rebaja la puerta ni se convierten controles dirigidos en una afirmación de calidad global.

## Evidencia

Artefactos: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`.

- `context-type-grounding-v1/known100-comparison/report.json`: 100 parejas, ambos binarios.
- `context-type-grounding-v1/refined12-comparison/report.json`: 3/8→4/8, mismas entradas.
- `context-type-grounding-v1/reviewOcritical-review-v1/critical-review-v1.json`: 28 controles independientes.
- `context-type-grounding-v1/root-verification/`: fuentes y logs de la verificación final.
- `configuration-slots-v1/negative-pending-critical-v1/followup-v2/`: corrección de la negación pendiente.
- `canonical-pilot-v2/report.json`: segundo piloto original.
- `canonical-pilot-v2-known-refinement-v1/report-corrected-harness-v2.json`: resultado válido previo a la corrección del filtro.
- `canonical-pilot-v2-known-refinement-v1/harness-namespace-failure.json`: fallo del harness preservado.

Binario final SHA-256: `7201939097504a377d3d3048a1da7702673867e01310f3609707b1187ab19dfb`.
