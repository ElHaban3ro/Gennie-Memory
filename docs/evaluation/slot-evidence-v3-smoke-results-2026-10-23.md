# Smokes sintéticos iniciales de Slot Evidence V3

## Alcance y congelación

Se han ejecutado seis peticiones HTTP completas con `google/gemma-3-4b-it` sobre fixtures sintéticos nuevos y revisiones de wire separadas. No se envió ninguna etiqueta, dictamen ni expectativa al proveedor. El primer caso solo sirve como diagnóstico de formato porque «currently» no tenía anclaje temporal explícito. Los demás casos recibieron adjudicación ciega favorable para respuesta directa y atribución de los distractores. El holdout separado de catorce preguntas no se reutilizó. Ninguna de las seis respuestas produjo una traza estructuralmente utilizable.

Antes de cada petición se congelaron corpus, consulta, solicitud, resultado FTS, base SQLite, binario y fuentes relevantes. Se registraron hashes de prompt, schema y payload previos y se verificaron contra las capturas; ambos payloads coinciden exactamente. El primer paquete está en [`assets/slot-evidence-v3/smoke-1`](assets/slot-evidence-v3/smoke-1/) y el segundo en [`assets/slot-evidence-v3/smoke-2`](assets/slot-evidence-v3/smoke-2/). Las adjudicaciones previas se guardan separadas del input enviado al proveedor.

## Resultados

| Revisión | Transporte/recibo | Validación estructural | Resultado | Tokens entrada/salida | Coste | Latencia clasificador / total |
| --- | --- | --- | --- | ---: | ---: | ---: |
| Smoke 1, prompt original | HTTP 200 completo, Parasail | Falló: `slot_evidence_v2_anchor_phrase_not_literal` | Sin selección; no puntuable | 751 / 198 | USD 0,00003994 | 8.940,252 / 8.951,970 ms |
| Smoke 2, prompt de copia literal | HTTP 200 completo, Parasail | Falló: `slot_evidence_v2_anchor_component_order_invalid` | Sin selección; no puntuable | 801 / 251 | USD 0,00004459 | 5.354,200 / 5.371,548 ms |
| Smoke 3, citas literales por componente | HTTP 200 completo, Parasail | Falló: `slot_evidence_v2_anchor_component_gap_invalid` | Sin selección; no puntuable | 783 / 321 | USD 0,00004737 | 2.286,994 / 2.288,125 ms |
| Smoke 4, cita con coma | HTTP 200 completo, DeepInfra | Falló: `slot_evidence_v2_subject_span_not_identifier` | Sin selección; no puntuable | 816 / 193 | USD 0,00006010 | 2.671,718 / 2.673,127 ms |
| Smoke 5, sujeto literal sin puntuación intermedia | HTTP 200 completo, DeepInfra | Falló: `slot_evidence_v2_query_target_quote_not_literal` | Sin selección; no puntuable | 843 / 214 | USD 0,00006355 | 2.913,366 / 2.915,816 ms |
| Smoke 6, cita literal de slot con ejemplo explícito | HTTP 200 completo, DeepInfra | Falló: `slot_evidence_v2_identity_anchor_ambiguous` | Sin selección; no puntuable | 884 / 192 | USD 0,00006340 | 2.338,797 / 2.339,735 ms |

Los cuerpos crudos no se conservan. Cada respuesta tiene hash y recibo completos, pero los errores no permiten recuperar la cita exacta ni afirmar cuál fragmento produjo el segundo fallo. El primer intento detectó además que la captura de uso omitía los alias `prompt_tokens`/`completion_tokens`, aunque `recall` conservaba 751/198. La instrumentación se corrigió y la suite focal verificó esos aliases antes del segundo intento; su captura registra 801/251 correctamente.

## Decisión

