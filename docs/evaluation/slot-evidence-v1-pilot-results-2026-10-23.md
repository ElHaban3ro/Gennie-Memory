# Resultados del smoke: slot evidence V1

Fecha: 2026-10-23. Prueba exclusivamente sintética, anterior a cualquier puntuación. El protocolo congelado está en [diseño](slot-evidence-v1-design-2026-10-23.md).

## Resultado

**No se abrió TRAIN.** G0 (Gemma 3 4B con la ruta baseline) completó el ejemplo sintético, seleccionó el único candidato y devolvió un recibo completo. G1 (Gemma 3 4B con `slot_evidence_v1`) falló la validación estructural en 10 de 10 intentos: recibió recibo completo en todos, pero ninguna respuesta se aceptó ni seleccionó. No se cuentan como aciertos, errores semánticos ni abstenciones válidas. Se detuvo antes de invocar el corpus TRAIN.

Decisions y Jev devolvieron HTTP 400 en los dos preflights anteriores. No hay recibos completos para esas llamadas y su coste queda **desconocido**, no cero.

## Métricas y coste

| Brazo | Intentos | Respuestas válidas | Selecciones | Tokens entrada/salida | Coste informado |
| --- | ---: | ---: | ---: | ---: | ---: |
| G0 Gemma baseline | 1 | 1 | 1/1 | 115 / 6 | US$0,00000490 |
| G1 Gemma slot evidence | 10 | 0 | 0/10 | 3.569 / 3.102 | US$0,00029786 |
| **Subtotal conocido** | **11** | **1** | **1/11** | **3.684 / 3.108** | **US$0,00030276** |
| Decisions / Jev preflight | 2 | 0 | 0 | desconocidos | **desconocido** |

El coste de las llamadas con recibo se suma desde `usage.cost` del proveedor. El piloto TRAIN, por tanto, tuvo **0 llamadas y US$0 de coste informado**; esto no cambia el coste desconocido de los dos HTTP 400 anteriores.

## Motivos de rechazo G1

Los códigos locales de las 10 respuestas fueron: `slot_evidence_invalid_trace` (1), `slot_evidence_identity_binding_invalid` (2), `slot_evidence_unresolved_identity_has_match_fields` (1), `slot_evidence_roster_or_bounds_invalid` (1), `slot_evidence_query_anchor_invalid` (1), `slot_evidence_query_anchor_omits_stable_id` (1), `slot_evidence_identity_anchor_invalid` (1), `slot_evidence_roster_count_mismatch` (1) y `slot_evidence_query_anchor_stable_id_not_literal` (1). Los códigos más generales corresponden a intentos hechos antes de compilar diagnósticos más específicos. Dos smokes se hicieron durante una comprobación temporal de citas separadas; ambos igualmente fallaron por otras reglas estructurales y ninguno pasó el contrato conservador final. Cada respuesta conservó el error atómico y no filtró una selección parcial.

## Interpretación y siguiente paso

El resultado actual no demuestra que la idea de extracción por slots mejore o empeore la calidad: G1 no logró producir trazas que cumplan el contrato en el smoke. Sí demuestra que el gate estructural funciona y que el baseline Gemma completa esta ruta sencilla. Las diferencias repetidas de anclas, identidad y roster hacen que una campaña de 432 casos carezca de valor hasta corregir la salida.

No se modificaron etiquetas ni gold, no se consultaron las particiones reservadas, no se ejecutó una revisión ciega y no se estimó calidad a partir de estos smokes. La implementación sigue experimental, sin adopción ni cambio del proveedor predeterminado. El siguiente ciclo debe reducir la complejidad de la serialización o dividir la extracción en etapas y repetir primero un smoke sintético con recibo; cualquier cambio de paquete requiere crítica del protocolo antes de puntuar TRAIN.
