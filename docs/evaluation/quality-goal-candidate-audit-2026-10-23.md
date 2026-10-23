# Auditoría de candidatas del objetivo de calidad — 2026-10-23

## Decisión

No hay una candidata lista para pasar a la siguiente comparación ciega. Dos auditorías independientes de solo lectura concluyeron que la evidencia disponible no identifica un error confirmado de calidad en la ruta vigente Decisions + `baseline` que justifique desplegar otro selector. No se cambió código, datos, etiquetas, configuración ni resultados; no se llamó a proveedores ni se abrió V6/holdout. Los dos revisores son agentes Codex de la misma familia, no árbitros humanos ni independientes por familia.

## Evidencia contrastada

- En las tres muestras recientes de Decisions + `baseline`, la selección coincidió con las etiquetas vigentes en V7 exploratoria (64/64), la revisión retrospectiva M5 v2 (9/9) y la matriz de versiones (8/8). No prueban calidad poblacional; tampoco revelan un fallo que una candidata pueda corregir ahora.
- Los tres desacuerdos antiguos de M5 v2 no son falsos recuerdos bajo el contrato de cuatro relaciones: dos conservan información parcial y uno refuta la premisa solo dentro de esa reunión. La única degradación técnica de aquella campaña se mantiene como tal; no se convierte en error semántico ni en selección positiva.
- `utility_v1` ganó en su desarrollo conocido, pero su cohorte v2 nueva perdió tres abstenciones correctas por seleccionar otro referente; queda rechazada para promoción.
- `witness_v1` perdió evidencia útil en respuestas, refutaciones y parciales y añadió tres degradaciones técnicas; su resultado negativo sigue intacto.
- `roles_v1` empató con baseline en 162 operaciones y costó 41–115 % más. `slot_evidence_v3` no produjo una traza aceptable en sus smokes; en el único caso emparejado reportado quedó 0/1 frente a 1/1 de Decisions, con configuraciones completas distintas y muestra insuficiente.
- La señal V7 de FTS (8/56 evidencias parciales por debajo de top 1) no es un fallo demostrado de `recall`: Decisions seleccionó correctamente los 64 casos y la sonda de ranking no ejecutó el clasificador.

Fuentes: [M5 v2](conversation-m5-v2-relation-audit-2026-10-23.md), [Decisions frente a roles](quality-goal-decisions-roles-diagnostics-2026-10-23.md), [V3 local](slot-evidence-v3-local-query-slots-results-2026-10-23.md), [testigo](witness-v1-fresh-results-2026-10-19.md), [utility v1](utility-v1-cohort-v2-results-2026-10-18.md).

## Qué cambia para el trabajo siguiente

No se debe repetir el ajuste de V2/V3, `roles_v1`, `witness_v1` ni otra variante de `utility_v1`, ni convertir una mejora de ranking FTS en una mejora de `recall`. La puerta local del paso 3 sigue sin superarse. Para seguir con evidencia, hace falta crear un único conjunto de desarrollo nuevo, disjunto de V6 y de las cohortes expuestas, con las cuatro relaciones revisadas antes de ejecutar; usarlo para diseñar y validar una política completa contra `baseline` en modo local o en una campaña de desarrollo permitida. Solo un pase local íntegro —preserva cada apoyo, refutación y parcial del control y no añade selecciones ajenas— permitiría abrir después una cohorte fresca ciega. Si ningún mecanismo pasa, se conservan fallos y baseline y no se promueve.

Este informe no cierra ni reduce el objetivo de cinco pasos: registra que la siguiente puerta todavía no está satisfecha. Coste nuevo de API: USD 0. Coste de Codex: no disponible.