Los dos intentos iniciales fallaron 0/1 y se detuvieron sin reintentar; no se enviaron sus intentos 2/3. Aunque transporte y recibo fueron correctos, ninguna produjo una traza validada ni selección, por lo que no hay puntuación semántica. Esta decisión inicial quedó ampliada por los smokes posteriores registrados abajo; el gate de comparación, TRAIN y adopción continúa cerrado.

Una revisión crítica recomendó separar en el protocolo transporte/recibo, estructura, dictamen semántico ciego y resultado del gate; también precisó que las tres repeticiones deben usar exactamente la misma pregunta, roster, prompt, schema, binario, modelo y configuración. El protocolo ya fija esos requisitos, exige el mismo ID positivo en las tres y trata una abstención como fallo del gate positivo. El siguiente trabajo es revisar localmente el formato de identidad para reducir errores de orden/copia antes de congelar otra revisión.

## Revisión local del contrato posterior a los smokes

La inspección del parser confirmó que el segundo código de fallo cubría varias condiciones de orden/contención y, sin el cuerpo raw, no permite identificar cuál ocurrió. Se modificó el wire V3 para pedir `kind_quote` explícita junto a `subject_quote` y `stable_id`. El parser localiza cada cita exactamente una vez, exige que las tres estén dentro de `anchor_quote`, contiguas salvo espacios en blanco y sin solaparse; admite cualquier orden. No normaliza, repara ni busca componentes por proximidad. El contrato de offsets permanece prohibido.

La prueba focal cubre los seis órdenes de componentes y rechaza separadores de coma o conjunción. `cargo fmt --all --check` y `cargo test --offline -p iromem --quiet` pasan (223 aprobadas, 24 ignoradas); luego pasó la suite workspace (1.021 aprobadas, 25 ignoradas).

## Smoke 3, revisión de wire explícito

El caso nuevo preguntó por la ciudad de la curadora Mateo Solis C-77; un hecho distinto sobre Ivy Chen I-12 sirvió como distractor. La revisión ciega previa dictaminó respuesta directa (Porto) y consideró ajena la cláusula de Ivy, sin ambigüedad textual relevante. FTS5 recuperó exactamente el candidato esperado.

La petición Gemma 3 4B/Parasail completó HTTP 200 y recibo: 783/321 tokens, USD 0,00004737, latencia de clasificador 2.286,994 ms y total 2.288,125 ms. Falló cerrada con `slot_evidence_v2_anchor_component_gap_invalid`, sin selección ni puntuación semántica. Las huellas preflight de inputs, fuentes, binario y base permanecieron idénticas. El cuerpo raw no se conserva, así que no se conoce qué separador o límite concreto falló. No hubo reintento.

**Resultado acumulado:** tres revisiones, cada una con 0/1 trazas utilizables; coste API total USD 0,00013190. La revisión actual no supera el gate estructural. No se ejecutan las dos repeticiones de estabilidad ni TRAIN, no se compara contra baseline y no se promueve. Este fallo muestra que separar las citas no bastó; no demuestra un error semántico del modelo. El siguiente paso es revisar localmente el límite wire/parser con el fallo preservado antes de un nuevo preflight.

## Hardening local posterior

La crítica independiente detectó que el decoder compacto mantenía un camino de offsets heredados aunque el JSON Schema V3 ya los excluye. Como el decoder no hacía validación local completa del schema, una salida fuera de esquema podía tomar esa rama y aceptar una identidad con puntuación separadora. Se cerró la brecha en el decoder: la ruta compacta ahora exige los cuatro campos literales de ancla y rechaza cualquier clave de offsets antes de construir la traza. La cobertura prueba las seis permutaciones de tipo/sujeto/ID y verifica que los offsets se rechazan incluso si el proveedor evade el schema. La suite workspace pasó tras el arreglo (1.021 aprobadas, 25 ignoradas). Esta corrección reduce un bypass estructural; no reinterpreta ni rescata el smoke 3, no cambia su coste o salida, y no acredita mejora de calidad.

