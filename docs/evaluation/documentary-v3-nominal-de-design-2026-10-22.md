# Complemento nominal «de»: primitiva de construcción local

Fecha: 2026-10-22. Estado: diseño documental y revisión estática fuente-only; sin implementación o pruebas por este agente. No modifica puertas, raws, dictámenes ni afirmaciones de calidad.

## Dictamen

`nominal_complement_de` puede ser una primitiva semántica de construcción: afirma que un núcleo nominal local tiene un complemento nominal explícito introducido por «de», con dirección y operandos exactos, mientras deja indeterminado el subtipo de relación física. Es más preciso que una asociación incidental porque preserva una estructura que esa asociación no implica. No representa material, propiedad, ubicación, contenido, función o uso; tampoco convierte «papel» en una hoja individual existente.

La propuesta es válida solo si su construcción y sus restricciones se consumen efectivamente. Cambiar `associated_with` por una etiqueta nueva sin validar el enlace de los dos núcleos y sin distinguirlos en matching sería un renombrado vacuo y debe rechazarse. La nueva primitiva no necesita enumerar escenarios del mundo para conservar «bandeja de papel»; tampoco resuelve a qué núcleo se aplica «grande».

## Alcance leído

Reglas: `AGENTS.md`, backend y testing. Fuente propia inspeccionada: únicamente el ítem `s7-041` de `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/next-extraction-v7/source-03-review-packet.json`, texto «Senda tiene una bandeja de papel grande.» El raw conserva `has_item`, un tamaño sobre `tray`, clase literal `bandeja de papel`, y ningún link en `tray`. El estado validado presente en ese paquete se observa como dato histórico, no se adopta como crítica de fidelidad.

Código leído de forma estática: `request_context_profile.rs`, shape/enum Link en `validation.rs`, `pair.rs` (`identity_depth`, patrones de referencia y anclas parciales), `quantified_match.rs` y entrada de `request_context_matching.rs`. No se leyeron preguntas, contrapartes, gold, V6 ni resultados de utility. No se ejecutaron Cargo, Rust, APIs o campañas. Este archivo es la única modificación de este encargo por este agente.

## Forma cerrada y activación

Reutilizar `Reference.links` con exactamente los campos existentes:

```json
{"kind":"nominal_complement_de","target":"paper_descriptor","evidence":[{"start_byte":16,"end_byte":32,"quote":"bandeja de papel"}]}
```

El owner es la referencia que contiene el link. `target` es el ID de otra referencia local; no texto libre. El perfil `request_context_profile` sigue teniendo solo `version` y `features`; versión `documentary_request_context_v1`, feature nueva `nominal_complement_de`. La lista cerrada pasa de cinco a seis features posibles y su máximo de cinco a seis, sin duplicados y manteniendo dependencias previas. Sin feature, el kind nuevo es inválido. Declarar el feature no relaja otras formas ni activa producto.

Para esta versión mínima, el target es un descriptor nominal léxico: `denotation=entity`, `name=null`, `declared_class` explícita y no vacía, `anaphora=null`, sin binding. «papel» se mantiene como uso nominal local de masa/descriptor, identificado por su incidencia entrante nominal_de; entity no añade una afirmación de existencia física, hoja o lote. No se cambia a class: eso podría transformar una modificación de uso nominal en propiedad de una clase entera. `semantic_type` puede permanecer ausente; la capitalización y el link no crean tipos. Otros tipos de complemento requerirían contrato explícito posterior, no una coerción silenciosa de este perfil.

El núcleo owner se toma de `name` si existe; de lo contrario, de `declared_class`. No hay fallback a cita de cláusula ni a mention arbitraria. El núcleo target es su `declared_class`. El Evidence es único y debe ser exactamente la construcción contigua:

`owner_core + whitespace + de + whitespace + target_core`

Los spans de ambos núcleos son sus campos existentes, no búsquedas por substring en una quote amplia. El token intermedio es exactamente `de`, con límites de palabra; no se aceptan «en», «del», «para», «desde», material expresado en una cláusula diferente o una coincidencia de letras dentro de otra palabra. Se permiten únicamente espacios Unicode en los separadores; no puntuación, otra cláusula o contenido nominal omitido. Los offsets UTF-8 son exactos y el Evidence coincide con los bytes de fuente, desde el inicio de owner_core hasta el fin de target_core. La mention local owner contiene la construcción; la mention target contiene su núcleo. Menciones repetidas requieren sus spans concretos, no la primera ocurrencia encontrada.

Para un owner anónimo, una futura representación descompuesta usaría `declared_class=bandeja`, target descriptor `papel`, y conservaría la mención completa `bandeja de papel` con el link. El raw anterior, cuya clase literal es `bandeja de papel`, permanece intacto. No puede añadirse el link fingiendo que ese span entero es ya el núcleo `bandeja`. Para «bandeja Berta de papel», el Evidence puede ser `Berta de papel` y el núcleo owner el nombre `Berta`; la clase `bandeja` se conserva con su evidencia aparte. Este es un contrato simple deliberadamente acotado, no un parser nominal general.

