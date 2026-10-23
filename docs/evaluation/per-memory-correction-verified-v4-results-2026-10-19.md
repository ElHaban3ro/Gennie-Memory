# Verificación selectiva v4: correcciones tipadas y límite de contexto

Estado2026-10-19: mecanismo optativo implementado y auditado, **sin aprobación para ampliar**. V5pasa su piloto conservador, pero el diagnóstico obligatorioV4mantiene dos IDsajenos. Ninguna configuración predeterminada o gráfica del READMEse cambia;V6sigue con0llamadas. El goal vuelve a figurar activo; la autorización general de pruebas está resuelta.

La comprobación aislada ahora contiene dos choices: corrección ordinaria con instrucciónV1íntegra y contraste explícito entre contextos. `CheckedMemory.paths` conserva ambos juicios y `useful` debe ser su OR. Servicio y auditor JSONvalidan campos/versión/IDs/revisiones/composición. Ambas respuestas usan un solo recibo global: coste y tokens se cuentan una vez. Las rutas antiguas omiten ese campo y no cambian su traza histórica. No se añade SQL, secretos, dependencias o servicios. [Contrato previo a llamadas](per-memory-correction-verified-v4-protocol-2026-10-19.md).

| Tanda conocida | Resultado de calidad | Coste / HTTP | Latencia CLI |
| --- | --- | --- | --- |
| V5:13consultas | 0IDajeno,0pérdidas de base/scopedV1; pasa puerta conservadora.9/13sets exactos | USD0,001055496;14HTTP;1,4445×subconjunto histórico | p50374,5ms;p95941,4ms |
| V4:5fallos anteriores | Recupera los5IDspermitidos; añade2IDsajenos.3/5sets exactos; falla | USD0,000399504;5HTTP;1,1424×subconjunto histórico | p50365,2ms;p95476,0ms |

Son tandas seleccionadas de desarrollo, sin nueva revisión ciega ni latencia emparejada. La exactitud de conjuntos y omisiones se conserva junto al pase conservador; no significa verdad final, completitud semántica o rendimiento poblacional.

EnV5solo se verificó la corrección válida de autoría, por vía ordinaria. El contraste06607entró por dato=truey evitó el check. No hubo un contraste positivo observado en el verificador; ese piloto no prueba su estabilidad o que la nueva vía haya reparado aquel error. Las pruebas locales cubren contraste solo y OR, sin convertir mocks en prueba semántica.

EnV4se recuperaron los tres parciales antes omitidos: plantilla incompleta, ingredientes con cantidades ilegibles y autonomía medida sin agotar la batería. Los otros dos errores persisten: el tramo de otra rutaLindeyun ensayo individual tomado como política general. Ambos tuvieron dato=trueycorrección=false, por lo que ningún verificador de correcciones puede rechazarlos. El informe de5/5IDsútiles ocultaría la contaminación: hay7IDsseleccionados,2no permitidos. Se detuvo antes de campañas completas oV6.

Siguiente hipótesis, aún no implementada: separar en la propuesta un juicio sobre el contexto del dato y una elección sobre qué evidencia aporta(facto/parte, corrección o ninguna). Aplicar contexto solo a datos, conservando la corrección de premisas por su propia vía; evitar el veto global de versiones antiguas. Una tarea multiopción podría mantener dos preguntas por recuerdo, pero su coste/calidad deben medirse. El contexto debe excluir atributos desconocidos preguntados y totales no disponibles; no asumir que otro juicio del mismo modelo es independiente o correcto. Mantener los13/5casos conocidos, puerta conjunta y V6intacta.

117pruebas workspace pasan, formato y compilación correctos. Verifican versiones/omisión de campos antiguos, ORtipado alterado, contraste solo, respuestas ausentes con consumo conocido y recibo único. Las18operaciones nuevas/19HTTPcompletaron con recibos: **USD0,001455de proveedor**, sin coste desconocido nuevo. Acumulado: **3257operaciones/21campañas emparejadas, USD0,230445886conocidos+3operaciones históricas de importe desconocido**. Excluye Codex de programación/revisión. DB temporal propia eliminada, sin servidores ni reinicios del dashboard.

El objetivo de cinco pasos permanece activo y sin completar: faltan mecanismo aprobado, consistencia, contraste nuevo y adopción demostrada. [Decisión conjunta](../../outputs/iromem/evaluation/per-memory-correction-verified-v4-development-2026-10-19/decision.json), [V5con costes/omisiones/trazas](../../outputs/iromem/evaluation/per-memory-correction-verified-v4-pilot-2026-10-19/metrics.json), [V4con contaminación visible](../../outputs/iromem/evaluation/per-memory-correction-verified-v4-v4-targeted-2026-10-19/report.json).

Actualización posterior: la separaciónV5se implementó y falló el diagnóstico conocido; los contextos falsamente compatibles persisten. [Estado vigente](per-memory-correction-verified-v5-results-2026-10-19.md).