## Smoke 4, revisión prospectiva de cita con coma

Se congeló un caso sintético nuevo sobre Elina Costa (C-66), cuya ciudad es Riga, con un distractor sobre Tomas Yeo (T-30), que vive en Tallinn y archivó el ledger. La pregunta incluye coma entre nombre e ID. Una revisora ciega, sin etiquetas ni salida, dictaminó que Riga es respuesta directa y que los hechos de Tallinn pertenecen a Tomas. SQLite/FTS5 recuperó el candidato esperado como único top-1. Inputs, búsqueda, fuentes, binario y base tienen hashes pre/post idénticos; los hashes de prompt, esquema y payload quedaron en la captura.

Una llamada HTTP 200 completa a Gemma 3 4B, servida por DeepInfra, produjo 816/193 tokens, coste USD 0,00006010, 2.671,718 ms de clasificador y 2.673,127 ms totales. Falló cerrada con `slot_evidence_v2_subject_span_not_identifier`; no hubo selección ni puntuación semántica. El cuerpo raw no se conserva y ese código se usaba en más de una etapa, así que el texto causante es desconocido. No se reintentó: se ejecutó 1 de las 3 repeticiones previstas y las otras dos quedaron sin ejecutar.

El acumulado V3 asciende a cuatro smokes, todos sin traza utilizable, con coste conocido USD 0,00019200. Ninguna revisión de prompt ha pasado tres smokes idénticos consecutivos; estabilidad, TRAIN, comparación y adopción siguen cerrados. Como seguimiento retrospectivo, el decoder local ahora separa el error de cita literal del sujeto de consulta del error de binding del sujeto de identidad. No cambia los criterios de aceptación. La prueba enfocada pasó (20/20); `cargo test --offline --workspace --quiet` pasó 1.026 pruebas, 25 ignoradas, cero fallidas; `cargo fmt --all --check` y `git diff --check` pasan. Este ajuste solo hará más localizable un próximo fallo; no explica ni corrige retrospectivamente smoke 4.

## Smoke 5, revisión con instrucción de sujeto más precisa

Tras smoke 4, se separaron los códigos de error para cita de sujeto de consulta y binding de identidad. Como hipótesis de formato —no como causa confirmada— también se precisó el prompt: `subject_quote` del ancla debe contener solo el nombre, sin tipo, ID ni puntuación adyacente, y preservar la capitalización. Una revisora ciega confirmó Delft como respuesta directa para Nora Kim (N-41) y atribuyó Breda/archivo a Omar Vale (O-28). FTS5 recuperó el recuerdo esperado como único top-1.

La primera de tres peticiones idénticas terminó HTTP 200 completa, servida por DeepInfra: 843/214 tokens, USD 0,00006355, 2.913,366 ms de clasificador y 2.915,816 ms totales. Falló cerrada con `slot_evidence_v2_query_target_quote_not_literal`; no hubo selección ni puntuación semántica. El error actual no distingue si falló la cita de un slot o de una premisa; el raw no se conserva. Se detuvo la revisión después de 1/3 intentos, sin reintentar. Los hashes pre/post de inputs, fuentes, binario y base coinciden.

**Estado acumulado actualizado:** cinco peticiones con recibo, cero trazas utilizables, coste conocido USD 0,00025555. Smoke 4 y 5 fallaron en etapas diferentes; no se atribuye el primer error a la coma ni se afirma que la instrucción añadida haya resuelto su hipótesis. Ninguna revisión superó tres smokes idénticos; TRAIN, comparación y adopción siguen cerrados. El siguiente paso local es distinguir errores de cita de slots y premisas y añadir pruebas por rama antes de otra petición.

## Smoke 6, slot de ciudad con cita de ejemplo

