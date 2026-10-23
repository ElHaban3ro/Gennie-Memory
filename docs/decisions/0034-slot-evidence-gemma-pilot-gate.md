# 0034 — Gate técnico de extracción de evidencia por slot

## Contexto

`witness_v1` valida citas y bindings estructurados, pero no exige que el recuerdo conteste el atributo pedido. Se preparó `slot_evidence_v1`, una extracción remota estructurada por consulta con anclas, slots, premisas, proposiciones y relaciones explícitas. El experimento usa solo fixtures sintéticos a través de OpenRouter; no se comprobó la retención remota, por lo que no se permite contenido de usuarios.

## Alternativas

- Puntuar TRAIN inmediatamente tras compilar el selector.
- Aceptar respuestas parcialmente estructuradas y completar roster o referencias localmente.
- Exigir un smoke válido, conservar la degradación atómica y no abrir TRAIN hasta tener salida estructurada estable.

## Decisión y estado

Se eligió el gate fail-closed. El baseline Gemma G0 completó el único ejemplo sintético y devolvió recibo. G1 falló la validación estructural en 10/10 intentos, aunque todos devolvieron recibos completos. Los intentos variaron entre incumplimientos de ancla, identidad y roster; no se cuentan como métricas semánticas. Decisions y Jev también fallaron con HTTP 400 en preflights anteriores, sin recibos completos. El coste conocido del smoke fue US$0,00030276; dos solicitudes sin recibo conservan coste desconocido. No se ejecutó TRAIN, no se abrieron reservas, no se modificó gold y no se promovió la política.

El contrato mantiene `slot_evidence_v1` como opción explícita no predeterminada y los datos sintéticos como único ámbito autorizado de este piloto. [Diseño y protocolo](../evaluation/slot-evidence-v1-design-2026-10-23.md), [informe de métricas/coste](../evaluation/slot-evidence-v1-pilot-results-2026-10-23.md).

## Revisión

Revisar tras rediseñar la interfaz de extracción (por ejemplo, extracción en etapas o formato más simple) y volver a pasar una crítica de contrato. Antes de puntuar TRAIN se requiere un smoke estructural repetible, recibo completo y alcance sintético fijado. El resultado actual no permite concluir mejora ni regresión de calidad.
