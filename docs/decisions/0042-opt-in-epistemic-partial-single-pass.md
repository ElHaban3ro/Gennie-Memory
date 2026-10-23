# 0042 — Habilitar single-pass de evidencia parcial como política optativa

Estado: adoptada como opción explícita; `baseline` sigue predeterminado.

## Contexto

La cascada `epistemic_partial_rescue_v1` alcanzó calidad alta, pero excedió los límites de coste y p95; se retiró. La política de una llamada `epistemic_partial_single_pass_v1` empató con baseline en V10, luego mejoró casos parciales sin pérdidas en V11 y confirmó el resultado en V14.

## Evidencia y decisión

En V11, Decisions baseline quedó en 15/16 y single-pass en 16/16, sin IDs ajenos ni regresiones; coste 1,449× y p95 0,838×. La revisión fue ciega a la clave y la candidata rescató una respuesta parcial. En V14, una nueva cohorte revisada a ciegas obtuvo 14/16 frente a 16/16: rescató dos parciales, conservó los otros 14 resultados, dejó vacíos los cuatro negativos y no degradó ningún caso. Coste 1,436× y p95 0,957×, por debajo del límite 1,5×. Los dos informes conservan hashes, recibos, fallos por caso y límites sintéticos.

Se conserva la política como opción seleccionable en `RecallRequest.selection_policy`; no se cambia el valor predeterminado. La mejora observada es pequeña y de alcance parcial, con coste por consulta aproximadamente 44 % mayor. El desempeño fuera de estos bancos no está probado. Cambiar el predeterminado requerirá evidencia adicional de producto y no forma parte de esta decisión.

## Revisión

Revisar si una cohorte representativa o una medición por usuario muestra regresiones, una relación coste/calidad distinta o que los parciales no deban seleccionarse por defecto. V12/V13 fueron rechazadas antes de proveedor por fallos de gold/revisión; V14 corrigió el set y congeló hashes antes de las llamadas. [Resultados V14](../evaluation/epistemic-partial-single-pass-v1-v14-results-2026-10-23.md) · [Resultados V11](../evaluation/epistemic-partial-single-pass-v1-v11-results-2026-10-23.md).