La nueva revisión añadió al prompt un ejemplo literal para slots: ante una petición de ciudad, citar exactamente `Which city`, sin palabras adyacentes. También se distinguieron localmente errores no literales/ambiguos para slots y premisas. La revisora ciega confirmó Ghent como respuesta directa para Priya Nand (P-52) y atribuyó Bruges/indexación a Luca Miro (L-19); FTS5 devolvió el candidato esperado como único top-1.

La primera de tres peticiones idénticas recibió HTTP 200 completa, servida por DeepInfra: 884/192 tokens, coste USD 0,00006340, 2.338,797 ms de clasificador y 2.339,735 ms totales. Falló con `slot_evidence_v2_identity_anchor_ambiguous`, sin selección ni puntuación semántica. El raw no se conserva; ese error no distingue entre cero anclas compatibles y más de una. Se cerró en 1/3, sin reintento. Los hashes pre/post de inputs, fuentes, binario y base coinciden.

**Estado acumulado actualizado:** seis peticiones con recibo, ninguna traza utilizable, coste conocido USD 0,00031895. La precisión del ejemplo de slot no evitó este fallo, que ocurrió en la etapa de binding de identidad; no se atribuye a la cita del slot. Ninguna revisión pasó tres smokes idénticos consecutivos; TRAIN, comparación y adopción siguen cerrados. El siguiente ajuste local separará la ausencia de ancla compatible de la ambigüedad real, con aceptación intacta.

## Diagnóstico local posterior a smoke 6

Se separó `slot_evidence_v2_identity_anchor_ambiguous` en ausencia de ancla compatible (`slot_evidence_v3_identity_subject_has_no_matching_anchor`) y múltiples anclas (`slot_evidence_v3_identity_subject_matches_multiple_anchors`), con regresiones locales para ambos casos y para IDs repetidos. La regla de selección no cambió. La prueba enfocada pasó; suite workspace: 1.027 aprobadas, 25 ignoradas, cero fallidas; formato y diff limpios. La revisión no reconstruye la respuesta perdida ni modifica los hashes del smoke 6. El siguiente paquete debe ser una nueva revisión congelada antes de ejecutar llamadas; los tres smokes idénticos siguen pendientes desde cero para esa revisión.

## Smoke 7, ejemplo explícito de identidad candidata

Se añadió al prompt un ejemplo que liga `subject_quote` con el nombre, el identificador y el fragmento literal de la frase candidata. La revisión ciega confirmó Galway como respuesta directa para Yara Bell (Y-74) y atribuyó Limerick/revisión a Keon Shaw (K-13). FTS5 devolvió el único candidato esperado. Suite local: 1.027 aprobadas, 25 ignoradas, cero fallidas; compilación del binario correcta.

La primera repetición recibió HTTP 200 completa, servida por Parasail, con 954/2.500 tokens, coste comunicado USD 0,00016316 y 20.267,051 ms totales. Terminó en `slot_evidence_invalid_response`; no hubo selección ni puntuación semántica. Como no se conserva el cuerpo raw, no se puede afirmar si el límite de salida produjo truncamiento ni qué forma concreta fue inválida. Se detuvo en 1/3, sin reintentos. Los hashes pre/post de inputs, búsqueda, fuentes, binario y base coinciden.

**Estado acumulado:** siete llamadas completas, cero trazas utilizables, coste conocido USD 0,00048211. Aún no existe comparación con baseline, mejora demostrada ni gate de estabilidad pasado; TRAIN y adopción siguen cerrados. La siguiente tanda usa diagnósticos locales más precisos y metadatos de respuesta sin cuerpo crudo.

## Diagnóstico local posterior a smoke 7

