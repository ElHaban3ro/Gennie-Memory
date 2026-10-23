# Auditoría de minimalidad: suficiencia referencial y tarea pendiente

Fecha: 2026-10-22. Estado: propuesta documental acotada, sin implementación ni ejecución. La puerta histórica **116 complete + faithful permanece fallida**; no se cambian raws, dictámenes, etiquetas, estados ni denominador.

## Decisión

Las formas existentes evitan ampliar el DTO, pero no bastan por sí solas como contrato operativo probado. Para la credencial, `property(subject:ref, attribute:attribute_id, value:term)` puede transportar un valor referencial junto con `references[].links.owned_by`; hace falta registrar un atributo de suficiencia referencial y fijar sus guards. `class_suffices` no representa fielmente un individuo mediante texto de clase. Para Atlas, `quantity(subject:ref, attribute:attribute_id, amount:quantity)` ya conserva la cuenta y `pending_action` admite tarea explícita: la ausencia de ese calificador puede conservar «tarea no especificada por esta fuente» sin inventar acción, siempre que el comparador no la trate como comodín. Esa semántica y su control necesitan especificación y comprobación propias.

Por tanto, el delta preferido es **vocabulario y reglas cerradas sobre formas existentes**, antes de crear `item_suffices` o un campo nuevo. La existencia de `property.value:term` no demuestra que su comportamiento actual preserve esta semántica; este informe no atribuye soporte ejecutado. La alternativa de predicado se reserva si un preflight posterior demuestra una incompatibilidad concreta de guards que la forma property no pueda resolver sin romper su contrato.

## Lectura y límites

Se leyeron AGENTS, reglas backend/testing, índice de memoria de proyecto, índice de decisiones, contrato local V3 y [auditoría de incertidumbre](documentary-v3-uncertainty-gate-audit-2026-10-22.md). Se consultaron exclusivamente estas dos entradas de fuente del bundle `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-author-corrections-v1/known116-bundle/`:

- `d-5b4860f3cf848e9dcbf9985a-input.json`: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.»
- `d-79d8fc5f9965fe9d130446dd-input.json`: «Del expediente Atlas no quedan páginas pendientes.»

Del archivo `consolidated-failure-diagnosis-v1/source-diagnosis.json` se extrajeron solo los dos ítems con esos IDs. Sus clasificaciones se conservan como diagnóstico previo, no como nuevos dictámenes ni aceptación. No se consultaron preguntas, contrapartes, gold, utilidad, particiones reservadas ni otras entradas raw. No se leyó código para inferir comportamiento nuevo ni se ejecutaron Cargo, binarios o API. La comprobación es documental.

## 1. Credencial: núcleo afirmado y pérdida actual

La segunda cláusula afirma suficiencia de una credencial referenciada; no dice que todas las credenciales de alumna basten. El raw usa `class_suffices(subject=ina,class=text("su credencial de alumna"))`, carece de referencia para la credencial y de vínculo posesivo. Copiar esas palabras no conserva el objeto, su restricción posesiva ni su función referencial. La cobertura declara tanto represented como unsupported para segmentos de ese claim; ninguna de ambas etiquetas resuelve la pérdida.

La eliminación ocurrida y la no exigencia normativa son afirmaciones separadas en Faro. La propuesta se limita a la suficiencia; no reconstruye una matrícula, presentación, permiso, regla objetivo ni acto cumplido. Tampoco introduce una relación global entre la eliminación y toda obligación posible.

### Forma mínima candidata

Registrar exactamente el atributo nuevo `sufficient_item`. Usar el átomo existente:

`property(subject: ref, attribute: symbol sufficient_item, value: ref credential)`

Aquí `subject` es el beneficiario textual de la suficiencia, no el dueño deducido automáticamente; `value` es el ítem suficiente, no una clase, un requisito necesario ni una acción. Proposición `judgment:fact`, `status:state`, `basis:normative`, polaridad positiva y `normative_scope:faro`; el ámbito debe estar respaldado por el texto. El perfil `attribute_profiles`, si se activa para property.value, tendrá fila de `sufficient_item` que admita exclusivamente `ref`, sin unidades. No habilitar text, quantity, symbol ni described_ref por conveniencia. El perfil inicial se limita a subject y value con denotación entity, sin grupo, clase ni binder: la marca distributive del predicado genérico no autoriza repartir esta suficiencia entre miembros, instanciarla ni elevarla a una clase. Fuera de ese alcance se conserva unsupported, sin coerción.

