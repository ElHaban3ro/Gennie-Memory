# Gemma 4B: filtro fallido e interrupción por HTTP429

Fecha:2026-10-22. La prueba ejecutó **3de16HTTP**:dos respuestas válidas y una respuesta HTTP429 sin recibo de consumo. En los **8juicios disponibles**, cinco roles correctos, tres erróneos: **dos evidencias útiles perdidas y una selección ajena**. La candidata falla calidad independientemente del429. No se reintenta ni amplía esta versión. No es una tasa global sobre64juicios;13preguntas quedaron sin ejecutar.

## Cambio ejecutado y evidencia

Se implementó una familia de evaluación `gemma` reutilizando el contrato generativoV2: Gemma3-4B-it exclusivamente en Parasail, una llamada por pregunta, JSON cerrado con rolesA/B/C/none, sin respuestas/citas generadas. Mensajes, rúbrica, fuentes, orden y esquema permanecen idénticos al contraste anterior; solo cambian modelo/proveedor y se retira reasoning no declarado por el endpoint. No es una ablación pura de pesos. Alias y proveedor fueron confirmados en ambas respuestas; no hay revisión de pesos inmutable demostrada.

La revisión previa detectó y corrigió dos controles antes de red: exigir presupuesto en el informe además del runner, y exigir ID/tokens/coste completos en recibos. Modelo/proveedor inesperados, campos/enums/cardinalidad inválidos, truncamiento o recibo incompleto bloquean el pase y detienen nuevas llamadas; cualquier cargo conocido se conserva. El fallback está deshabilitado y no existe bucle de reintento. La prueba ignorada de transporte terminó correctamente porque registró y detuvo el429; eso **no** es un pase de calidad.

El caso1 pierde un aporte parcial y acepta una fuente ajena al dato solicitado; el caso2 pierde una refutación explícita. Las fuentes, roles y errores están completos en `report.json` y `inputs.jsonl` de los artefactos. El informe no transforma el429 en `none` ni en un error semántico adicional. La revisión de fallos separa observaciones de hipótesis sobre capacidad/instrucciones.

## Métricas de esta tanda

| Métrica | Resultado |
| --- | ---: |
| Peticiones ejecutadas |3/16|
| Respuestas clasificables |2|
| Preguntas no ejecutadas |13|
| Roles correctos observados |5/8|
| Errores de rol observados |3|
| Evidencias útiles perdidas |2|
| Selecciones ajenas |1|
| Coste API conocido |USD0,000033370|
| Peticiones sin importe informado |1|
| Tiempo HTTP de las dos respuestas |1379,61ms;858,11ms|
| Tiempo de la petición429 |519,65ms|
| Pase de calidad |No|
| Comparación contemporánea con baseline |No ejecutada|
| Recursos completos/p50/p95 de producto |No medidos|
| Adopción/V6 |No|

La estimación conservadora previa fueUSD0,00269048 para16peticiones, bajo el máximoUSD0,005. Los importes reales anteriores proceden de los recibos; el429 tiene coste desconocido, no cero. Coste/tokens Codex de desarrollo no cuantificados. Los tres tiempos HTTP no representan latencia de recuperación completa ni permiten acreditar estabilidad o colas.

## Estado del objetivo y estrategia

Los cinco pasos siguen activos; contrato y bancos existen, mecanismo elegible pendiente. La candidata documental se congela como fallida con64/116completas y fieles, sin modificar all116, resultados o etiquetas. La ampliación nominal propuesta queda diferida. El contrato all116 pertenece a esa candidata; no se impone como requisito a todo recuperador. Las puertas generales conservan cero selecciones ajenas/pérdidas/degradación nueva, al menos una ganancia, recibos completos y coste/p50/p95≤1,5×baseline, estabilidad y validación ciega final.

Gemma tampoco es elegible. El fallo confirma que un modelo pequeño distinto y salida cerrada no bastaron en los dos casos observados; no prueba que todos los clasificadores o toda selección desde texto fallen. No se cambia producto/default, README/gráficas de rendimiento, DB, dependencias, credenciales o servicios. No hubo commit/push. La reservaV6 conserva su hash y no fue leída.

La continuación debe atacar los errores semánticos documentados con una hipótesis comprobable; no reintentar el429 ni añadir ejemplos del banco a esta versión. La hipótesis de dos clasificadores/segmentos permanece documental y exige viabilidad antes de invertir en transporte. No hay permisos humanos pendientes.

## Verificación y trazabilidad

867pruebas pasan en default(24omitidas) y con HTTP/2(69omitidas),0fallos; la mayoría de omitidas son pruebas remotas manuales. Doce informes históricos afectados se reconstruyen byte por byte idénticos. Las425reproducciones documentales de la fase anterior permanecen históricas, no se presentan como repetidas en esta fase. El control nuevo compara payloads completos para exigir la lista exacta de cambios, y rechaza identidades, roles y recibos incompletos. Las pruebas no acreditan pertinencia semántica.

Artefactos: `outputs/iromem/evaluation/contribution-gemma-minimal-v1-2026-10-22`:preparación, requests, fuentes/binario ejecutados, diario, runner-status, informe, presupuesto, compatibilidad y logs. [Protocolo congelado](gemma-classifier-v1-protocol-2026-10-22.md), [crítica previa](gemma4b-capacity-screen-review-2026-10-22.md), [revisión de fallos](gemma-classifier-v1-failure-review-2026-10-22.md), [decisión estratégica](quality-goal-strategy-critique-2026-10-22.md).

Acumulado externo:6132operaciones;23campañas emparejadas completas —esta tanda no es una—; **USD4,565449878conocidos +21operaciones de importe desconocido**, excluye Codex. Historial anterior6129/USD4,565416508/+20 permanece intacto. No sumar esta prueba a una cohorte histórica para publicar un rendimiento nuevo.
