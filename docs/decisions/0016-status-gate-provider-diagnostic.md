# 0016 — Comprobación de respaldo y diagnóstico entre proveedores

> Corrección de cierre, 2026-10-17: se conservan el protocolo, los datos y las puntuaciones históricas. Las selecciones discordantes con una etiqueta negativa no prueban una afirmación falsa; algunas preservan evidencia que refuta la pregunta. Las conclusiones de mejora de verdad o superioridad derivadas solo de esa abstención quedan retiradas. Véase el [cierre de M5](../evaluation/m5-closeout-2026-10-17.md).

Fecha: 2026-10-17. Estado: `support_gate_v1` **descartado para calibración**; Jev **candidato a contraste nuevo**, no predeterminado.

## Contexto

La instrucción `evidence_v1` corrigió dos de tres falsos recuerdos de estado en desarrollo, pero no evitó uno que negaba expresamente la adjudicación. Se necesitaba distinguir si un segundo juicio sobre un único recuerdo bastaba o si el problema dependía del comportamiento de Decisions.

## Alternativas y prueba

Una regla léxica de negación sería frágil ante paráfrasis y ante positivos que sí preguntan por una prohibición. Se diseñó `support_gate_v1`: Decisions elige con la instrucción base y, si selecciona un ID, Decisions evalúa si ese recuerdo aislado respalda la respuesta. Se creó antes de medir un banco sintético nuevo de tres conversaciones, con negativos que tienen candidatos y positivos de control. Decisions base obtuvo 28/36. El gate falló el primer negativo del piloto de tres casos y se detuvo por el criterio previo. Como diagnóstico posterior, Jev obtuvo 36/36 con candidatos idénticos; el [informe](../evaluation/conversation-m5-support-gate-v1-results.md) muestra métricas y límites.

## Decisión y condición de revisión

No se añade gate a la ruta predeterminada ni se cambia automáticamente a Jev. Jev falló un negativo en un sondeo anterior y este banco no tiene etiquetas humanas independientes. Antes de una decisión de proveedor, fijar una comparación en conversaciones nuevas por ámbito completo, revisar etiquetas sin mostrar salidas, medir abstención, positivos, coste y latencia, y conservar una sola reserva intacta. `support_gate_v1` permanece explícito solo para reproducir el experimento; no se debe recomendar como mejora validada.
