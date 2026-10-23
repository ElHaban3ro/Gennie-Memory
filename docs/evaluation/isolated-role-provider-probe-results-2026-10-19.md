# Verificación aislada: comparación de proveedores

**Ningún proveedor supera las puertas del diagnóstico.** Jev reduce el coste de verificación, pero acepta dos IDs no permitidos y pierde evidencia útil. Decisions conserva los cinco casos V4, pero pierde dos refutaciones V5. No se incorpora una variante ni se cambia `baseline`; V6 sigue sin llamadas.

## Qué se midió

Los 43 positivos de las propuestas históricas V2 —25 V5 y 18 V4— recibieron dos comprobaciones nuevas, una por proveedor. Se preservaron consulta, recuerdo, instrucciones completas de datos V1 o correcciones V4, criterios y composición. **Solo cambia `model`** dentro de cada par. Se alternó el orden de proveedores; se conservaron las 86 respuestas, sin reintentos ni selección de mejores repeticiones.

La comparación es sobre propuestas congeladas conocidas. No mide una propuesta nueva, búsqueda nueva ni el flujo completo contemporáneo. El informe Rust independiente vincula fuentes, pares, estados y recibos; además revalida las trazas históricas V2 con su auditor ordinario de verificación. El primer informe y la versión que incorpora esta última comprobación coinciden en todas las métricas.

## Calidad y coste

| Verificador y grupo | Pérdidas de evidencia útil de base/scopedV1 | IDs no permitidos | Verificación nueva USD | Flujo estimado/base histórica | Puerta de calidad |
| --- | ---: | ---: | ---: | ---: | --- |
| Decisions, 13 V5 | 2 | 0 | 0,000819000 | 1,981× | Falla |
| Decisions, 5 V4 | 0 | 0 | 0,000588900 | 2,402× | Pasa |
| Jev 1.13, 13 V5 | 2 | 2 | 0,000542724 | 1,603× | Falla |
| Jev 1.13, 5 V4 | 2 | 0 | 0,000393960 | 1,845× | Falla |

El flujo estimado suma al coste nuevo de verificación los recibos históricos de propuesta: USD 0,000628320 en V5 y USD 0,000251244 en V4. **No son nuevos cargos de propuesta ni coste medido de un flujo completo actual.** Ambos grupos deben cumplir calidad y coste ≤1,5× antes de implementar y evaluar una nueva ruta; ningún proveedor cumple.

Las 43 verificaciones Decisions cuestan USD 0,001407900 y las 43 Jev USD 0,000936684: reducción observada del **33,5 % en esta etapa**. No demuestra un ahorro equivalente del sistema completo ni compensa las pérdidas y selecciones incorrectas.

| Verificador y grupo | p50/p95 de una comprobación aislada |
| --- | --- |
| Decisions V5 | 399 / 687 ms |
| Decisions V4 | 377 / 551 ms |
| Jev V5 | 348 / 544 ms |
| Jev V4 | 335 / 361 ms |

Son tiempos de etapa, no latencias end-to-end de `recall` o Codex. Las respuestas de una corrección incluyen sus dos elecciones en un solo HTTP y recibo.

## Qué explican las pérdidas

Decisions pierde `cev5_073_06` y `cev5_066_06`. El primer recuerdo niega que Laurel use agua de la red y especifica un depósito de lluvia; el segundo niega que se capturen aves en Ribera y describe lectura con telescopio. Ambas son refutaciones explícitas de la premisa preguntada. La propuesta histórica marcó ambas como `requested_fact_or_partial`, por lo que **solo recibieron la comprobación de datos**, sin pregunta de corrección.

Esto evidencia una dependencia del mecanismo: un tipo propuesto fallable decide de forma exclusiva qué tarea verifica un recuerdo. No basta con aislar el contexto ni con mejorar el coste del proveedor. Todavía no se ha medido si añadir la otra vía recupera esos recuerdos sin aceptar nuevos IDs ajenos; no se da por resuelto.

Jev acepta `cev5_071_03` y `cev5_068_07`, pierde la condición `cev5_068_12` y el contexto parcial `cev5_069_12`, y pierde los parciales `cev4_062_27` —músicos de una sección— y `cev4_063_09` —ingredientes con cantidades ilegibles—. El ID 06912 mantiene su discrepancia semántica retrospectiva documentada, sin retirarlo del gold para pasar. No se convierten las pérdidas parciales ni las refutaciones en una única exactitud optimista.

## Verificación, registro y próximo experimento

137 pruebas del workspace pasan; dos tests remotos manuales quedan ignorados en la ejecución offline normal. Se ejecutó una sola vez el nuevo test manual de 86 peticiones. Formato y compilación del evaluador pasan; una comprobación adicional de las 55 pruebas del evaluador pasa tras reforzar el vínculo con las trazas históricas. No hay política de producto nueva, cambios SQL, secretos, servicios o bases temporales nuevas.

Artefactos: `outputs/iromem/evaluation/role-isolated-provider-probe-2026-10-19/`, con solicitudes/preflight anteriores a red, diario y `report-audited.json`. La huella del ejecutable de prueba se conserva; las fuentes archivadas son parciales y no se afirma una reconstrucción autónoma del workspace. [Protocolo](isolated-role-provider-probe-protocol-2026-10-19.md).

Gasto remoto nuevo: **86 operaciones/HTTP, USD 0,002344584**, todos los recibos completos. Acumulado del objetivo: **3427 operaciones, 21 campañas emparejadas completas, USD 0,241054998 conocidos y tres operaciones históricas de importe desconocido**. Este diagnóstico de etapas no incrementa las campañas emparejadas completas. No incluye programación/revisión con Codex.

El siguiente diagnóstico debe probar explícitamente la vía alternativa que falta cuando un rol propuesto rechaza evidencia útil, con textos aislados completos y composición OR visible. Primero se medirán precisión, preservación y coste sobre propuestas congeladas; no se implementará un bypass sin comprobación ni se seleccionarán retrospectivamente las respuestas más convenientes. Una segunda vía no tiene ahorro demostrado y puede añadir falsos positivos. Las puertas completas de desarrollo, consistencia y revisión nueva V6 permanecen pendientes.