La referencia `credential` tiene denotación entity y mención «su credencial de alumna». Su descripción debe conservar «de alumna», sin convertir «alumna» en nombre propio, persona adicional o matrícula ocurrida. Puede declararse clase literal «credencial de alumna» si la guía permite esa frase nominal como clase acreditada; si se segmenta solo «credencial», el complemento «de alumna» requiere preservación operativa adicional y no puede quedar únicamente en la mención. Esta decisión de segmentación debe cerrarse prospectivamente: no aceptar ambas con pérdida del complemento.

Para posesivo resuelto, la arista exacta es `credential --owned_by--> owner_ref`, con evidencia propia «su credencial de alumna» más el antecedente que acredita `owner_ref`. No invertirla, convertirla en associated_with, copiarla al beneficiario o hacer transitividad. Dependencias efectivas: claim→credential, claim→beneficiary, claim→Faro; credential→owner_ref y scopes/evidencias de cada nodo. El claim depende de la descripción y del posesivo completos aunque la evidencia se aloje en referencias.

### Antecedente y guard de publicación

Ina es un antecedente posible por la continuidad «para Ina; basta su…». Eso no constituye aquí una resolución crítica independiente ni elimina automáticamente otras lecturas del posesivo. **No se fija `owned_by:ina` solo porque el subject sea Ina.** Si el texto y la revisión aislada acreditan el antecedente, se usa esa referencia real. Si la fuente admite ambigüedad, se conserva como tal y no se fabrica un dueño desconocido existencial, una lista arbitraria de personas o una identidad por cercanía.

Una anáfora no resuelta puede conservar material textual, pero no descarga la restricción posesiva. Si el contrato vigente no transporta el posesivo pendiente con sus lecturas, la representación sigue parcial/unsupported. Un futuro sobre de lecturas solo podría cerrarlo con inventario exhaustivo justificado y cada hoja completa; un flag unresolved o el acuerdo entre hojas incompletas no basta. Beneficiario y dueño siguen siendo roles distintos incluso cuando su identidad esté acreditada.

La ruta property debe conservar en cualquier witness el atributo, valor referencial completo, dueño, descripción, ámbito, fundamento y qualifiers. No rebajar a valor desnudo ni permitir matches por clase. Quitar el vínculo posesivo o cambiar dueño debe bloquear la equivalencia; que dos objetos sean credenciales de alumna no prueba identidad. Suficiencia no implica necesidad, posesión física actual, presentación, autorización global ni cumplimiento.

### Comparación con `item_suffices`

El predicado alternativo cerrado sería `item_suffices(beneficiary:ref,item:ref)`, no distributivo, con los mismos límites normativos, referencias y dependencias. No necesitaría ampliar la forma Atom, pero sí vocabulario y semántica operativa. El nombre del predicado no resuelve el posesivo. La forma property ya ofrece la aridad y los tipos requeridos: añadir ambos carriers ahora duplicaría semántica y crearía una equivalencia nueva que habría que demostrar. Se prefiere un solo carrier property si supera los controles; si no, registrar la limitación concreta antes de elegir el predicado. No se declara equivalencia automática con class_suffices.

## 2. Atlas: cero y tarea no especificada

La fuente afirma que no quedan páginas pendientes del expediente Atlas. El raw conserva `quantity(subject=atlas,attribute=pending_pages,amount=exact 0 páginas)`, positivo fact/state y basis description. No afirma qué trabajo tendría que hacerse con las páginas, cuánto había antes ni qué evento terminó. Digitalizar, revisar, entregar y cualquier tarea concreta permanecen sin acreditar.

El diagnóstico previo señala que no hace falta capacidad nueva para conservar cuenta y desconocimiento. También conserva una deuda administrativa: `coverage.issue_id=pending_task_unspecified` no coincide con el Issue identificado por el reporte previo. Este informe no inspecciona otros raws, no corrige ese campo y no usa su eventual corrección como resolución semántica.

### Selector mínimo sin campo nuevo

Definir un perfil prospectivo cerrado `pending_task_scope_v1`, aplicable exclusivamente a `quantity` con atributo `pending_pages` y amount quantity. No crea una acción ni un evento. La vista operativa derivada tiene exactamente dos variantes:

| Selector derivado | Condición de fuente | Significado |
| --- | --- | --- |
| `specified(action_id)` | Un único qualifier configuration conocido `pending_action`, valor symbol de fixed_action_rows, con evidencia fuente que expresa esa tarea o antecedente único explícito en el mismo texto | La cuenta corresponde a esa tarea declarada. |
| `unspecified_no_source` | Ningún pending_action y revisión fuente acredita que el texto no determina tarea | La fuente cuenta páginas pendientes sin fijar tarea. No es una variable universal ni una tarea anónima ejecutada. |

