# 0044 — Restaurar baseline como política predeterminada

Estado: adoptada por instrucción del usuario el 2026-10-23; sustituye la [decisión 0043](0043-epistemic-single-pass-default.md).

## Contexto y decisión

El contraste emparejado de 540 preguntas con Decisions obtuvo 441/460 positivas con `baseline` frente a 401/460 con `epistemic_partial_single_pass_v1`. Ambos dejaron vacías 80/80 negativas, sin degradaciones ni IDs prohibidos y con candidatos idénticos. Baseline costó USD 0,029742 frente a USD 0,035898; p95 total 863 frente a 950 ms. Las mejoras de V11/V14 no se sostuvieron en esta cohorte sintética más amplia.

Se restaura `SelectionPolicy::default()` a `Baseline`: omitir `selection_policy` vuelve a elegir la configuración anterior. Decisions sigue como proveedor predeterminado. Single-pass permanece disponible explícitamente como experimento; no se eliminan resultados ni políticas. La alternativa de mantener la promoción se descarta por la pérdida observada y la decisión del usuario.

El evaluador conserva el arreglo que envía la política explícitamente, incluida baseline. La corrida que omitió el campo se mantiene identificada como inválida para comparación. No se repiten campañas remotas ni se alteran los resultados históricos de búsqueda de otros motores.

## Verificación y revisión

La prueba del contrato cubre `RecallRequest` sin política, proveedor implícito y políticas explícitas. Se verifica el workspace y el formato sin nuevas llamadas remotas. Una promoción futura requiere evidencia comparativa adicional; esta decisión no declara calidad general en datos reales.

Fuente: [comparación completa, diarios y costes](../evaluation/epistemic-single-pass-competitor-comparison-2026-10-23.md).
