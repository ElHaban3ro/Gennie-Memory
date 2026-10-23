# 0043 — Adoptar single-pass epistémico como política predeterminada

Estado: adoptada para la selección de semilla de iROMEM; Decisions sigue siendo el proveedor predeterminado.

## Contexto y decisión

La decisión 0042 habilitó `epistemic_partial_single_pass_v1` como opción explícita. V11 y su confirmación ciega V14 mostraron mejoras emparejadas en dos cohortes sintéticas independientes de 16 casos: 15/16→16/16 y 14/16→16/16. En ambos casos rescató recuerdos parciales, sin pérdidas ni IDs ajenos. El coste reportado fue 1,449× y 1,436× y la razón p95 0,838× y 0,957× frente a Decisions con `baseline`.

Se cambia `SelectionPolicy::default()` a `EpistemicPartialSinglePassV1`. El valor explícito `baseline` permanece disponible para reproducir el control y comparaciones. El proveedor no cambia: la ruta predeterminada continúa usando Decisions. Los resultados V11/V14 son pequeños y sintéticos; no prueban mejora universal ni autorizan atribuir estas cifras a recuerdos reales.

## Comparación con otros motores

La política modifica únicamente la clasificación de candidatos dentro de iROMEM. No cambia FTS5, el ranking ni las rutas de búsqueda de Engram, Lodis o Basic Memory. Se conservan sus resultados históricos de recuperación como comparativo de búsqueda, etiquetados por separado de selección de semilla. El banco no ofrece clasificación de semilla equivalente para esos tres motores, por lo que no se afirmará una comparación homogénea de esa etapa. Las nuevas métricas de Decisions se publicarán en un informe aditivo, sin reescribir ni mezclar los históricos de otras cohortes.

## Revisión

Reevaluar la decisión si una cohorte independiente de mayor tamaño muestra regresiones, si el coste relativo supera el límite operativo o si el feedback de usuarios contradice que seleccionar contexto parcial sea el comportamiento deseado. Mantener `baseline` explícito permite revertir la configuración sin retirar la política.

Fuentes: [resultados V11](../evaluation/epistemic-partial-single-pass-v1-v11-results-2026-10-23.md), [resultados V14](../evaluation/epistemic-partial-single-pass-v1-v14-results-2026-10-23.md), [comparación ampliada de búsqueda](../evaluation/broader-comparison-2026-10-16.md). Esta decisión sustituye el estado de adopción de [0042](0042-opt-in-epistemic-partial-single-pass.md).
