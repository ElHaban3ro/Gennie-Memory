# Preflight de casos sintéticos nuevos para Slot Evidence V2

Fecha: 2026-10-23. Estado: anotación semántica previa; **sin ejecución de clasificador ni comparación de rendimiento**.

## Alcance y revisión

Se prepararon doce pares sintéticos nuevos de pregunta y resumen. El autor inicial entregó clases y selección esperada; un segundo agente recibió solo los textos, sin esas etiquetas, y adjudicó los doce casos. Un tercer crítico resolvió las discrepancias contractuales señaladas abajo. Las etiquetas del autor, la primera revisión ciega y la adjudicación se conservan como resultados de anotación; ninguna es predicción de un clasificador.

La tabla refleja la lectura reconciliada para uso futuro. Los casos aún no tienen IDs estables y trazas V2 completos; por eso no son una fixture ejecutable ni un benchmark congelado. El caso 11 queda fuera de la puntuación de V2.

Los textos completos se preservan en [`cases.json`](../../outputs/iromem/evaluation/slot-evidence-v2-new-cases-preflight-2026-10-23/cases.json), SHA-256 `527d715b8bf22dde65fa9f755c5d70aa848cc644937971ddabb0bb72644b0000`.

| Caso | Clase reconciliada | Selección prevista | Estado y límite |
| --- | --- | --- | --- |
| 01 | `answers` | Sí | Color pedido, respuesta directa. |
| 02 | `answers` | Sí | La pregunta solicita confirmar la fecha; no presenta una premisa separada que requiera `supports`. |
| 03 | `refutes` | Sí | Refuta que el ferry dejara de operar en octubre. |
| 04 | `partial` | Sí | Informa la sala, pero no el piso; la fuente no afirma explícitamente que este último no se registró. |
| 05 | `unrelated` | No | El instrumento corresponde a Teo, no a Alba. La revisión ciega lo llamó primero contexto seleccionable; el crítico lo corrigió según el límite de identidad y el selector V2. |
| 06 | `explicit_unmeasured` | Sí | La nota dice explícitamente que la visita no quedó registrada. No es silencio ni una respuesta que confirme Luma. |
| 07 | `explicit_unmeasured` | Sí | La nota dice explícitamente que no se registró el PIN. |
| 08 | `answers` | Sí | La nota final dice La Serena; Valparaíso aparece como propuesta previa descartada. Proponer Valparaíso como respuesta sería el error deliberado del caso. La validación estructural por cita literal, por sí sola, no demuestra que el clasificador no cometa ese error. |
| 09 | `answers` | Sí | Día de entrega indicado directamente en español. |
| 10 | `refutes` | Sí | Corrige Cali por Palmira. |
| 11 | `partial` | Sí, fuera de V2 | Aclara aprobación para el equipo norte y deja el sur sin registrar. La persona que responde “quién” no aparece en la pregunta como ancla; el binding V2 actual no permite puntuar esta pregunta abierta. |
| 12 | `explicit_unmeasured` | Sí | La nota dice explícitamente que la dosis no está consignada. |

## Discrepancias y decisión

El caso 02 recibió inicialmente `supports` del autor y `answers` del crítico ciego. La adjudicación final es `answers`: la pregunta «Is that right?» pide confirmar el dato; no formula una premisa separada junto a otro slot solicitado.

En los casos 06, 07 y 12, el crítico ciego propuso no seleccionar la falta del valor. La revisión contractual y el código vigente confirman que `explicit_unmeasured` es una relación accionable dirigida al slot y puede seleccionarse cuando la fuente afirma explícitamente que no se registró. Eso no autoriza inferir ausencia del mero silencio.

En el caso 05, la primera revisión interpretó `context_only` como seleccionable. El selector V2 solo selecciona `answers`, `partial`, `explicit_unmeasured`, `supports` y `refutes`; además, saber el instrumento de Teo no ayuda a responder por Alba. Se fija `unrelated`, sin selección.

El caso 11 queda como control de cobertura para una futura evaluación general, pero no se mezclará con el denominador V2 mientras la identidad de quien respondió no pueda anclarse desde la pregunta. El caso 08 permite medir si el clasificador distingue un borrador incorrecto de una nota final; pasar la validación de citas no basta para resolver esa distinción.

## Puerta pendiente

El próximo artefacto debe añadir IDs estables explícitos, spans/targets V2 y manifiesto de hash, conservando la lista conceptual y sus adjudicaciones. Después, y solo si la revisión congelada pasa los tres smokes consecutivos requeridos por el [protocolo](slot-evidence-v2-protocol-2026-10-23.md), se podrá ejecutar la cohorte nueva y comparar G0, G2 y D0 con roster idéntico. Cada salida de clasificador necesita revisión ciega propia. Hasta entonces no hay métricas de calidad, latencia ni coste de la nueva cohorte, ni evidencia para cambiar el baseline.

Se añadió además la regresión local `explicit_unmeasured_slot_evidence_is_selected_without_inferencing_a_value`: una traza V2 con ausencia explícita y vinculada al locker se valida y conserva para contestar el slot. El test no comprueba que el clasificador distinga ausencia explícita de silencio; esa es precisamente una medición pendiente de la cohorte.
