# Matriz estática de slots y piezas de evidencia V1

Fecha: 2026-10-23. Extensión sintética de [evidence-roles-v1](evidence-roles-v1-contract.md), no reemplazo. La crítica independiente aprobó esta matriz para una preflight estática de estructura y consistencia de anotaciones. **No mide extracción automática, verdad semántica, calidad de recuperación ni runtime.**

## Contrato de salida

La consulta contiene una identidad o ancla opcional, cero o más premisas corregibles y uno o más slots solicitados. La entrada se limita a un conjunto finito y declarado de memorias. Cada memoria declara su cobertura para el alcance consultado: `complete`, `incomplete` o `unknown`. La cobertura de fuente es distinta de la completitud de la respuesta.

Cada proposición conserva `memory_id`, `proposition_id`, sujeto/ancla o referencia no resuelta, predicado/slot, valor opcional, alcance (evento, tiempo, condición y modalidad), cita fuente y relaciones independientes con premisas y slots. La identidad se juzga por candidato. Coincidir en un nombre no demuestra que sean la misma persona.

Estados de evidencia por slot:

- `answers`: un hecho afirmado y contextualizado responde el slot.
- `partial`: aporta una parte útil del slot sin afirmar que completa la respuesta.
- `explicit_unmeasured`: la fuente afirma literalmente que ese slot no se midió dentro del alcance citado.
- `not_observed_in_this_memory`: esa memoria completa no aporta evidencia observada para el slot. Solo es válido cuando `source_coverage=complete` para esa memoria y ese alcance.
- `unknown`: la fuente o su cobertura es incompleta/desconocida; no equivale a silencio.

La completitud agregada (`complete`, `partial`, `unanswered`, `unknown`, `ambiguous`) se deriva después por slot sobre los candidatos declarados. Un recuerdo que solo identifica a alguien es contexto: no cuenta como evidencia útil de respuesta salvo que resuelva el vínculo de identidad de otra proposición. Refutar una premisa no contesta automáticamente un slot; las dos relaciones permanecen separadas. Valores distintos no se declaran contradictorios sin compatibilidad de identidad, slot, evento, tiempo y condiciones.

## Casos y salidas esperadas congeladas

Los identificadores son locales y sintéticos. `coverage=complete` significa que la fuente fue revisada exhaustivamente para el slot/alcance señalado; el checker solo valida la anotación y no puede probar que esa revisión sea verdadera.

