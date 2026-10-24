# Guarda local de sujeto en oraciones posteriores V3

Fecha: 2026-10-23. Estado: corrección local experimental; baseline y política predeterminada intactos.

## Hallazgo

La ablación de [recorte por oración](slot-evidence-clause-slicing-retrospective-2026-10-23.md) mostró que el resumen de Marta/Teo tenía el dato útil en la primera oración y un distractor en la segunda. La guarda V3 existente inspeccionaba cláusulas relativas y complementos de reporte antes del predicado, pero no distinguía de forma explícita un sujeto con ID ajeno en una oración independiente posterior cuando la cita de una proposición cubría ambas oraciones.

## Cambio y prueba

La validación y democión locales comprueban también el sujeto estable más cercano antes del predicado dentro de la oración o cláusula contrastiva que lo contiene. Los límites cubiertos son punto/interrogación/exclamación con inicio de oración reconocible, punto y coma, y los conectores explícitos `, while` / `, mientras`. Solo demueven si el segmento del predicado no repite el ID consultado y el predicado sigue a un ID estable ajeno. La regla no reescribe citas ni produce una paráfrasis.

La regresión nueva usa el resumen congelado de `smoke-14` y una traza sintética con dos proposiciones: una cita exacta de Marta/Valencia y una cita amplia que intenta atribuir a Marta `lives in Girona`, cuyo sujeto literal es Teo. El mecanismo demueve solo la segunda proposición y la misma memoria sigue seleccionada por la primera. Controles verifican además que Carlos antes de Marta no cause democión, y que una mención ajena en la misma oración no cambie el control existente de coordinación/reporte.

Verificación: `cargo test --offline -p gennie-memory-retrieval` (**94 aprobadas, 1 ignorada, 0 fallidas**); `cargo test --offline -p gennie-memory` (**228 aprobadas, 24 ignoradas, 0 fallidas**); `cargo fmt --all --check` y `git diff --check` pasan. El test existente del banco ciego de 14 casos continúa con 7/7 respuestas retenidas y 0/7 casos ajenos seleccionados mediante trazas escritas manualmente.

## Revisión prospectiva ciega de límites de oración y cláusula

Se congeló `tests/fixtures/slot-evidence-v3-sentence-scope-prospective-v1.json` (10 casos sintéticos nuevos; SHA-256 registrado en la decisión 0037). Dos lectores recibieron únicamente las preguntas y resúmenes, sin etiquetas, trazas ni código. Coincidieron en los 10 juicios: siete consultas contestables con la entidad/valor explícitos y tres no contestables para la persona preguntada (el valor corresponde a otra persona, solo hay reporte sobre otra persona o la ubicación pedida es actual y solo hay una pasada). No hubo desacuerdos que requirieran adjudicación.

Al contrastar esos juicios con el guard local, la prueba focalizada detectó dos huecos adicionales: una segunda cláusula separada por punto y coma y otra introducida por `, while`. Se incorporaron límites explícitos para esos dos patrones; los controles verifican que el predicado de Marta siga asociado a Marta y que el de Teo se demueva. La prueba también verifica sujetos ajenos en oraciones separadas, orden inverso y un positivo en español. Esto es una prueba de comportamiento del guard con excerpts construidos a partir de los textos congelados, no ejecución del extractor ni métrica de recuperación de extremo a extremo.

Coste API: USD 0; latencia remota: no medida. Verificación final tras la extensión: retrieval **95 aprobadas, 1 ignorada, 0 fallidas**; `gennie-memory` **228 aprobadas, 24 ignoradas, 0 fallidas**; `cargo fmt --all --check` y `git diff --check` pasan. Persiste el warning preexistente de importación no usada en el evaluador. La ruta V3 sigue experimental y no se adopta; baseline, etiquetas originales y negativos históricos permanecen intactos. Alcance conocido: la nueva regla solo cubre límites explícitos enumerados; no resuelve pronombres, sujeto implícito ni todas las coordinaciones. La fixture prospectiva no se usó para ajustar etiquetas ni para volver a llamar al proveedor.

## Límite y decisión

Esto corrige un control local concreto y demuestra conservación de una proposición útil junto a una ajena, no capacidad de extracción remota ni una mejora de cohorte. La regla cubre sujetos con ID explícitos en oraciones separadas; no cubre todas las cláusulas coordinadas o sujetos pronominales. Las 14 salidas remotas V3 siguen inválidas y no se repitieron. No se mide latencia de modelo ni se incurre en coste API; el warning de importación no utilizada en pruebas de `gennie-memory` persiste. Antes de comparar modelos hace falta una cohorte nueva revisada a ciegas que incluya límites de oración y cláusulas dentro de una misma oración.
