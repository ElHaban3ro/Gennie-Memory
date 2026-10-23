# Contraste de versiones Jev: sin mejora de calidad

Fecha2026-10-19. Diagnóstico sintético conocido, no validación nueva ni campaña completa contra Decisions/base. [Método fijado antes de llamadas](jev-version-probe-protocol-2026-10-19.md).

Se implementó el proveedor explícito `jev114` para `typesafe/jev-1.14`; `jev` conserva1.13 y Decisions sigue predeterminado. El payload probado es idéntico salvo `model`. No hay variables, secretos, dependencias, puertos ni conexiones nuevas en apps.96pruebas workspace pasan, incluido el contrato de versión, deserialización y rechazo por el runner de proveedor contradictorio. CLI real con DB temporal propia, ya eliminada.

Diez preguntas/20operaciones alternadas. Ambas versiones devolvieron **exactamente las mismas selecciones en10/10pares**: cero ganancias, un caso con ID no permitido —el carné ante una pregunta por avalista—, cero degradaciones y recibos completos. Misma lista de candidatos comprobada por par. El modelo servido distingue `typesafe/jev-1.13-20260917` y `typesafe/jev-1.14-20261014`; la coincidencia no demuestra que sean internamente el mismo modelo.

| Medición del runner | Jev1.13 | Jev1.14 |
| --- | ---: | ---: |
| Operaciones |10|10|
| Coste reportado USD |0,000566076|0,000566076|
| Mediana ms |348,40|347,52|
| p95 ms |620,62|361,49|

Recursos pasan el límite diagnóstico1,5×. La cola medida de1.14 es menor en esta muestra pequeña y no compensa la falta de mejora de calidad. No es garantía de latencia, comparación con el predeterminado ni tiempo de respuesta final de Codex.

Se detiene antes de ampliar: falla el criterio previo de al menos una ganancia y cero IDs ajenos. La auditoría Rust pasó sin proyección de auxiliares. Los conteos usan gold original; los diez desacuerdos de contexto se conservan como interpretación retrospectiva independiente, no como nuevos aciertos. La [rúbrica v2](evidence-context-rubric-v2.md) solo guía futuros bancos y no cambia V6 ni sus etiquetas. Una preparación, trámite o planificación no establece por sí sola el dato solicitado; se mantiene como contexto distinto de falsedad. Los dos roles discrepantes siguen parciales para la pregunta multipartes, sin modificación al gold.

Coste de esta tanda: **USD0,001132152**. Acumulado del objetivo **2565operaciones,17campañas emparejadas, USD0,180465124 conocidos y dos operaciones de importe desconocido**. Excluye programación/revisión de Codex. Ningún resultado se promueve. V6 sigue con cero llamadas. [Datos, recibos y método de cálculo](../../outputs/iromem/evaluation/jev-version-probe-2026-10-19/report.json).

Siguiente investigación: evaluar una formulación con ejemplos contrastivos generales que distingan dato solicitado de dato asociado, preservando parciales y refutaciones; aislar el cambio de instrucción, no seguir alternando modelos sin señal. Antes de llamar: fijar ejemplos sin IDs/respuestas de los casos, casos/puertas y costes. Mantener también la puerta del gold original y no aprobar una pérdida bajo el argumento de que era contexto discutido. Aún faltan pase completo de desarrollo, validación nueva y adopción para cerrar el goal.