| Caso | Consulta; candidato(s) de evidencia | Juicio esperado |
|---|---|---|
| 1 | Q: «¿En qué ciudad vive Ana Ruiz, empleada E-17?» M01: «Ana Ruiz (E-17) es arquitecta.» | Identidad `match`; ciudad `not_observed_in_this_memory` solo con cobertura completa de M01 para ciudad. M01 aporta contexto, no evidencia de ciudad. Con cobertura parcial/desconocida, ciudad `unknown`. |
| 2 | Q: «¿En qué ciudad vive Ana Ruiz (E-17)? Creo que es Lima.» M02: «Ana Ruiz (E-17) vive en Quito.» | Identidad y alcance `match`; la premisa Lima queda `refutes`; ciudad `answers(Quito)`. Son juicios separados. |
| 3 | Q: «¿Qué versión del despliegue D-42 se publicó? Creo que 4.2.» M03: «El despliegue D-42 publicado el martes fue 4.1.» | Evento/tiempo compatibles; premisa 4.2 `refutes`; versión `answers(4.1)`. |
| 4 | Q: «¿En qué ciudad y año abrió el taller T-8?» M04: «El taller T-8 abrió en Rosario.» | Ciudad `answers(Rosario)`; año `not_observed_in_this_memory` solo con cobertura completa de M04. Respuesta agregada `partial`; si la cobertura no es completa, año `unknown`. |
| 5 | Q: «¿Qué material y acabado tiene la mesa M-6?» M05: «La mesa M-6 es de roble y tiene acabado mate.» | P1 material `answers(roble)`; P2 acabado `answers(mate)`. Ambas piezas son seleccionables y útiles; se conserva la respuesta de ambos slots. |
| 6 | Q: «¿Qué correo usa Bruno, empleado E-22?» M06: «Bruno (E-22) vive en Córdoba.» | Identidad `match`, pero ciudad no responde correo. Correo `not_observed_in_this_memory` solo con cobertura completa para ese slot; en otro caso `unknown`. No seleccionar como evidencia del correo. |
| 7 | Q: «Lina (E-31) usa Vim, ¿verdad? ¿Qué editor usa?» M07: «Lina (E-31) no usa Vim; usa Helix.» | P1 refuta la premisa Vim; P2 contesta editor `Helix`. La refutación no sustituye la respuesta. |
| 8 | Q: «¿Cuál fue el destino final del viaje V-5 en junio?» M08a: «En mayo se planeó Cusco para V-5.» M08b: «El viaje V-5 terminó en Arequipa en junio.» | M08a es plan de mayo, no responde resultado final de junio. M08b `answers(Arequipa)`. Valores distintos no son contradicción porque difieren evento/tiempo. |
| 9 | Q: «¿Qué proveedor aprobó el comité C-3 el lunes?» M09: «El comité C-3 podría aprobar Nube Sur la próxima semana.» | Posibilidad futura no afirma aprobación efectiva el lunes ni refuta esa premisa. Sin cobertura completa, resultado `unknown`; con cobertura completa del candidato acotado, `not_observed_in_this_memory`. |
| 10 | Q: «¿Cuál fue el promedio de espera del lote L-9?» M10a: «L-9 tuvo 40 casos.» M10b: «El promedio de espera de L-9 no se midió.» | M10a no responde el promedio; ausencia solo se deriva con cobertura completa. M10b `explicit_unmeasured` con cita literal y alcance L-9. Responder “no se midió” para ese lote; no inferir cero ni una afirmación global. |
| 11 | Q: «¿En qué ciudad vive Alex?» M11a: «Alex vive en Salta.» M11b: «Alex vive en Neuquén.» | Ambos vínculos de identidad `unresolved`; conservar los valores como candidatos sin declarar contradicción ni elegir ciudad. Agregado `ambiguous`. |
| 12 | Q: «¿Cuál es la reserva del evento EV-9 de Elena, fotógrafa E-44?» M12a: «Elena (E-44) es fotógrafa.» M12b: «Elena tiene la reserva QZ-81.» | M12a resuelve contexto, no el número. M12b tiene reserva, pero vínculo de identidad/evento `unresolved`; conservar como candidato sin atribuirlo a EV-9. Agregado `ambiguous`/abstención. |

El conjunto finito de candidatos es `{M01, M02, M03, M04, M05, M06, M07, M08a, M08b, M09, M10a, M10b, M11a, M11b, M12a, M12b}`. Las anclas `E-*`, `T-*`, `D-*`, `M-*`, `V-*`, `C-*`, `L-*` y `EV-*` solo existen en este fixture sintético.

## Checks de preflight

El CLI `iromem-eval validate-slot-contract <matriz.json> <informe.json>` ejecuta el checker estático sobre el fixture congelado en `tests/fixtures/slot-alignment-v1/development.json`. La primera ejecución pasó 12/12 casos, cinco pruebas unitarias focalizadas y formato. Eso prueba solo las propiedades enumeradas abajo tal como se codificaron; el recibo declara explícitamente que verdad semántica, extracción, runtime y calidad de recuperación no fueron medidas.

Un checker local debe congelar y comprobar las doce salidas esperadas y verificar:

1. IDs únicos de candidatos/proposiciones y resolución de cada referencia dentro del conjunto declarado.
2. Cada cita coincide con la memoria asignada; slots y premisas existen en la consulta; cada hecho apunta a su sujeto y slot/premisa declarado.
3. `answers`, `supports` y `refutes` exigen sujeto, evento, tiempo y condiciones compatibles con el objetivo. Diferencias no alineadas no son contradicción.
4. `explicit_unmeasured` requiere cita fuente explícita; no se infiere de un valor ausente.
5. `not_observed_in_this_memory` exige cobertura `complete` para el ID de memoria y alcance exactos. `incomplete`/`unknown` producen `unknown`; no se convierte una falta de anotación en silencio.
6. `partial` se deriva de los estados por slot; evidencia solo de identidad no cuenta como slot respondido.

La aprobación del checker probaría únicamente forma, referencias, grounding literal y consistencia entre anotaciones esperadas. La cobertura completa seguirá siendo una anotación revisada, no una verdad certificada por software. Después de esta preflight todavía hará falta un mecanismo automático sin etiquetas/gold en runtime y una medición de recuperación que supere las puertas originales.
