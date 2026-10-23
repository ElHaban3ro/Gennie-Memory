# Slots V3: derivación local y esquema restringido — 2026-10-23

## Resultado

**No mejora demostrada; V3 no se adopta.** Las correcciones trasladaron al esquema restricciones que ya podía verificar localmente —cantidad/ID de slots, IDs estables literales presentes en la consulta, una forma acotada de premisa relacional y referencias de destino permitidas—, pero la extracción de Gemma volvió a fallar. En el último smoke entregó una cita exacta de la cláusula de Fenn, aunque marcó la proposición `unrelated` y la enlazó al único slot; el validador rechazó la combinación y no seleccionó. No se convirtió esa salida en respuesta mediante una regla léxica.

Dos lectores ciegos coincidieron en la última fixture: **Northvale** corresponde a Botanist Fenn Arlo F-21; **Mariner’s Reach** corresponde a Marine archaeologist Luisa Carden M-88. La búsqueda FTS5 recuperó el único recuerdo esperado. Corrijo la comparación inicial: el control de USD 0,00000726 era Gemma con política baseline, pero el proveedor predeterminado real es Decisions. Ejecuté el mismo caso con la configuración vigente: V3 Gemma fue **0/1 selecciones** y baseline Decisions **1/1 recuerdos seleccionados**. El modelo y proveedor difieren porque se comparan las configuraciones completas; es una señal emparejada sobre pregunta/candidato, no una atribución causal a la política ni una tasa de calidad poblacional. Decisions seleccionó el recuerdo completo; no produjo una respuesta aislada ni adjudicó semánticamente la ciudad.

## Secuencia de pruebas

| Fixture | Revisión ciega | Resultado V3 | Tokens entrada/salida | Coste | Clasificador |
| --- | --- | --- | ---: | ---: | ---: |
| `local-query-slots-v1` | 2/2; Mariner’s Reach para Luisa | Falló `slot_evidence_v3_slot_count_unresolved`: dos IDs de slot para una pregunta de un solo slot | 1.115 / 323 | USD 0,00008805 | 4.253,75 ms |
| `local-query-slots-v2` | 2/2; Northvale para Fenn, Mariner’s Reach para Luisa | Tras fijar cantidad/IDs de slots, falló `slot_evidence_v2_stable_id_not_literal`: el modelo propuso Northvale como otra ancla/ID de la consulta | 1.156 / 321 | USD 0,00008990 | 4.081,11 ms |
| `local-query-slots-v3` | Desacuerdo de granularidad: ubicación de trabajo o ciudad | Excluida antes de API; no es puntuable | — | USD 0 | — |
| `local-query-slots-v4` | 2/2; Northvale para Fenn | Falló `slot_evidence_proposition_non_actionable_target_invalid`: el modelo etiquetó `unrelated` una proposición enlazada al slot; cero selección | 1.207 / 162 | USD 0,00007655 | 3.006,59 ms |
| Control Gemma + baseline (no predeterminado) | Misma adjudicación ciega | Completó y seleccionó el recuerdo esperado; no aisló una respuesta | 174 / 6 | USD 0,00000726 | 675,49 ms |
| Configuración vigente: Decisions + baseline | Misma adjudicación ciega | Completó y seleccionó el recuerdo esperado; modelo servido `openai/gpt-6-luna-decisions-20261006` | 245 / 0 | USD 0,00002450 | 1.093,82 ms |

Los tres requests V3 recibieron HTTP 200 de DeepInfra y recibo completo. Se detuvo cada revisión tras su única llamada; no hubo reintentos. **Coste de esta tanda remota (3 V3 + 2 controles): USD 0,00028626.** Frente a la configuración vigente Decisions, V3 costó 3,12× y su clasificación tardó 2,75× en este caso. El control Gemma también costó menos, pero no es el predeterminado. Acumulado separado con los dos smokes anteriores de derivación local del sujeto (`local-name-span-v1/v2`): USD 0,00042401 en siete llamadas; no mezclar como cohorte.

Los detalles/revisiones y capturas están en [`assets/slot-evidence-v3/local-query-slots-v1/`](assets/slot-evidence-v3/local-query-slots-v1/), [`.../local-query-slots-v2/`](assets/slot-evidence-v3/local-query-slots-v2/) y [`.../local-query-slots-v4/`](assets/slot-evidence-v3/local-query-slots-v4/). La fixture V3 descartada conserva su crítica ciega sin salida del modelo. Las capturas separan hash/recibo de `message.content`, usan fixtures sintéticas y permisos `0600`.

## Verificación local y decisión

La prueba focal de esquema/parsing pasa: 24 casos compactos. `cargo fmt --all --check` y `cargo test --offline --workspace` pasan; recuento final: bootstrap `iromem` 234 aprobadas/24 ignoradas, otro target 4 aprobadas, evaluación 700 aprobadas, kernel 6 aprobadas y retrieval 95 aprobadas/1 ignorada; **1.039 aprobadas, 25 ignoradas, cero fallidas**.

El mecanismo local es estructural y acotado. Reconoce IDs estables con forma ASCII mayúscula-guion-dígito y premisas relacionales en patrones ingleses `What did ...` y españoles `Qué/Que ...` con verbos de comunicación. IDs con otros formatos, premisas fuera de esos patrones y preguntas complejas pueden quedar omitidos o rechazados. Un parseo válido tampoco demuestra que el predicado responda ni que el modelo haya interpretado la consulta correctamente.

No se cambia baseline ni se promueve V3. La comparación real con Decisions completa el control de un caso, pero no el gate de calidad. Paso siguiente recomendado: dejar de ajustar el wire de V3 por error individual y probar una política distinta en datos sintéticos frescos y ciegos, contra Decisions, con control de configuración declarado y presupuesto de llamadas congelado antes de red. El objetivo de calidad sigue activo y no cumplido.

## Diagnóstico retrospectivo del siguiente candidato

Una revisión independiente propuso centrar el siguiente estudio en atribución de sujetos dentro de complementos reportados: una oración como «Bob dijo que Carlos vive en Lima» no acredita que Bob viva en Lima. El guard local de V3 ya detecta formas acotadas de ese patrón. Pasaron tres pruebas focales existentes: exclusión de sujeto extranjero en complemento reportado, conservación de evidencia cuando se pregunta por el contenido comunicado y los controles prospectivos de sujeto/oración. Esto confirma comportamiento local en trazas tipadas, no extracción automática del clasificador.

El límite decisivo es de integración: `baseline` devuelve selección de recuerdos completos y no proporciona proposiciones/citas tipadas sobre las que aplicar el guard. V3 sí las proporciona cuando el modelo logra construir una traza válida, pero sus tres smokes recientes fallaron antes de seleccionar. Por tanto, no hay una ganancia de producto que medir aplicando solo el guard a `baseline`; hacerlo requeriría otro extractor o etapa nueva, que no se añade en este ciclo. No se abrió otra campaña remota ni se gastó API.

La siguiente iteración queda definida como una comparación prospectiva de una política completa de atribución, con Decisions+baseline en ambos brazos, fixtures disjuntas revisadas a ciegas y candidatos/búsqueda idénticos. Primero debe demostrarse localmente que la política conserva datos directos y contenido reportado solicitado, y bloquea la atribución ajena. Si la extracción estructurada sigue siendo requisito, la puerta local debe incluir salidas reales capturadas y fallar antes de ampliar; no se ajustará V3 campo por campo ni se contarán estos tests de unidad como mejora de calidad.