El código agrupaba errores de JSON incompleto, sintaxis y forma de la traza bajo `slot_evidence_invalid_response`. Ahora los separa en `slot_evidence_trace_json_incomplete`, `slot_evidence_trace_json_syntax_invalid` y `slot_evidence_trace_shape_invalid`; no almacena el contenido, no lo repara ni relaja las comprobaciones. La deserialización estricta desde el texto original se conserva para que claves JSON duplicadas sigan siendo rechazadas. Prueba enfocada: 1/1; suite workspace: 1.028 aprobadas, 25 ignoradas, cero fallidas; `cargo fmt --all --check` y `git diff --check` pasan. Esto hará más informativo un próximo fallo, pero no identifica retroactivamente cuál rama falló en smoke 7 ni demuestra una mejora de selección.


## Smoke 8, corrección explícita de ciudad tras traslado

La revisora ciega confirmó Porto como ubicación actual de Ada Moreno (A-61), Coimbra como ubicación previa, y atribuyó Braga/revisión a Soren Vale (S-12). FTS5 recuperó el único candidato esperado. Los inputs, revisión ciega y búsqueda están separados; el preflight fijó prompt, esquema, fuentes, binario y base antes de la llamada.

La primera repetición fue HTTP 200 completa, servida por DeepInfra, con `finish_reason=stop`, 964/179 tokens, 593 bytes de contenido, USD 0,00006610, 2.694,475 ms de clasificador y 2.696,331 ms totales. Falló cerrada en `slot_evidence_v3_identity_subject_has_no_matching_anchor`, sin selección. Este caso no llegó al límite de salida y produjo JSON con forma decodificable; por tanto el nuevo diagnóstico descarta truncamiento para smoke 8, aunque no reconstruye el `subject_quote` emitido. Se detuvo en 1/3 sin reintento. Los hashes pre/post de inputs, búsqueda, fuentes, binario y base coinciden.

**Acumulado:** ocho respuestas completas, ninguna traza puntuable, USD 0,00054821. La nueva telemetría identifica la etapa de binding de identidad, pero no permite saber qué cita concreta emitió el modelo. El siguiente paso es ajustar esa parte del prompt localmente y conservar la aceptación estricta; no se afirma mejora todavía.

## Diagnóstico local posterior a smoke 8

El código anterior agrupaba dos causas: que el modelo no extrajera ningún ancla de identidad de la pregunta, o que `identity.subject_quote` no incluyera el identificador de ninguna ancla extraída. Se separaron en `slot_evidence_v3_identity_query_anchor_missing` y `slot_evidence_v3_identity_subject_quote_omits_query_identifier`, con regresiones locales. La aceptación sigue siendo fail-closed: ambas ramas rechazan la traza. El smoke 8 conserva su código histórico y no permite saber cuál subrama ocurrió. No se altera su coste, salida ni resultado.


## Smoke 9, ciudad actual con tipo profesional explícito

La revisión ciega confirmó Dijon como ciudad actual de Mona Lee (Q-27) y atribuyó Lyon/catalogación a Arin Sol (P-14). FTS5 encontró el candidato único esperado. El primer intento recibió HTTP 200 completo, servido por DeepInfra, con `finish_reason=stop`, 956/470 tokens, 1.485 bytes de contenido, USD 0,00009480, 5.526,127 ms de clasificador y 5.529,236 ms totales. Falló con `slot_evidence_v2_kind_not_literal`; no se seleccionó recuerdo. El output terminó normalmente y no llegó al límite. No se conserva texto crudo, así que no se sabe cómo el modelo escribió `kind_quote`. Se detuvo en 1/3, sin retry; hashes pre/post intactos.

**Acumulado:** nueve llamadas con recibo, cero trazas utilizables, USD 0,00064301. Los fallos recientes ahora están localizados en distintas partes: binding de identidad (smoke 8) y cita literal del tipo del sujeto (smoke 9). Esto justifica una instrucción de formato más concreta para citas de tipo, sin alterar el parser ni aceptar paráfrasis. No implica mejora hasta validación nueva.


## Smoke 10, tipo profesional distinto tras la revisión del prompt

