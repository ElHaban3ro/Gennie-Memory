# Contraauditoría del cierre y la cobertura de adjunción

Fecha: 2026-10-22. Revisión independiente, documental y estática. No constituye aceptación, no sustituye dictámenes previos y no modifica gold ni puertas de evaluación.

## Dictamen acotado

Para «Senda tiene una bandeja de papel grande», encuentro defendible que las dos hojas cubran la **adjunción del descriptor léxico** bajo las definiciones explícitas del paquete. No encuentro un tercer propietario apoyado por este texto ni una afirmación explícita de material, uso o doble tamaño que exija otra hoja. La justificación no es que existan dos hojas válidas: es la conservación del predicado principal, los dos núcleos nominales pertinentes, el complemento y la atribución alternativa del mismo modificador, sin convertirlos en magnitudes o relaciones más específicas.

Este resultado es un juicio de fidelidad limitado al texto y al contrato léxico suministrados. No demuestra que el código establezca exhaustividad semántica, que cualquier genitivo pueda reducirse a asociación ni que dos hojas basten para otra fuente. El `reading_set` V1 no tiene un campo que delimite formalmente «solo cobertura de adjunción». No debe presentarse este juicio como una nueva garantía automática de cierre por proyección.

## Material examinado y aislamiento

Se examinó primero únicamente el paquete fuente `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/lexical-attachment-corrections-v1/review-packet.json`: sus dos fuentes, raws, vocabulario y `semantic_guide`. Se contrastó con las implementaciones actuales de `documentary_v3_io/readings.rs`, `documentary_v3/validation.rs`, `documentary_v3/validation/graph.rs` y `documentary_v3/attribute_profiles.rs`. Se leyeron las reglas AGENTS/backend/testing. No se consultaron preguntas del benchmark, gold, V6 ni dictámenes de otros críticos para decidir esta conclusión. La memoria general solo se utilizó para recordar la separación entre validación mecánica y fidelidad; no aportó evidencia lingüística ni resultados de este caso.

No se ejecutaron Cargo, servicios, modelos ni API. Los ejemplos negativos siguientes son razonamientos documentales, no controles ejecutados ni evidencia de comportamiento observado.

Huellas SHA-256 del material auditado:

| Archivo | SHA-256 |
| --- | --- |
| `review-packet.json` anterior | `7ce52998bf5c9278e00bf176a814f687e947d4977bc54ca1e744bfeb780a013b` |
| `tools/evaluation/src/documentary_v3_io/readings.rs` | `c7a5f714e9e570dfdfef2315e42b1703207a314693a3ac4b931e85eedcb440b9` |
| `tools/evaluation/src/documentary_v3/validation.rs` | `e0c4b465111a2e4e923434023ba45a5cf25fb7d576b7bd294538d6ad13304bf9` |
| `tools/evaluation/src/documentary_v3/validation/graph.rs` | `6809588c8d3cf91ce38adf17aa3661f6ebf1bac5ab88807b2b01d37e313bf577` |

## Qué conserva el grafo de Senda

Las dos hojas conservan `has_item(senda, tray)`, la referencia nombrada Senda, la clase literal `bandeja`, la referencia local con clase literal `papel` y el enlace dirigido `tray associated_with paper`. La segunda referencia no afirma una hoja concreta, una cantidad, almacenamiento ni suministro. La guía lo prohíbe expresamente; `denotation: entity` debe interpretarse dentro de ese límite, sin añadir una existencia física independiente.

La única diferencia sustantiva es el sujeto de `property(subject, nominal_qualitative_descriptor, "grande")` y su dependencia: `tray` o `paper`. Ambas mantienen el modificador, su cita, su propietario correspondiente y el resto de la fuente. La ambigüedad `adjective_attachment` cita el sintagma completo y enumera ambas hojas; no se sustituyó el inventario por el único hecho invariante de posesión.

El razonamiento de cobertura es el siguiente:

1. El modificador está dentro del sintagma «una bandeja de papel grande». Hay un núcleo principal, `bandeja`, y un núcleo del complemento, `papel`, aptos para recibir la adjunción conservada. No aparece otro núcleo candidato dentro de ese sintagma. Hacerlo predicación de Senda requeriría una construcción o evidencia adicional que no proporciona esta oración.
2. La guía define el atributo como atribución cualitativa léxica, sin interpretación física especializada. Cambiar su propietario cambia un compromiso del grafo y exige las dos hojas. Cambiar un tamaño físico imaginado manteniendo el mismo propietario no cambia este compromiso léxico.
3. «De papel» no expresa aquí «fabricada con», «destinada a guardar», «contiene» ni otra relación verbal explícita. Sus posibles lecturas de material y destino son interpretaciones plausibles, pero el contrato permite conservar la relación genitiva sin elegir una. No encuentro en esta fuente una especialización explícita que el enlace neutro esté borrando.
4. Que en una situación real sean grandes tanto la bandeja como el papel no aporta por sí solo otra lectura textual: ninguna hoja niega que el otro objeto tenga una propiedad física. Tampoco se declara exclusividad física. Añadir una hoja que atribuya léxicamente el único adjetivo a ambos núcleos necesitaría justificar una doble predicación de ese token; la mera posibilidad de que ambos sean grandes no la justifica.

