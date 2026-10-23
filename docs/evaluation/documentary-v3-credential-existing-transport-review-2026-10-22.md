# Credencial: transporte existente y alcance del mapa de origen

Estado: revisión técnica conocida, no ciega, sin dictamen semántico nuevo ni medición de calidad. Se preserva la [nota conceptual anterior](documentary-v3-credential-context-transport-analysis-2026-10-22.md). Se leyeron esa nota, `nominal_possessive_matching.rs`, `documentary_v3_io/{readings.rs,delivery.rs,packet.rs}` y el protocolo `source-context-transport-evidence-v1/protocol.json`. No se consultaron otros reviews, gold o resultados de las nuevas invocaciones. No se ejecutaron Cargo, fmt, builds, APIs, CLI o servicios ni se modificó Rust.

## Conclusión técnica

La propuesta puede demostrar más que la correcta construcción de un mapa por el runner. Combinada con una traza de igualdad exacta del documento y sus operandos, y con el código observado que copia esos datos sin resolver el posesivo, permite establecer conservación de la ocurrencia fuente **en la ruta compuesta y acotada de esa invocación**. No requiere cambiar el motor para obtener esa evidencia. El protocolo solo especifica las comprobaciones; esta revisión no ha visto sus salidas y no certifica que hayan pasado.

La afirmación defendible sería: el contexto parcial publicado por esta invocación sigue refiriendo a esta ocurrencia de la fuente, con U, H, su claim y el cotexto intactos, sin sustituir el posesivo por el owner solicitado. El mapa identifica la ocurrencia de entrada observada, no acredita por sí mismo la emisión histórica original. Tampoco convierte el reporte JSON aislado en un transporte autosuficiente entre consumidores.

## Qué aporta el código existente

En `tools/evaluation/src/documentary_v3/nominal_possessive_matching.rs`, `possessive_partial` toma `sd` de `self.source_report["document"]`, no del documento de pregunta. Tras comprobar compatibilidad, construye un testigo `nominal_possessive_context` y:

- vacía `bindings`, `answer_terms` y `answer_propositions` (líneas 291–293);
- copia el claim completo a `source_constraint`, el request a `request_constraint` y cada documento completo al campo de su lado (294–297);
- copia referencias, ámbitos, qualifiers y dependencias de ambos lados por separado (298–309);
- publica el testigo como contexto parcial con referente no probado (310–311).

Por tanto, hay una operación concreta que puede conservar la restricción, no solo un comentario de intención. `possessive_head_matches` permite que una restricción `owned_by` solicitada quede como deuda cuando el fuente tiene el posesivo; esa excepción no escribe sobre U ni añade el owner a la fuente. Las comprobaciones de compatibilidad deciden relevancia parcial; no son una prueba de identidad del referente. El owner de la pregunta permanece del lado de la pregunta.

`readings.rs::run_with_context` exige que cada lado de la entrada tenga exactamente `text` y `raw`; en la ruta sin envelopes valida ambos y pasa sus informes a `classify_pair_context`. El reporte retiene separadamente source, question y result antes de `write_report` (líneas 745–794). Para envelopes, la matriz retiene resultados por par de lecturas; las firmas de respuesta completa no convierten el testigo parcial en respuesta completa. El protocolo debe comprobar la ruta realmente ejecutada, no inferirla solo por nombre del comando.

`packet.rs::checked_packet` y `delivery.rs::audit_format` añaden integridad útil en su propia frontera: hash congelado, vocabulario, texto, identidad/orden de filas y conservación del raw. No son automáticamente pasos del comando de pares: no basta que existan para atribuir su garantía a la invocación técnica. Además, sus IDs son locales al paquete y los hashes identifican contenido. Dos recepciones del mismo paquete pueden seguir teniendo contextos de origen distintos. No se encontró en estos archivos un token de emisión que viaje con cada testigo.

La copia íntegra preserva la estructura que pudiera depender del contexto; no hace falta que un algoritmo resuelva esa dependencia. Esta inspección cubre la construcción y serialización observadas. No demuestra por sí sola todas las garantías de módulos no inspeccionados ni el comportamiento de futuros consumidores.

## Cuándo el expediente es suficiente

Para este alcance local son suficientes las siguientes condiciones conjuntas, que el expediente puede comprobar sin adjudicar el antecedente:

1. El runner fija un token opaco fresco por ocurrencia de entrada antes de consumir la salida, con mapa inmutable al input fuente exacto. El mismo origen puede usarse al variar la pregunta; si se usan invocaciones independientes, registrar explícitamente la relación entre sus entradas fuente idénticas sin fusionar por ello las ocurrencias. Una ocurrencia fuente es distinta de una ejecución que la procesa: el protocolo debe declarar qué identifica cada token.
2. El reporte queda ligado a su invocación concreta, y el testigo a su ruta dentro de ese reporte. La igualdad se comprueba sobre el raw/documento completo, no solo Claim/H/U ni el hash del texto. Se comprueban también los campos duplicados del testigo contra ese mismo documento, y Claim/H/U contra sus posiciones en él. Esto detecta una copia íntegra correcta acompañada de un `source_constraint` o array auxiliar alterado.
3. Las dos preguntas técnicas modifican exclusivamente el owner solicitado y preservan la fuente exacta. Ambas salidas deben conservar la construcción fuente, mantener la deuda y no generar binding o respuesta. Dos entradas de fuente independientes con bytes idénticos deben mantener tokens distintos; la prueba no presupone que sus referentes sean diferentes.
4. El consumidor de este expediente interpreta el testigo como referencia al contenido de esa ocurrencia fuente, y mantiene el mapa o un vínculo verificable a él. No aplica resolución bajo el contexto de la pregunta ni transforma ese contexto parcial en una conclusión sobre la credencial de la persona solicitada.