La adjudicación ciega confirmó Kyoto para Liv Chen (C-44) y atribuyó Osaka/revisión a Felix Noor (F-18). FTS5 recuperó el candidato esperado como único top-1. La llamada HTTP 200 completa fue servida por Parasail; `finish_reason=stop`, 1.008/332 tokens, 1.070 bytes de contenido, USD 0,00005692, 8.722,709 ms de clasificador y 8.725,280 ms totales. Falló con `slot_evidence_v2_anchor_phrase_not_exact` sin selección. No hubo truncamiento. El cuerpo no se conserva y el código no indica qué componente impidió formar el ancla; por ello no se atribuye éxito a la instrucción nueva sobre `kind_quote`. Se detuvo en 1/3 sin reintento y los hashes pre/post coinciden.

**Acumulado:** diez recibos completos, cero trazas utilizables, USD 0,00069993. La revisión del prompt no basta para concluir mejora y la selección continúa cerrada. El siguiente paso es separar diagnósticos para la formación local del span de identidad y continuar solo con nuevos casos.

## Diagnóstico local posterior a smoke 10

La formación del span de identidad ahora distingue el caso en que `subject_quote` contiene el ID estable, además de las ramas previas de anclas de consulta ausentes o IDs omitidos. El parser sigue derivando el ancla localmente y rechaza ambos casos; una regresión cubre una cita de sujeto contaminada con el ID. Prueba focalizada y suite workspace pasan: 1.028 aprobadas, 25 ignoradas, cero fallidas; `cargo fmt --all --check` y `git diff --check` pasan. Smoke 10 conserva su error original; no se concluye que contuviera el ID en la cita. La nueva rama aún requiere un caso futuro para validar su diagnóstico remoto.

## Smoke V3 11 — proveedor fijo, slot no literal

Para controlar la variación de proveedor observada en smoke 10, esta revisión fijó V3 a DeepInfra mediante `provider.order` y desactivó fallback; el baseline y las rutas anteriores conservaron su enrutamiento. El caso nuevo preguntó dónde trabaja actualmente Rina Das (R-82), con un distractor atribuido a Pavel Ivo (P-16). La revisión ciega confirmó Tallinn y separó Tartu como dato de Pavel; FTS5 recuperó el único candidato correcto. Preflight y postflight conservaron hashes idénticos de inputs, fuentes relevantes, binario y base SQLite.

La única llamada recibió HTTP 200 completo de DeepInfra y terminó con `finish_reason=stop`: 1.008/599 tokens, 1.967 bytes de contenido, USD 0,00011030, 4.869,595 ms de clasificador y 4.871,445 ms totales. Falló cerrada con `slot_evidence_v3_slot_quote_not_literal`; no seleccionó el candidato y no es puntuable. No se guardó cuerpo crudo: el código significa que el `quote` propuesto no coincide de forma literal y única con la pregunta, pero no permite conocer el texto concreto ni distinguir esas subcausas. Se detuvo en 1/3, sin reintento.

El proveedor fijado elimina una fuente de variación, pero no resolvió el fallo: smokes 8, 9 y 11 ya fueron servidos por DeepInfra y fallaron en binding de identidad, cita del tipo y cita del slot, respectivamente. El cambio no acredita mejora de calidad. El total acumulado es **11 recibos, 0 trazas puntuables y USD 0,00081023**; no hay estabilidad de tres repeticiones, comparación contra baseline ni adopción. Próxima revisión local: considerar una instrucción de cita para preguntas con forma `Where does ... work now?`, sin reinterpretar smoke 11 ni relajar el parser. Artefactos completos y recibo en [`assets/slot-evidence-v3/smoke-11`](assets/slot-evidence-v3/smoke-11/).

## Revisión local posterior a smoke 11

