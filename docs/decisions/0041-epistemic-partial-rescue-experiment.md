# 0041 — Cascada optativa de rescate de estado parcial

## Contexto

En V9, Decisions + `baseline` obtuvo 14/16; las dos omisiones son evidencia explícita de dato provisional frente a dato confirmado. Jev 1.13 obtuvo 12/16 y omitió las mismas consultas parciales, por lo que cambiar de proveedor no resuelve la señal observada. Las cuatro refutaciones y los cuatro controles ajenos sí quedaron correctos con Decisions.

## Decisión experimental

Añadir `epistemic_partial_rescue_v1` como política seleccionable: ejecuta `baseline` primero; si este elige, conserva ese ID y no hace otra llamada. Solo ante abstención consulta una segunda vez si algún candidato declara explícitamente un estado estimado, preliminar, pendiente, no confirmado o no registrado para el mismo referente y atributo pedido. Las dos etapas usan el mismo proveedor y candidatos; recibos conocidos se agregan y una falla de rescate degrada la operación preservando el consumo conocido inicial.

El objetivo es surfacing de evidencia parcial, no emitir la respuesta faltante. El segundo juicio sigue siendo una clasificación remota falible: no hay cita ni verdad verificada. La variante es optativa, de alcance experimental y sin cambio a `baseline`; no añade tablas, dependencias, secretos, puertos ni otro modelo. Puede duplicar la latencia y el coste de cada caso donde la primera etapa se abstenga.

## Resultado y reversión

El [protocolo V1](../evaluation/epistemic-partial-rescue-v1-protocol-2026-10-23.md) fijó antes de inferencia la cohorte V9, sus hashes y puertas: rescatar dos parciales, preservar los 14 IDs del baseline, dejar vacíos cuatro negativos, cero degradaciones y recibos completos, con coste y p95 dentro de 1,5×. La campaña obtuvo 16/16 exactas, cero pérdidas y cero extras, pero costó 1,557× y p95 de servicio 2,230×. V9 ya era conocido y el resultado no es confirmación independiente. Se conserva el diario y se retira la opción del runtime; no se reintenta ni ajusta V1 en V9.

Decisión: rechazar V1 para producto por incumplir recursos; `baseline` sigue siendo la única ruta vigente. La [medición completa](../evaluation/epistemic-partial-rescue-v1-results-2026-10-23.md) conserva calidad, coste, latencia, hashes y límites. Una candidata posterior debe evitar llamadas secundarias generales a los controles ajenos y pasar primero una puerta local en datos de desarrollo nuevos.