El código aporta evidencia para la copia y la separación de lados; la traza comprobaría ese comportamiento en las ejecuciones concretas. El mapa aporta la identidad de ocurrencia que la copia JSON no codifica. La cuarta condición fija el significado de la composición: si el siguiente consumidor se limita a recuperar la construcción de la misma entrada, conserva la dependencia κ(U,c0) sin conocer κ. Aquí c0 es el contexto asociado al origen, no un registro exhaustivo de hechos reconstruido desde el texto.

Esto no es una equivalencia global postulada entre fuente y DTO: hay pasos inspeccionables, copias comparables y alteraciones que deben fallar. En cambio, una declaración final `semantics:source_reference` que ningún consumidor verifica o respeta solo documentaría intención. Los controles de tampering prueban la frontera que realmente ejecuta su rechazo; un rechazo en el runner no demuestra un rechazo adicional dentro del motor.

## Límite real y ampliación mínima del contrato consumidor

No hace falta añadir un campo a cada U ni cambiar `nominal_possessive` para probar la ruta compuesta local. Si el resultado se conserva y consume siempre junto al expediente inmutable, ese wrapper puede ser el contrato de transporte. Hace falta describir expresamente esta obligación; el JSON aislado actual no la impone.

El seam es la salida de `readings.rs::run_with_context`, justo después de obtener el reporte y antes de `write_report`, o la entrada del siguiente consumidor que lo recibe. En evaluación puede permanecer fuera del motor. El wrapper mínimo debe ligar:

```text
transport_version
origin_occurrence_token -> frozen exact input source
invocation_identity -> exact input and report
witness_locator -> report path and source claim/U/H within that origin
interpretation_mode = source_context_reference
```

Es un esquema de obligación, no un DTO implementado ni una nueva autoridad semántica. El texto fuente y el vocabulario/guía usados deben ser recuperables desde el expediente. Se puede evitar duplicarlos usando referencias a archivos inmutables verificadas; el token no debe derivarse únicamente del contenido. Cada testigo hereda su origen por la combinación invocación+posición en el reporte, sin necesitar identidad global nueva dentro del raw.

El consumidor debe rechazar o mantener pendiente, con causa distinguible: origen ausente/desconocido; token reutilizado para una ocurrencia distinta; input o reporte que no coincide con el registro congelado; testigo/claim/H/U o sus copias auxiliares inconsistentes; pérdida del mapa al exportar; modo de interpretación no soportado. Estos son errores prospectivos del contrato wrapper, no códigos ya presentes en el motor. Ninguno se trata como referente falso ni se repara eligiendo Ina/Sira.

El coste es mantener el registro, verificar integridad/comparación del reporte y conservar su disponibilidad durante consumo o exportación. Las comparaciones requieren trabajo proporcional a los datos conservados; el código ya duplica documentos y arrays y reserva presupuesto antes de copiarlos. No se midieron bytes adicionales, latencia o coste operativo. Versionar un wrapper externo puede conservar los formatos actuales; exigirlo dentro de la CLI implicaría cambiar su contrato de salida y revisar a sus lectores. No se propone implementar ninguna de esas ampliaciones en esta tarea.

Una precaución semántica concreta: `referent_resolution:"source_indeterminate"` no debe interpretarse como hecho de que c0 carece de antecedente determinado. En esta ruta debe leerse como «el transporte no publica una resolución del referente fuente». Si un consumidor usa el literal para borrar una relación que c0 sí impone, la dependencia deja de conservarse. Precisar ese significado en el contrato consumidor es suficiente para este análisis; no se adjudica si ocurre en esta fuente y no se cambia aquí el enum.

## Contraejemplo y siguiente comprobación

Un lector recibe dos reportes idénticos, descarta el wrapper y fusiona U por `(text_sha256, root, 60, 62)`. Después toma `owned_by:Sira` de la pregunta como resolución del posesivo fuente. Aunque los dos tests del runner hubieran mostrado copias perfectas y tampering rechazado, este lector rompería el transporte. El contraejemplo afecta la frontera posterior; no demuestra que `possessive_partial` efectúe esa sustitución. Mantener el wrapper y prohibir esa resolución lo bloquea sin resolver c0.

Acción comprobable: completar el protocolo con comparación del documento entero y de sus duplicados, vínculo reporte↔invocación↔ocurrencia y una comprobación del consumidor que conserva o rechaza el wrapper. Si pasa, acreditar conservación contextual de esta ruta local compuesta. No extrapolarla a JSON desprendidos del expediente, procedencia histórica de la fuente, producción o aceptación semántica integral.
