# Verificación selectiva: campañas completas y revisiones

Estado2026-10-19: **ninguna variante supera todas las puertas**. No se promueve ni abreV6. La autorización general del usuario cubre las pruebas sintéticas; el antiguo bloqueo de permisos está resuelto.

| Tanda | Calidad / preservación | Coste informado | Recursos |
| --- | --- | --- | --- |
| V1,V5:64pares/128operaciones | Contra base:0IDsajenos,0pérdidas,2preguntas recuperadas y9casos con evidencia adicional. Frente a scopedV1: pierde1IDútil | USD0,008573416ambas rutas; candidataUSD0,005093216 | Coste1,463×; p500,780×; p950,881×. Pasa recursos, falla preservación |
| V1,V4:64pares/128operaciones | 2casos contaminados con IDsajenos,3pérdidas frente a base | USD0,009730608ambas rutas | Coste1,162×; p500,972×; p951,410×. Pasa recursos, falla calidad |
| V2,piloto13V5 | Repara contraste06607pero pierde corrección06806 | USD0,001039996/14HTTP | Coste1,423×histórico; no amplía |
| V3,piloto13V5 | Conserva06806pero vuelve a perder06607 | USD0,001069696/15HTTP | Coste1,464×histórico; no amplía |

Los ratios de pilotos son históricos, no latencia emparejada. EnV5completa, p50base469,6ms/candidata366,3ms; p95base989,4ms/candidata871,6ms. Medición de CLIrecall, sin Codex;64pares conocidos no garantizan latencia poblacional ni completitud semántica. Las cuatro categoríasV5obtienen16/16en la candidata, pero pierde un ID permitido: esos porcentajes no prueban que se hayan conservado todas las partes.

## Corrección del mensaje de progreso y puerta unificada

Se comunicó prematuramente queV5había pasado todas las puertas tras leer el informe contra base/recursos, sin leer el resultado completo de preservación. **Era incorrecto**: perdió06607respecto de scopedV1. V4ya se había lanzado; se completó y contabilizó como campaña iniciada prematuramente, conservando sus cinco regresiones. No se ejecutóV2del banco conocido niV6. El reporte de preservación ahora incluye una calificación conjunta `passes_global_and_preservation`: requiere simultáneamente pase global y ninguna pérdida/IDajeno. Una prueba de regresión impide que un pase contra base oculte una pérdida anterior. Los informes originales permanecen intactos; la [decisión conjunta](../../outputs/iromem/evaluation/per-memory-correction-verified-v1-development-2026-10-19/decision.json) registra rechazo.

## Hallazgos con trazas

V1verificó9correccionesV5y rechazó tres: eliminó dos IDsajenos(06610y06807) pero también06607, permitido. Este último sitúa la captura con redes en otro programa explícitamente separado de Ribera; el verificador no lo aceptó como corrección de la mezcla de contextos. Gold intacto. La categoría sigue teniendo otro IDcorrecto, por eso la métrica16/16oculta esa pérdida.

V4mostró problemas previos al verificador: mezcla de rutasSarga/Linde, evento individual confundido con política general y omisión de plantilla parcial, ingredientes sin cantidades legibles y autonomía medida sin alcanzar agotamiento. Los tres útiles omitidos estaban en puesto1, no fuera del prefijo; recortar/ampliar candidatos no explica esos fallos. Los dos ajenos entraron por vía de dato=truey no recibieron comprobación. Verificar solo correcciones no cubre todo el problema.

V2explicitó parciales/límites/contexto en la propuesta y reescribió el verificador para contrastes. Recuperó06607pero el verificador rechazó06806, que corregía la autoría sin dar el año. Se había omitido del texto reescrito la regla explícita de que una corrección no necesita contestar el atributo desconocido; es una hipótesis causal, no un mecanismo interno comprobado.

V3mantiene exactamente la propuestaV2y restaura íntegro el verificadorV1con una cláusula adicional sobre contraste. Una prueba exige ese prefijo original y la igualdad del resto del payload. Conservó06806pero rechazó06607. EnV2este recuerdo tuvo dato=true/corrección=truey evitó el veto; enV3tuvo dato=false/corrección=truey fue vetado, con la misma construcción de propuesta. No se capturaron bytes de red ni se estima una tasa de variación poblacional. No basta reescribir una instrucción ni atribuir todo el error a un solo juicio.

Se detuvieron ambas revisiones antes de sus diagnósticosV4preparados;0llamadas en ellos. No se eligió una repetición favorable ni se alteraron gold, fuentes o resultados.

## Verificación, coste y pendiente

115pruebas workspace pasan; formato y compilación correctos. Cubren versiones, trazas, aislamiento, composición, recibos, conservación literal de instrucciones y puerta conjunta; no prueban acierto semántico. Todas las282operaciones nuevas completaron, sin fallos técnicos ni recibos ausentes. Coste de esta tanda: **USD0,020413716**. Acumulado reconciliado: **3239operaciones/21campañas emparejadas, USD0,228990886conocidos+3operaciones históricas de importe desconocido**, excluyendo Codex. La base temporal propia se eliminó, sin servidores ni reinicios del dashboard.

El objetivo de cinco pasos sigue sin alcanzarse. Contrato y bancos existen; falta un mecanismo que supere todas las puertas, consistencia y contraste nuevo revisado aparte. Próxima hipótesis documentada, aún no implementada: separar corrección ordinaria y contraste explícito en juicios tipados, medir su coste, y conservar los fallosV4como puerta obligatoria de propuesta/contexto. No asumir que esa separación resolverá los juicios erróneos o la variación. V6congelada permanece con0llamadas.

La autorización general ya está resuelta. El registro del goal consultado sigue `blocked` desde el permiso anterior; la herramienta de goals no admite cambiarlo a activo. La investigación avanzó en este turno, pero reanudar automáticamente ese registro requiere una acción del usuario en la app. Esto no es una nueva solicitud de permiso para pruebas.

[Diagnóstico y contabilidad por diario](../../outputs/iromem/evaluation/per-memory-correction-verified-v1-development-2026-10-19/diagnosis.json), [V5ycalificación conjunta](../../outputs/iromem/evaluation/per-memory-correction-verified-v1-development-2026-10-19/paired-v5/qualification.json), [V4](../../outputs/iromem/evaluation/per-memory-correction-verified-v1-development-2026-10-19/paired-v4/report.json), [pilotoV2](../../outputs/iromem/evaluation/per-memory-correction-verified-v2-pilot-2026-10-19/report.json), [pilotoV3](../../outputs/iromem/evaluation/per-memory-correction-verified-v3-pilot-2026-10-19/report.json).

Actualización posterior: el goal ya figura activo; no hace falta reactivación adicional. V4implementó vías de corrección tipadas y falló por dos datos de contexto ajeno; la autorización general sigue vigente. [Estado actual](per-memory-correction-verified-v4-results-2026-10-19.md).