El selector es una vista privada especificada aquí, no una key añadida al raw ni un estado ya implementado. La ausencia no puede distinguir por sí sola unspecified de omisión del extractor: si hay tarea explícita en el texto y falta el qualifier, es extraction_failure y falla completitud. Si hay varios qualifiers, valor desconocido, conflicto, dependencia pendiente o antecedente no único, no entra en ninguna variante aprobada; queda bloqueado con la deuda correspondiente. No convertirlo en unspecified_no_source para rescatarlo.

Para el texto Atlas el único selector candidato es unspecified_no_source. Conservar `0 páginas`, `pending_pages`, sujeto y fuente de alcance intactos. No añadir `pending_action=digitize`, negación de digitize, un action_actor, evento completado o total de páginas del expediente. La polaridad positiva de la cuenta cero no debe convertirse en negación de una tarea.

### Guard de no transferencia

Antes de cualquier respuesta numérica, cierre, refutación, composición o fallback que consuma esta cuenta, exigir igualdad de tarea además de sujeto, atributo, unidad, tiempo, modalidad y qualifiers. La matriz normativa del perfil es:

| Fuente | Consumo que exige una tarea | Resultado permitido |
| --- | --- | --- |
| specified(A) | specified(A), identidad de ActionId acreditada | Puede continuar a los demás guards; no acredita por sí solo respuesta. |
| specified(A) | specified(B), A distinto de B | No transferir la cuenta; incompatibilidad de tarea no refuta el dato de B. |
| unspecified_no_source | specified(A) | Pendiente; nunca publicar cero para A. |
| unspecified_no_source | unspecified_no_source | Solo conservar la afirmación literal sin tarea; nunca universalizarla. |

Si un consumidor no trae selector, no interpretarlo como aceptación de toda tarea. Exigir el perfil o bloquear la publicación de amount. Un eventual canal parcial puede transportar la proposición fuente íntegra con la indeterminación, pero no completar un slot específico ni fabricar un enlace de relevancia. La cuenta cero no implica cero pendientes en todas las tareas, todas las épocas, otras unidades u otros expedientes. No hay proyección quantity→evento.

## Controles operativos prospectivos

Son requisitos para una implementación futura, no tests ejecutados ni fixtures incorporados. No requieren preguntas conocidas ni contrapartes: pueden verificarse con construcciones sintéticas fuente y un consumidor de contrato aislado.

1. Forma válida positiva: property suficiente con referencia, descripción, owned_by acreditado y ámbito; quantity de pendientes con selector unspecified_no_source. Su validación estructural debe preceder a interpretar cualquier control negativo.
2. Mantener cita y borrar solo owned_by: debe perder la equivalencia posesiva. Cambiar el dueño por otra referencia tampoco puede conservarla. Cambiar beneficiario no cambia dueño automáticamente. La arista exige la referencia real respaldada.
3. Mantener posesivo y borrar o debilitar «de alumna»: falla preservación. Sustituir valor referencial por una clase text, o item_suffices por class_suffices, no conserva la afirmación.
4. Suficiencia no ejecuta nada: retirar toda referencia a acciones no impide conservar el estado suficiente; añadir matrícula/presentación/permisos derivados debe rechazarse. Eliminar ámbito Faro no permite un hecho global.
5. Fuente de pendientes sin tarea: no se produce evento ni acción nueva; amount cero se conserva solo en el claim original. Consumirla con cualquier ActionId concreto debe quedar pendiente en todas las rutas, incluidos fallback y restricciones cerradas.
6. Fuente sintética con tarea explícita y qualifier eliminado: no puede pasar como unspecified_no_source. Fuente con A no equivale a B; fuente con dos tareas no se reduce a una por conveniencia. Issue_id null o corregido no cambia estos juicios.
7. Invariancia: renombrar IDs y permutar filas preservando grafo no cambia resultado. Intercambiar propietario, ámbito, tarea, unidad o sujeto sí debe cambiar el guard. Dos lecturas incompletas equivalentes no acreditan fidelidad.
8. Deuda y procedencia: candidato local elegible, cita exacta, estado unresolved o cobertura represented no sustituyen completitud global ni revisión de atribución/preservación. Una revisión nueva retiene todos los 116 originales y sus fallos.

## Coste y cierre

No se propone modificación de DTO. El coste futuro mínimo es registrar un atributo, cerrar perfiles de valor/tarea, comprobar dependencias y todas las rutas de consumo, preparar controles positivos/negativos y revisión aislada de semántica. El posible coste extra de posesivos ambiguos no se oculta: si no hay carrier fiel de incertidumbre con owner verificable, ese caso permanece sin resolver, aunque property pueda transportar el caso resuelto.

Solo se creó este documento. Sin compilación, ejecución, nueva extracción, recursos de proveedor ni evidencia de mejora/adopción. Un preflight futuro puede rechazar cualquiera de estas propuestas; no dispensa la puerta histórica ni demuestra completitud de los dos ítems.