`slot_evidence_v3_slot_quote_not_literal` solo demuestra que el slot propuesto no se pudo verificar como una cita literal única; como el raw no se conserva, no sabemos si faltó la frase, se parafraseó o se repitió. La pregunta usa `Where does ... work now?`, mientras que el ejemplo del prompt solo mostraba `Which city`. Se añadió un ejemplo literal para esa forma de pregunta y un test de prompt; no se modificaron parser, etiquetas, banco ni aceptación. Esta es una hipótesis prospectiva para smoke 12, no una explicación confirmada del smoke 11 ni una mejora aún.

## Smoke V3 12 — el slot pasa; la premisa no es literal

Se probó un fixture nuevo, con otra identidad, bajo el prompt que contiene el ejemplo literal `Where does`. La crítica ciega confirmó que Malmö responde dónde trabaja Juno Park (J-91), y que Aarhus corresponde a Kai Voss (K-24); FTS5 encontró el candidato esperado. Una llamada HTTP 200 completa de DeepInfra terminó normalmente (`finish_reason=stop`), con 1.037/300 tokens, USD 0,00008185, 3.299,804 ms de clasificador y 3.301,998 ms totales. El decoder aceptó la cita de slot y luego falló en `slot_evidence_v3_premise_quote_not_literal`; no hubo selección. Este recorrido sugiere que la cita de ubicación quedó verificable en este caso, pero no repara la premisa inválida ni produce una traza puntuable. Sin raw no sabemos qué premisa citó el modelo. Se detuvo en 1/3, sin reintento; hashes pre/post estables. [Artefactos](assets/slot-evidence-v3/smoke-12/).

**Acumulado V3:** 12 recibos completos, 0 trazas puntuables, USD 0,00089208. La nueva instrucción avanzó una etapa estructural en un caso nuevo, pero sigue sin selección válida ni evidencia de mejora semántica. El paso local siguiente es precisar cuándo una pregunta de ubicación formula una premisa corregible de empleo actual y cómo citarla literalmente, sin atribuir al smoke 12 el texto raw que no se conserva.

### Revisión local posterior a smoke 12

La pregunta `Where does ... work now?` puede separar el dato de ubicación (`Where does`) de la premisa corregible de empleo actual (`work now`). Se añadió al ejemplo del prompt la cita literal de ambas partes y una prueba del decoder que verifica que llegan a `requested_slot` y `correction_premise` por separado. No se cambió el parser ni se admitió ninguna reparación. El test focal pasó; falta una validación remota nueva para evaluar este ajuste.

## Smoke V3 13 — el binding de identidad falla después de slot y premisa

Se usó un fixture nuevo, con la pregunta de ubicación actual y distractor de otro sujeto. La crítica ciega confirmó Bergen para Sol Vega (S-64) y separó Tromsø/índice para Theo Noll (T-20); FTS5 encontró el candidato correcto. La llamada completa a DeepInfra (`finish_reason=stop`) produjo 1.056/228 tokens, USD 0,00007560, 2.495,895 ms de clasificador y 2.498,358 ms totales. El decoder superó las validaciones de slot y premisa y luego falló con `slot_evidence_v3_identity_subject_quote_contains_stable_id`; no seleccionó nada. Sin raw, no sabemos qué cita de identidad emitió. Se detuvo en 1/3, sin reintento; hashes pre/post intactos. [Artefactos](assets/slot-evidence-v3/smoke-13/).

**Acumulado V3:** 13 recibos completos, 0 trazas puntuables, USD 0,00096768. El ejemplo explícito permitió recorrer una etapa estructural adicional en este caso nuevo, pero el clasificador sigue incumpliendo el contrato de cita y no existe evidencia de mejora semántica. El siguiente fallo concreto está en la cita de identidad del sujeto, que debe mantenerse separada del ID según el contrato; no se admite ni reinterpreta el resultado.

## Revisión local del contrato tras smoke 13