Cada target de uso nominal pertenece a una sola incidencia owner local (salvo IDs alias de ese mismo owner ya demostrados); compartir el mismo target entre owners distintos es inválido en este subset, aunque el descriptor literal sea idéntico. No se deduplica `papel` entre construcciones para ahorrar nodos. Owner y target pertenecen al mismo scope/entorno de rama y binder. Se rechazan self-link, target ausente, ciclos nominales, deudas de identidad/alias contradictorio y reutilización del descriptor como entidad global. El subset mínimo es flat: target sin links, binder, anáfora o alias nombrado; todos sus alias explícitos conservan el mismo uso nominal, descriptor y tipo compatible. Un target con restricciones anidadas no soportadas queda pendiente, nunca se limpia para admitirlo. Restricciones existentes del owner se conservan. El perfil no licencia predicados nuevos o descripciones arbitrarias.

## Semántica del testigo

El testigo denota `NominalDe(head_occurrence, tail_descriptor)` en el scope local. Afirma la construcción dirigida «núcleo de complemento», no una variable que el consumidor pueda resolver como cualquier relación del mundo. `semantic subtype unspecified` es parte fija del significado del kind; no requiere otro campo ni una lista de lecturas materiales/funcionales. La forma exacta se sustenta en las evidencias y referencias; la cita no sustituye a sus operandos.

Esta relación restringe la descripción del owner. No entra en la clausura Same ni prueba que dos owners anónimos sean el mismo objeto por compartir `bandeja de papel`. Tampoco identifica dos papeles físicos: comparar dos usos nominales `papel` es comparar una restricción léxica acotada, sin emitir global `Same` para entidades anónimas. Un testigo de descripción puede indicar esa compatibilidad, pero conserva `identity_proven=false` cuando no haya prueba independiente.

«La bandeja está ubicada sobre el papel» o «la bandeja está hecha de papel» contienen relaciones más específicas. Sustituirlas solo por NominalDe pierde información, aunque una porción de la superficie contenga «de papel». Debe conservarse la proposición espacial/material íntegra con el carrier competente. Esta primitiva no descarga esa obligación; si no existe carrier, sigue faltante. Una asociación incidental con papel sin construcción nominal «de» no satisface el nuevo kind, y el nuevo kind no se convierte automáticamente en `associated_with` para aprovechar sus rutas actuales.

Una clase literal completa existente puede ya conservar el descriptor como unidad opaca cuando la tarea solo requiere igualdad literal del descriptor completo. No se exige reemplazarla universalmente. La nueva primitiva solo aporta capacidad si se necesita y verifica la estructura composicional de sus operandos; no es un requisito artificial para aprobar todos los nominales.

## Consumo y límites del motor existente

El DTO Link ya es suficiente: `validation.rs` define sus campos cerrados kind/target/evidence. Su enum actual no incluye este kind; `request_context_profile.rs` permite exactamente cinco features y máximo cinco. Es necesario ampliar permisos y añadir validación específica de construcción, no un DTO nuevo.

`answer_reference_pattern` compara los kinds distintos de associated_with mediante igualdad de kind y recursión. Esa parte ofrece un punto reutilizable, pero no basta para declarar implementación mínima completa. `identity_depth` aplica guard de asociaciones y luego contrasta específicamente owned_by: ignoraría nominal_de en una identidad de nombre coincidente. `quantified_match::associated_targets` recoge solo associated_with sobre alias; la correspondencia cuantificada restringe kinds a owned_by/part_of/associated_with. Las anclas parciales recorren links genéricamente, pero deben conservar la restricción nueva también desde el lado fuente y todos los alias. Por tanto, añadir el enum y confiar en matching genérico deja una brecha demostrable por lectura estática.

Reutilizar el motor actual requiere un guard restrictivo tipado para nominal_de, aplicado antes de aceptar identidad nominal, ancla parcial o patrones. El matching cuantificado queda unsupported en el subset mínimo; no se amplía su whitelist automáticamente. Reúne restricciones de todos los aliases Same compatibles, de ambos lados, sin que un alias sin link o un nombre coincidente permita eludirlas. `Different` en el mismo componente bloquea; Unknown no acredita diferencia. Un link nominal nunca crea un alias.

La comparación de targets léxicos exige clase literal y denotación entity iguales, tipos explícitos compatibles, scopes y todas las restricciones del subset flat mediante mapa local. No llama identidad global esperando obtener Same entre descriptores anónimos; no elimina sus deudas para fabricar ese resultado. Esta comparación es satisfacción de un descriptor, limitada al nuevo carrier. No amplía el significado general de identity ni el matching de asociados históricos.