La ambigüedad lingüística restante es real: el texto no decide el propietario del adjetivo ni el sentido físico del complemento o del descriptor. El grafo conserva la primera por alternativas y los demás mediante expresiones locales sin especialización. Eso permite una defensa de cobertura en este lenguaje tipado, no una declaración de que el español haya fijado dos mundos completos. Si una interpretación plausible exigiera un compromiso distinto que la asociación o la clase literal no conservaran, habría que representarlo o mantener apertura; no podría descartarse por conveniencia.

## Qué exige y qué comprueba V1

La regla semántica del paquete dice que las interpretaciones apoyadas deben ser exhaustivas y que cada hoja conserve la fuente completa y el propietario exacto. No autoriza hojas que solo retengan lo útil para una pregunta. Por tanto, un compromiso explícito perdido sigue siendo un fallo aunque no altere una respuesta posterior.

El código confirma un mecanismo más limitado:

- `readings.rs:47-185` exige forma cerrada, hashes, de una a ocho hojas, identidad de documentos, citas literales y metadatos de ambigüedad. Cada hoja pasa por el validador V3. Las dimensiones admitidas para ambigüedades son `modal_basis`, `relation`, `reference` y `temporal_aspect`; los elementos de `unresolved_dimensions` admiten una dimensión textual libre.
- El estado `complete` depende de cierre declarado, ausencia de dimensiones pendientes y hojas completas. No existe análisis del español que descubra propietarios omitidos o pruebe que la lista de lecturas agota las interpretaciones. Tampoco una comprobación general que fuerce combinaciones de todas las dimensiones o descubra correlaciones entre ellas. El informe mantiene `semantic_fidelity_proven: false`.
- `validation/graph.rs:565-583` revisa los elementos de cobertura aportados y sus propietarios; no deriva del texto todos los compromisos que deberían tener una entrada. Una cita larga marcada `represented` no prueba por sí sola que su semántica esté conservada.
- `attribute_profiles.rs` comprueba pertenencia y tipos de valores. No implementa una equivalencia entre el atributo léxico y tamaño físico, ni entre una clase citada y una ontología del mundo. La guía semántica humana no es una prueba ejecutable de esas equivalencias.
- `readings.rs:597-710` combina las hojas fuente/pregunta y exige evidencia compatible e igual firma para respuesta completa en toda la matriz. Esta cuantificación recorre las hojas suministradas; no descubre hojas ausentes ni repara pérdida de fuente.

No encontré una obligación mecánica de enumerar todas las dimensiones físicas posibles. Tampoco encontré un operador de «envelope de proyección» que permita olvidar dimensiones expresadas y conservar `closed`. El contrato semántico exige fidelidad completa de cada lectura dentro de su representación; su satisfacción es aquí un juicio independiente, no consecuencia del estado `complete`.

## Contraejemplos que impiden generalizar el cierre

| Caso | Omisión o transformación indebida | Consecuencia para el argumento |
| --- | --- | --- |
| «La oficina está en la esquina» | Conservar solo `office associated_with corner` | Se pierde una relación espacial explícita. La asociación neutra no conserva toda la predicación; las citas no reparan ese grafo. |
| «La bandeja de papel grande y la tapa roja» | Conservar solo posesión o trasladar `roja` a la bandeja | Se pierde el modificador o su propietario inequívoco. Ni el hecho invariante ni el cierre declarado lo excusan. |
| «La bandeja es grande» frente a una pregunta por ancho en centímetros | Igualar descriptor léxico y tamaño cuantificado | La atribución literal no proporciona dimensión, unidad ni valor. Cierre de adjunción no autoriza esa respuesta. |
| Clase literal `papel` o `matrículas` | Convertirla automáticamente en hoja almacenada, material de fabricación, licencia o inscripción | La clase local citada no prueba esa ontología ni esas relaciones. El mismo texto debe conservar sus restricciones explícitas aunque no se especialice su sentido. |
| Dos alternativas correlacionadas, por ejemplo «o la tapa roja está dentro y la azul fuera, o la roja fuera y la azul dentro» | Enumerar colores y posiciones independientemente o retener solo dos hojas que omitan un conjunto expresado | Puede introducir combinaciones no apoyadas o perder alternativas apoyadas. Hace falta preservar las correlaciones del conjunto completo; contar hojas no demuestra cobertura. |

La otra entrada del paquete dice «Las matrículas de Faro se entregan en la oficina de la esquina». Su grafo conserva literalmente `oficina de la esquina` como clase, pero el enlace a `corner` es solo `associated_with`. Hay una cuestión específica que este cierre de Senda no resuelve: si la expresión locativa convencional exige una relación espacial operativa, la clase completa citada no permite asumir que esa relación ya está representada en el grafo. No declaro aquí corregida ni aceptada esa entrada. Debe revisarse ese compromiso por separado, sin deducir del genitivo neutral de «de papel» una regla universal para «de la esquina».

## Límite de la conclusión

No identifico una omisión concreta adicional en Senda que obligue a hojas material/uso/ambos tamaños bajo la guía léxica suministrada. Pedirlas únicamente porque describen mundos posibles confundiría el cierre de las alternativas textuales representadas con una enumeración de realidades. A la vez, el cierre no queda certificado por el validador ni por este documento. La conclusión requiere conservar íntegramente la guía, sus prohibiciones, cada compromiso explícito y el inventario de propietarios; no permite debilitar datos, dispensar puertas, recalificar historia ni promover extracción o producto.