La causa del error de smoke 13 no puede reconstruirse porque el cuerpo generado no se conserva. Sí se verificó una ambigüedad del wire: `query_anchors[].subject_quote` nombraba el sujeto de la pregunta sin ID, mientras `candidates[].identity.subject_quote` y las citas de proposición nombraban texto del recuerdo y podían incluir el ID. Se renombró únicamente el campo del ancla de pregunta a `subject_name_quote`; identidad y proposiciones del recuerdo conservan `subject_quote`. El esquema estricto rechaza el nombre antiguo en el ancla, y las pruebas mantienen intactas la premisa `work now`, el slot `Where does` y las citas de evidencia con ID. Esto elimina una ambigüedad contractual comprobada, pero no demuestra que causara smoke 13 ni que mejore resultados remotos. No hubo llamada nueva ni coste adicional; las 13 observaciones históricas permanecen congeladas.

El código de rechazo que smoke 13 reportó también se ha nombrado con mayor precisión para ejecuciones futuras (`slot_evidence_v3_query_subject_name_quote_contains_stable_id`), porque se dispara al derivar el ancla desde la pregunta. El recibo histórico conserva intacto su código original; el cambio solo afecta nuevos diagnósticos.

## Smoke V3 14 — contrato de ancla aceptado; cita fuente rechazada

El caso sintético nuevo preguntó dónde trabaja ahora la ingeniera Marta Solis (M-58), con un distractor explícitamente atribuido a Teo Rivas (T-26). Dos revisores ciegos coincidieron en Valencia y en no atribuir Girona/mapas a Marta. FTS5 recuperó solo el ID/revisión esperados. La llamada única fue HTTP 200 completa, servida por DeepInfra, `finish_reason=stop`, 1.067/207 tokens, coste USD 0,00007405, latencia del clasificador 2.782,611 ms y total 2.786,037 ms. El recorrido superó el nuevo campo de ancla y la validación de identidad, pero falló después en la cita fuente de una proposición (`slot_evidence_proposition_citation_invalid` en el recibo histórico); no hubo selección. Sin raw no se puede saber si la cita estaba vacía, excedía el límite o no aparecía literalmente en el recuerdo. No se reintentó. Captura y recibo en [`assets/slot-evidence-v3/smoke-14`](assets/slot-evidence-v3/smoke-14/).

La revisión posterior separa el código genérico de cita para ejecuciones futuras: un quote de fuente ausente/no contenido se reporta como `slot_evidence_proposition_source_quote_invalid`, distinto del error de límites/campos de cita. Una regresión local confirma ese mapping. Esto mejora diagnóstico, pero no reinterpreta el contenido perdido ni cambia el rechazo.

**Acumulado V3:** 14 llamadas con recibo, **0 trazas seleccionables de 14**, coste conocido **USD 0,00104173**. La ruta falla el gate de extracción fiable y se detiene; no se harán más llamadas V3. Las nuevas pruebas locales y la suite workspace pasan; el resultado remoto sigue siendo negativo.

### Comparación de control en el mismo caso

Para completar la referencia directa sin repetir V3, se ejecutó una sola llamada `baseline` con la misma pregunta, corpus, candidato y resultado FTS; solo cambió `selection_policy`. Gemma 3 4B, servido por DeepInfra, seleccionó el candidato adjudicado como útil: HTTP 200 completo, 155/6 tokens, USD 0,00000835, 942,941 ms de clasificador y 945,329 ms total. Frente a eso, V3 se abstuvo tras cita inválida: 0/1 frente a baseline 1/1 en este caso. El baseline incluyó el recuerdo entero, con la cláusula de Teo; V3 no llegó a preservar la parte útil. Es un resultado sintético pareado de n=1, insuficiente para una afirmación poblacional, pero no respalda mejora de V3 y confirma que no debe adoptarse. La comparación y ambos recibos están en [`smoke-14-comparison.json`](assets/slot-evidence-v3/smoke-14-comparison.json) y sus directorios de ejecución. Coste de las dos llamadas de smoke 14: USD 0,00008240; acumulado V3 más el control baseline: **USD 0,00105008**.
