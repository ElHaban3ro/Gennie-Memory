# Piloto de verificación selectiva: resultados

Ejecutado2026-10-19 tras autorización explícita:12preguntas sintéticasV5 conocidas, una ejecución por consulta. **Pasa la puerta conservadora del piloto; no demuestra todavía completitud, estabilidad ni rechazo de propuestas erróneas.** Gold original y configuración predeterminada conservados.

| Medida | Resultado |
| --- | --- |
| Operaciones completas / degradadas | 12/12;0degradadas |
| IDs seleccionados no permitidos | 0 |
| IDs útiles perdidos frente a base / scopedV1 históricas | 0/0 |
| IDs útiles adicionales frente a base histórica | 3 |
| IDs permitidos seleccionados / total gold | 12/16(75%) |
| Sets completos de IDs | 8/12(66,7%) |
| Clasificación dentro del prefijo | 12TP,69TN,3FN,0FP |
| HTTP reales | 13:12propuestas+1comprobación |
| Comprobaciones aceptadas / rechazadas | 1/0 |
| Coste informado completo | USD0,000939864 |
| Base histórica, mismo subconjunto | USD0,000667500; ratio1,408× |
| ScopedV1 histórica, mismo subconjunto | USD0,001015476; ahorro retrospectivo7,45% |
| Latencia CLI p50 / p95 | 361,3ms /1004,2ms |

Se conservaron los tres negativos vacíos y evidencia en las nueve preguntas útiles. Las tres ganancias contra la base son06714,06812y06912; el último contexto discutido se conserva conforme al gold original, sin resolver la discrepancia mediante relabelado. Los12sets coinciden con scopedV1histórica: hay preservación y menor coste de ese subconjunto, no una mejora semántica observada contra aquella versión. Coste/latencias históricas no son comparación emparejada ni prueban causalidad.

Cuatro IDs permitidos no se seleccionaron:06811estaba en puesto13, fuera del prefijo7;06914,07011y06911estaban dentro y recibieron ambos juicios negativos. Los dos primeros describen preparación/trámite; esa discrepancia con la rúbrica actual se registra sin cambiar el gold.06911describe una hora de funcionamiento, una parte concreta de la autonomía pedida: la omisión sigue siendo una limitación real. No se oculta usando la presencia de otro ID de la misma categoría. V5carece de cobertura por parte prospectivamente anotada; la puerta de preservación no certifica completitud semántica.

En06803se rechazó el ID de otro libreto en la propuesta original; solo se verificó la corrección válida06806. **Ninguna propuesta incorrecta llegó al verificador en esta corrida.** No atribuirle el rechazo ni afirmar que resuelve aquel error variable. Una sola ejecución tampoco demuestra estabilidad.

El coste tiene recibos de las13etapas, sin subtotal tratado como total. p50/p95incluyen la ruta CLI de recuperación/clasificación/composición de estas12operaciones, sin Codex; no se comparan como latencia emparejada con históricos. Muestra pequeña, sintética y conocida, sin nueva revisión humana/ciega.

112pruebas workspace pasaron antes del piloto; tras ampliar solo el auditor de preservación para la futura comparación completa,49pruebas del evaluador y formato/compilación pasaron. El binario de producto se conserva idéntico al piloto. DB temporal propia eliminada, ningún servidor iniciado ni dashboard reiniciado. La autorización de este piloto fue satisfecha; no se usó para enviar las52preguntas adicionales del banco completo.

Acumulado reconciliado:2957operaciones,19campañas emparejadas, **USD0,208577170conocidos+3operaciones de importe desconocido**. Excluye Codex de programación/revisión. El objetivo de cinco pasos no está completado: falta demostrar el mecanismo completo, estabilidad, contraste nuevo y decisión de adopción.

Siguiente comparación preparada, sin llamadas:64preguntasV5/128operaciones con Decisions/base y candidata, mismo binario/etiquetas, orden alternado. EstimaciónUSD0,009 y2–3minutos, máximo576HTTP; necesita autorización explícita de ese alcance. Los omitidos permanecen visibles y no se declara completitud por pasar una puerta conservadora. [Protocolo completo](per-memory-correction-verified-v1-development-protocol-2026-10-19.md).

[Informe Rust auditado](../../outputs/iromem/evaluation/per-memory-correction-verified-v1-pilot-2026-10-19/report.json), [métricas y omisiones](../../outputs/iromem/evaluation/per-memory-correction-verified-v1-pilot-2026-10-19/metrics.json), [latencias/concordancia exacta](../../outputs/iromem/evaluation/per-memory-correction-verified-v1-pilot-2026-10-19/campaign-report.json), [próxima comparación preparada](../../outputs/iromem/evaluation/per-memory-correction-verified-v1-development-2026-10-19/paired-v5/preflight.json).

Resultado posterior: la comparación completa y revisiones fallaron la puerta conjunta; no se promueve el pase de este piloto. [Estado vigente](per-memory-correction-verified-results-2026-10-19.md).