La colección de links nominales restrictivos es conjunto tipado: owner, kind, target descriptor y restricciones; no se satisface seleccionando una arista favorable e ignorando las demás. Las restricciones adicionales de cualquier alias se conservan. Si falta una en el otro lado, una conclusión completa que dependa de esa correspondencia queda bloqueada o parcial con deuda explícita, según el contrato consumidor. Un match de descriptor no identifica automáticamente su owner: requiere ancla independiente o prueba de patrón acotado. El subset mínimo no autoriza correspondencia cuantificada nueva.

El uso nominal target no puede publicarse como sujeto global: una modificación local como «papel grande» debe conservar su owner de uso nominal y su naturaleza de descriptor léxico contextual, sin universalización, instanciación o property distributive de todo papel. Ni siquiera compartir scope root permite esa promoción. Si un consumidor actual no puede conservar esa localidad en su salida, la ruta queda unsupported; no se aprueba el feature ignorando ese canal. Una representación con propiedad ordinaria sobre papel fuera de ese contexto no descarga la obligación.

Los links target participan en las dependencias efectivas, elegibilidad y detección de ciclos. Owner→target no publica un hecho independiente sobre el target. Toda salida conserva el link y el descriptor fuente íntegros; no hay fallback que los quite al fallar la ruta especializada. En negaciones y refutaciones se exige el mismo owner, scope, fundamento y restricciones; no se usa una negación con otro complemento para vetar globalmente los hechos de la misma clase.

Canonicalización y firmas incorporan kind dirigido, núcleo/descriptor tipado, owner local, target de uso léxico canónico y restricciones de aliases, binder/rama/scope. Reordenar links y renombrar IDs mantiene la firma; cambiar papel por metal, el kind o el owner la cambia. `associated_with` y `nominal_complement_de` nunca colapsan. Recursión, expansión de aliases, comparaciones y serialización cargan el mismo presupuesto compartido de la operación, sin reinicio por head, target, solicitud o fallback. Agotamiento deja deuda y payload; no elimina links para lograr éxito.

## Controles necesarios, todavía no ejecutados

| Control válido salvo donde se indica | Resultado exigido |
| --- | --- |
| Nominal real «bandeja de papel» frente a asociación incidental con papel | Solo la construcción real habilita nominal_de; asociación nunca la satisface. |
| Borrar el link manteniendo mención completa y evidencia | Se pierde la restricción composicional en el contrato que la requiere; no reconstrucción desde quote ni aprobación por flag. |
| Cambiar target a metal conservando owner, con control textual/evidencia coherente | Cambia firma y compatibilidad; no satisface descriptor papel. |
| Cambiar marker a en/para/del o cita sin ambos núcleos | Rechazo de forma; se informa aparte de rechazos semánticos válidos. |
| Invertir dirección o usar owner de otra ocurrencia | Rechazo o incompatibilidad; no propiedad global transferible. |
| Mismo nombre de owner y mismo descriptor | El guard conserva nominal_de; positivo solo con identidad/ancla independiente y demás restricciones válidas. |
| Mismo nombre de owner y distinto descriptor | No bypass por nombre; conclusión completa bloqueada. |
| Dos owners anónimos con idéntico descriptor | Compatibilidad descriptiva posible; global Same sigue sin demostrarse. |
| Alias fuente con papel y alias omitido/metal; target con links/binder | Todos participan; alias no rescata, incompatibilidad/deuda no se ignora; target fuera del subset queda unsupported. |
| Uso local «papel grande» extraído como propiedad de todo papel / instanciado sobre otra hoja | Rechazo de fidelidad y bloqueo de salida global o cuantificada; conservar descriptor contextual y owner local. |
| Negación de owner/descriptor distinto | No refuta ni veta el owner compatible; negación exacta conserva su alcance. |
| «Bandeja ubicada sobre el papel» reemplazada solo por nominal_de | Falla fidelidad por pérdida espacial, incluso si el JSON resultante fuera válido. |
| «Bandeja hecha de papel» reemplazada solo por nominal_de | Falla fidelidad material; no reinterpretación por substring «de papel». |
| Filtro histórico, patrón, ruta parcial y matching cuantificado | No eluden el link por ser genéricos; perfil ausente conserva comportamiento histórico. |
| Ciclo, ID renombrado, conjunto reordenado, límite compartido | Ciclo/deuda bloquea; renombrado/orden conserva; agotamiento no recorta favorablemente. |

Los positivos y negativos semánticos requieren validación documental completa antes de interpretar matching. Controles manuales no prueban extracción automática, fidelidad del ítem s7-041 ni resolución de «grande». Esta revisión propone una extensión acotada comprobable y localiza el trabajo mínimo que el motor aún necesitaría; no comunica una calidad ya obtenida.
