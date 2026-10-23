# 0015 — Selector experimental por evidencia

> Corrección de cierre, 2026-10-17: se conservan el protocolo, los datos y las puntuaciones históricas. Las selecciones discordantes con una etiqueta negativa no prueban una afirmación falsa; algunas preservan evidencia que refuta la pregunta. Las conclusiones de mejora de verdad o superioridad derivadas solo de esa abstención quedan retiradas. Véase el [cierre de M5](../evaluation/m5-closeout-2026-10-17.md).

Fecha: 2026-10-17. Estado: **evaluado y descartado para promoción**; `baseline` continúa predeterminado.

## Contexto

El segundo banco de desarrollo de `conversation` mostró cobertura top 5 de 75/75 positivos, pero Decisions seleccionó recuerdos que solo describían un recuento preliminar, una licencia solicitada o proveedores comparados ante preguntas por resultados finales. La elección semántica, no la cobertura, era el área afectada.

## Alternativas

Cambiar a Jev tras un sondeo de 16 casos elegido después de ver los errores no daba validación independiente y Jev también falló con proveedores. Una regla léxica que bloquee palabras como «final» podía rechazar hechos válidos y no representar la relación de evidencia. Se eligió probar una sola variante de instrucciones del mismo Decisions, activada por solicitud y sin alterar la búsqueda ni el predeterminado.

## Resultado y decisión

`evidence_v1` acertó 99/100 frente a 96/100 de la campaña base; dos de las tres diferencias de selección corrigieron negativos y la tercera fue un fallo de transporte anterior. El proveedor siguió eligiendo un recuerdo que decía expresamente que no se seleccionó proveedor. El [informe](../evaluation/conversation-m5-evidence-v1-results.md) separa aciertos, coste, latencia y límites. Según el [criterio previo](../evaluation/conversation-m5-evidence-v1-protocol.md), la variante no pasa a calibración. Se conserva como opción experimental explícita para reproducir la medición, sin promoverla a la ruta del producto.

## Condición de revisión

Reabrir la selección solo con una hipótesis distinta y preguntas de desarrollo nuevas, seguidas de calibración por conversación completa y etiquetas revisadas independientemente. No usar las cohortes reservadas ni el mismo error conocido para iterar prompts hasta obtener una cifra favorable.
