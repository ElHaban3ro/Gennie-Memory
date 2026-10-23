# Contexto local de pedidos documentales: contrato experimental V1

Estado posterior: implementación experimental y verificación local en el [informe de resultados](documentary-v3-request-context-results-2026-10-22.md). El texto siguiente conserva la especificación previa; su copia V3 congelada permanece en los artefactos. No acredita cierre de extracción autónoma ni adopción.

Estado: propuesta de diseño revisada el 22 de octubre de 2026 tras las críticas estáticas `critical-review-v1` y `critical-review-v2`; esta revisión retira la promesa de corrección normativa negativa señalada en RC2-B1 y queda pendiente de crítica final e implementación. Los freezes `design-v1.md` y `design-v2.md` conservan los diseños criticados sin cambios. No acredita extracción, utilidad, calidad, métricas ni puertas aprobadas. No modifica código, vocabulario, raws, fuentes, etiquetas ni resultados históricos.

## Alcance y evidencia disponible

Se consultaron únicamente las preguntas, su contrato entregado y el diagnóstico de `question-only-package.json` y `question-diagnosis.json` en `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-failure-diagnosis-v1/`, además del código de validación indicado y las reglas backend/testing. Para esta revisión se consultaron también `critical-review-v1.json`, `critical-review-v1.md` y `critical-review-v2.md` y las entradas/firmas actuales de `documentary_v3_io.rs`, `documentary_v3_io/readings.rs` y `documentary_v3/pair.rs`, autorizadas para cerrar el diseño. No se consultaron contrapartes fuente, pares ni gold. Los ejemplos siguientes son abstractos y nuevos; no describen las fuentes del banco.

Las brechas objeto del diseño son: (a) una finalidad o condición de matrícula asociada al acompañamiento normativamente preguntado, con actor interrogado y control del infinitivo potencialmente pendiente; (b) una autonomía máxima preguntada durante una ruta nominal, sin operación afirmada. La tercera pregunta del paquete, relativa a scopes, queda fuera.

El diseño preserva por separado identidad del actor, destinatario, destino de la acción subordinada, fuerza de la relación y condición nominal de la magnitud. Una representación fiel puede seguir sin respuesta y con incertidumbre. Una cita literal válida no prueba que la interpretación sea correcta.

## Reutilización y coste mínimo

| Carrier actual | Reutilización admisible | Límite que permanece |
| --- | --- | --- |
| `Request.target`, `slot`, `class_filter` | Se conservan sin sustitución. El actor interrogado sigue omitido del átomo y localizado por `slot`. | No expresan relación con una segunda acción. |
| `normative_scope_filter` | Conserva el ámbito normativo. | Un curso no identifica por sí solo una acción ni su finalidad. |
| `AtomPattern` | Conserva patrones sin publicar existencia; sus roles conocidos usan el vocabulario actual. | No basta para una acción subordinada cuyo destino no es el objeto de acompañamiento; tampoco define roles abiertos explícitos. |
| `Qualifier.configuration` / `quantity_extent` | Conserva máxima literalmente sobre el mismo target. | Ni `route` ni `context` se amplían: sus guards actuales no autorizan una magnitud nominal sin operación. |
| `guard`, `premise_ids`, `body.rule` | Se conservan cuando el texto realmente afirma condición/premisa/regla en su sentido actual. | Finalidad no equivale a guard cumplido ni premisa fáctica. Una regla con actor inventado pierde el interrogativo. |
| `for_purpose`, `Reference.links`, `Dependency` | Sin cambio de significado. | No reifican una ocurrencia ni sustituyen una relación semántica local al pedido. |
| `admission_request_context_v1` | Sirve de precedente para perfil cerrado optativo. | Es un permiso específico de admisión, no una equivalencia general de predicados/contextos. |

Se proponen dos relaciones locales en un campo opcional `context_bindings`, utilizable en `Request` y en `Claim` bajo perfil explícito. Compartir el DTO permite comparar una condición acreditada en fuente sin convertirla en propiedad mundial. Además, un operador fuente optativo `normative_open_pattern` permite conservar una norma impersonal sin fabricar una persona ni una ocurrencia. Este operador adicional tiene coste real: validación, dependencias y comparación específicas; no se obtiene relajando `Atom`.

El mínimo de conservación de preguntas no requiere ese operador fuente; el mínimo que también representa una norma impersonal útil sí lo requiere. Una fuente normativa con actor expresamente identificado usa la proposición normal y no necesita el operador abierto. No se añade un lenguaje general de eventos, un motor de control del infinitivo ni razonamiento transitivo.

## Activación y compatibilidad

Perfil nuevo propuesto en `/vocabulary/request_context_profile`, no incorporado al vocabulario vigente:

```json
{"request_context_profile":{"version":"documentary_request_context_v1","features":["nominal_during","normative_action_relation","normative_open_pattern"]}}
```

El objeto tiene exactamente `version` y `features`. `features` es un conjunto no vacío y sin duplicados de los tres literales anteriores. `normative_open_pattern` requiere `normative_action_relation`. No hay flags implícitos, alias ni promoción automática de otros perfiles. `nominal_during` habilita únicamente ese elemento de contexto; `normative_action_relation` habilita únicamente la relación de acción en los owners admitidos; `normative_open_pattern` habilita únicamente el body fuente adicional y nunca funciona sin la relación habilitada.

Sin perfil, `context_bindings` y el nuevo body son campos/variantes desconocidos: error de forma. Los documentos históricos válidos conservan exactamente su forma, informes y comportamiento. Con perfil, `context_bindings` puede faltar en cada owner; si existe debe ser array no vacío. `null` y `[]` son inválidos. Un perfil presente pero sin uso no cambia los informes antiguos salvo las identidades de vocabulario que efectivamente correspondan a una entrada nueva; no se reescriben hashes para fingir identidad histórica.

Se verifica legacy por entradas y vocabulario originales idénticos, sin el perfil. Probar una entrada derivada con vocabulario nuevo es otra comparación, no replay idéntico. Un consumidor antiguo debe rechazar la extensión; no debe ignorarla y publicar una respuesta descontextualizada. Los comandos histórico y contextual deben declarar aceptación/rechazo explícito; no hay retrocompatibilidad semántica implícita.

## Entradas públicas, adaptadores y recepción

La tabla fija comportamiento propuesto; no describe una implementación realizada. «Legacy» significa perfil ausente y carriers nuevos ausentes. Un carrier nuevo sin su feature/perfil es siempre inválido. Un perfil presente, incluso sin uso, nunca se ignora en una entrada de comparación histórica.

| Entrada actual | Forma | Legacy | Perfil V1 presente |
| --- | --- | --- | --- |
| `documentary_v3::validate(Value, text, vocabulary)` | flat | Informe actual idéntico | Valida DTO y conserva carrier; no clasifica. |
| `documentary_v3::classify_pair` | dos `DocumentResult` flat | Motor actual idéntico | Rechazo explícito antes del motor (`status:invalid`, sin selección, `complete_answer:false`), aunque los documentos validen. No downgrade. |
| `documentary_v3::classify_pair_context` | dos `DocumentResult` flat | Contexto V2 actual idéntico | Acepta solo mediante interceptor V1 y presupuesto compartido; se conserva la política contextual V2 restante. |
| `documentary_v3_io::run(..., paired:false)` | flat individual | Validador actual | Acepta validación V1; no clasifica. Envelope sigue siendo forma no admitida por esta entrada. |
| `documentary_v3_io::run(..., paired:true)` | flat/flat | `classify_pair` actual | Rechaza antes de clasificación con error explícito y sin informe de éxito; jamás cambia a contextual automáticamente. Cualquier envelope se rechaza. |
| `documentary_v3_io::readings::validate` | flat o envelope | Validador actual/hojas actuales | Valida flat y todas las hojas con perfil, conservando eligibility global. |
| `readings::run(..., paired:false)` | flat o envelope individual | Validación actual | Acepta validación V1; no clasifica. |
| `readings::run(..., paired:true)` | flat/flat, flat/envelope, envelope/flat, envelope/envelope | Clasificación histórica flat o matriz actual | Rechaza explícitamente antes de clasificar/matriciar: no selecciona por V1 y no envía hojas a `classify_pair`. |
| `readings::run_context` | flat/flat | `classify_pair_context` actual | Acepta mediante interceptor y presupuesto único. |
| `readings::run_context` | flat/envelope, envelope/flat, envelope/envelope | Matriz contextual actual | Acepta mediante interceptor en todas las celdas; gate global y firmas V1 definidos abajo. |

`run_with_context`, `classify_with_context` y `side` son wrappers internos: deben propagar la misma política de su entrada pública, no introducir otro camino. `classify_pair` no recibe envelope; un envelope requiere `readings` y no se puede aplanar a la primera hoja. No se crea aquí una nueva entrada pública ni un selector CLI implícito. La implementación deberá verificar el dispatch CLI que usa estos adaptadores y registrar la ruta real.

Rechazo de claves duplicadas: para entradas bytes con perfil V1 hace falta parser previo que detecte duplicados recursivamente antes de construir `Value`; un chequeo `shape` después de `serde_json::from_slice` no los recupera. Se aplica antes de `side`, validación de hojas, generación de matriz y clasificación. El dispatcher puede escanear primero bytes con límites y sin cambiar comportamiento legacy; si detecta carrier/perfil nuevo, los duplicados invalidan la recepción V1. Una API que ya recibe `Value` no puede certificar ausencia de duplicados en bytes históricos: su garantía es forma de ese valor, con origen no certificado. No debe afirmar validación de recepción byte a byte. Nada de esto reescribe ni reinterpreta los raws originales.

## DTO cerrado

Todos los objetos enumeran a continuación sus campos exactos; se rechazan extras y claves JSON duplicadas. `Id` es string no vacío y no blanco, de hasta 128 bytes UTF-8. `Evidence` reutiliza exactamente `{start_byte:u32,end_byte:u32,quote:string}`, intervalo UTF-8 semiabierto no vacío sobre el texto original y coincidencia byte a byte. Todo `Evidence[]` indicado es no vacío. No se inventan offsets ni spans para información ausente. IDs son locales al documento; coincidencia de strings entre documentos no prueba identidad.

`RefTerm` reutiliza `{kind:"ref",id:Id}` exclusivamente, sin `described_ref`, `text`, wildcard ni IDs externos. `SymbolTerm` reutiliza `{kind:"symbol",id:Id,evidence:Evidence[]}`; el ID debe existir en `fixed_action_rows`. Referencias, clases, aliases y ámbitos se validan por el contrato actual; un nombre propio no crea clase, propiedad ni vínculo.

`context_bindings` contiene de uno a ocho elementos, con IDs únicos locales al owner; esos IDs no se registran como referencias o claims globales. Ningún elemento contiene otro `context_bindings`. Scope efectivo, rama y binder son los del owner; no existe un `scope_id` alternativo en el elemento.

### Contexto nominal de una magnitud

```text
NominalDuring = {
  id:Id,
  kind:"nominal_during",
  anchor:RefTerm,
  evidence:Evidence[]
}
```

En `Request`, el owner debe ser `mode:answer`, `slot:amount`, `target.atom.predicate:quantity`, `judgment:fact`, `status:state`, polaridad positiva. El atributo es un ID fijo existente; no se cambia `autonomy` a otro atributo. La relación modifica el valor interrogado del target completo, incluidos sus qualifiers, nunca la referencia sujeto como hecho independiente. No exige fundamento de observación, capacidad o asignación: respeta `basis_requirement`.

En `Claim`, solo se admite un `body.kind:proposition` positivo `quantity`, `judgment:fact`, `status:state`, con cantidad expresamente afirmada. El contexto modifica esa misma cantidad; el fundamento original se conserva. Este primer contrato no añade el contexto a eventos, restricciones de valores, excepciones o reglas. Esas combinaciones quedan unsupported, sin borrar el contexto para seguir por fallback.

`anchor` debe ser una referencia nominal explícita con mención y nombre acreditados en scope accesible. V1 compara un solo `nominal_during` por owner. Dos contextos simultáneos, aunque ambos sean serializables en el array, son una combinación unsupported: conservar el array conjuntivo íntegro y su deuda, sin respuesta ni selección por esta ruta. No dividirlos en hojas. Las lecturas separadas se reservan a interpretaciones alternativas textualmente justificadas y completas; no distribuyen condiciones acumulativas. Admitir utilidad de una conjunción de varios contextos requiere ampliar explícitamente este contrato y criticar esa ampliación. `evidence` acredita la relación completa «durante + referente». No se descompone en fecha, tiempo de acción o recorrido ocurrido. Las restricciones adicionales del contexto solo pueden usarse si ya son representables fielmente por la referencia; si se pierden, coverage sigue pendiente y no se admite respuesta completa.

### Relación con un patrón de acción subordinado

```text
NormativeActionRelation = {
  id:Id,
  kind:"normative_action_relation",
  relation:Relation,
  action:SubordinateAction,
  evidence:Evidence[]
}
Relation = {kind:"purpose",evidence:Evidence[]}
         | {kind:"necessary_condition",evidence:Evidence[]}
         | {kind:"unresolved",alternatives:["purpose","necessary_condition"],evidence:Evidence[]}
SubordinateAction = {
  action_kind:SymbolTerm,
  actor:Participant,
  destination:RefTerm,
  evidence:Evidence[]
}
Participant = {kind:"reference",value:RefTerm,evidence:Evidence[]}
            | {kind:"owner_slot",role:"actor",evidence:Evidence[]}
            | {kind:"unresolved",reason:"unspecified"|"ambiguous_control"|"extraction_unresolved",evidence:Evidence[]}
```

Es un patrón no asertivo: no tiene `status:occurred`, `planned`, `basis`, existencia ni identidad propia. `destination` expresa exclusivamente el destino explícito de la acción subordinada, no su localización física ni un objeto genérico. La primera versión tiene solo actor y destino; si el texto exige otro participante, objeto, instrumento o vínculo de ocurrencia no representable, debe conservarlo como unsupported y bloquear completitud. No es permiso para descartar roles. La acción se identifica por un ID ya registrado, sin añadir equivalencias ni acciones desde la respuesta esperada.

El owner `Request` debe ser un patrón `action`, positivo, `judgment:required`, `status:state`, `mode:answer`, `slot:actor`, con actor ausente del target, `class_filter` respaldado, `basis_requirement:{kind:exact,value:normative}` y `normative_scope_filter` explícito. Se mantienen todos los argumentos conocidos del acompañamiento. V1 admite un único elemento de esta variante por owner; no se permite mezclarlo con `nominal_during` en el mismo owner.

La dirección es fija: el target del owner se requiere para la acción subordinada. `purpose` significa finalidad declarada; no demuestra necesidad lógica. `necessary_condition` expresa condición necesaria del acto subordinado; no demuestra suficiencia ni permiso para realizarlo. No se intercambian direcciones. Si el texto no decide entre ambas, se conserva `unresolved` con las dos alternativas, sin escoger una para facilitar matching.

`Participant.reference` requiere evidencia de control textual sobre ese participante; que una persona sea destinataria del acompañamiento no la convierte automáticamente en actor de matrícula. `owner_slot` conserva una identidad compartida expresamente acreditada con el actor preguntado, sin rellenarlo; la evidencia debe justificar ese control, no limitarse al infinitivo ambiguo. `unresolved` incluye `reason` obligatorio: `unspecified` cuando el texto no determina control, `ambiguous_control` cuando admite atribuciones distintas sin resolución y `extraction_unresolved` cuando la extracción no conservó control que podría constar. No intercambiar estos motivos. `unresolved` no crea referencia, persona genérica, estudiante existencial ni variable cuantificada. Su evidencia es el segmento cuya atribución queda pendiente. No incluye lista de personas inferidas.

Esta relación en `Claim` se admite sobre una proposición normativa requerida `action`, positiva, `status:state`, `basis:normative`, ámbito explícito y actor nombrado en el átomo completo; o sobre el body abierto siguiente. No se agrega a un hecho ocurrido para convertirlo en norma. En fuente, `owner_slot` solo es válido con el body abierto y su rol actor; una proposición completa usa `reference` o `unresolved`.

### Operador fuente normativo impersonal

```text
NormativeOpenPatternBody = {
  kind:"normative_open_pattern",
  target:AtomPattern,
  open_role:{role:"actor",class_filter:ClassFilter,evidence:Evidence[]},
  judgment:"required",
  status:"state",
  basis:"normative",
  normative_scope:RefId
}
```

`Claim` conserva su envoltura actual (id, scope, qualifiers, evidence, body) y opcionalmente `context_bindings` del DTO anterior. El `target` es `action` positivo con todos los roles del vocabulario excepto actor; el actor no debe estar simultáneamente presente. `class_filter` usa las variantes actuales cerradas y debe tener respaldo literal en la fuente. Exige exactamente una relación `normative_action_relation` en el owner. Qualifiers originales del claim permanecen en su lugar.

El operador afirma documentalmente una exigencia expresada como patrón abierto. No afirma que exista un tutor, que alguno acompañó o se matriculó, que todos los tutores estén obligados, ni que un individuo concreto pertenezca a la clase. No tiene cuantificador implícito. Una fuente realmente universal conserva su scope cuantificado explícito y exige la comprobación de dominio existente; V1 no la instancia ni la convierte a este body por conveniencia. Una disyunción de clases, un actor cuantificado o una prohibición necesitan su contrato propio si no se pueden representar sin pérdida; no se degradan silenciosamente a `required`.

El patrón abierto puede aportar contexto normativo útil y un filtro de clase; por sí solo nunca proporciona un ID de actor como respuesta. Los otros tipos de claims del documento siguen siendo independientes y conservan su evidencia.

## Scope, dependencias y deuda

Cada contexto está poseído por un solo request o claim, no es un endpoint global. El motor añade aristas implícitas owner→referencia para `anchor`, `destination` y `Participant.reference`, owner→símbolo para comprobación de vocabulario y owner→scope para visibilidad. `owner_slot` crea una dependencia local sobre el rol ausente, nunca una arista owner→owner que simule ciclo. No se añade ningún literal nuevo a `Dependency.role`: las aristas serializadas existentes de identidad, binding, normative_target y qualifiers se conservan; una redundancia coherente es admisible pero no sustituye las aristas implícitas.

La comprobación transitiva debe incluir todos los operandos anidados antes de filtrar claims. Requiere referencias presentes, scope accesible y la misma rama aplicable; no se lleva un binder de una rama hermana ni se resuelve un contexto por nombre aislado. Alias con desigualdad, anáfora pendiente, guards pendientes y errores de sus dependencias bloquean el testigo afectado. Una deuda no contamina hechos independientes.

Clasificación propuesta de incidencias, aún no implementada: forma/perfil/cita/rol/ID inválidos → `invalid`; fuerza relacional ambigua, control `ambiguous_control` o `extraction_unresolved` → `partial` con la localización exacta; control fielmente `unspecified` puede conservar un documento `complete` pero deja matching pendiente y no proporciona la ligadura; combinación fiel pero fuera de carriers V1 → `unsupported`. Las cadenas de incidencia deben fijarse con la implementación, sin alterar códigos históricos. El normalizado conserva el DTO completo incluso cuando no puede seleccionarlo.

Coverage no declara la cláusula ambigua totalmente resuelta: conserva el segmento, owner y deuda de control/fuerza. Representar la existencia de una ambigüedad no la descarga. El candidato debe exponer actor sin resolver, contexto sin acreditar, dato ausente, alcance/fundamento/qualifiers pendientes y límites de existencia por separado. No se fabrica una `Reference` para hacer desaparecer esa deuda.

## Comparación útil sin promoción de condiciones

Las comparaciones se construyen exclusivamente desde `DocumentResult` producido por validación de ambos documentos, con namespace, vocabulario y perfiles compatibles explícitos. El contrato conserva el DTO; no lo reduce a keywords. Se comprueba el contexto antes de cualquier salida de valor o fallback genérico, incluido truth/refutación/contexto. La ausencia de un contexto en fuente no prueba su falsedad ni autoriza omitirlo en pregunta. Prueba causal obligatoria: retirarlo de la fuente de un positivo debe quitar el apoyo contextual; intercambiarlo por otra ruta/destino/actor debe bloquear el testigo aunque el resto del target sea idéntico. Retirarlo de la pregunta cambia el pedido y su hash: nunca se usa esa pregunta simplificada para calificar el original. Si un adaptador entrega un owner nuevo sin su contexto declarado, debe conservar el fallo de integridad y no reenviarlo al motor histórico.

Reserva obligatoria de rutas: antes de despachar respuesta directa, refutación, comparación cuantificada, restricciones, composición de lecturas o fallback, inspeccionar los owners fuente/pregunta y la clausura de sus dependencias. Si alguno porta un contexto V1, la operación debe pasar por un interceptor contextual que solo devuelve `answer_match`, `partial_match`, `pending`, `incompatible` o `invalid`, acompañado del testigo o deuda; esos nombres son estados internos propuestos, no una modificación de métricas. Únicamente `answer_match` habilita una ruta de respuesta ya existente, y esa ruta debe transportar y revalidar el testigo íntegro al emitir. `partial_match` emite por la ruta parcial propia definida abajo; `pending` no puede caer al algoritmo histórico; `incompatible` no es refutación automática y `invalid` conserva el fallo técnico. Un contexto fuente no desaparece porque la pregunta carezca de él: un valor condicionado no demuestra un valor incondicional. Todos los cruces de una lectura cuantificada deben demostrar la ligadura; no se selecciona solo el cruce favorable.

Una salida parcial nueva exige pertinencia explícita acreditada por el mismo anclaje y relación documental conservada, además del payload y deudas íntegros. Compartir nombre, clase o atributo sin esa prueba deja solo un candidato pendiente. En toda salida parcial se fuerzan vacíos los términos, proposiciones de respuesta y bindings; ningún retorno de una ruta legacy puede elevarla a refutación o respuesta completa. La ausencia del perfil y de los carriers nuevos evita el interceptor y conserva los informes históricos byte a byte sobre sus entradas originales.

Un testigo contiene owner original de ambos lados, contexto(s), evidence, scope/rama, fundamento y mapa local de referencias. La correspondencia usa las reglas existentes de identidad y vínculos respaldados, incluyendo aliases y restricciones en ambos sentidos. El renombrado coherente de IDs no cambia la decisión; homónimos con anclas distintas no se unen. Los roles abiertos no son comodines universales. No se mezclan dos claims para fabricar una ocurrencia o la identidad de un actor.

| Evidencia fuente y pregunta compatibles | Resultado máximo autorizado por este contrato |
| --- | --- |
| Cantidad exacta de la misma magnitud/sujeto, mismo `nominal_during`, mismo alcance máxima/mínima y fundamento/guards acreditados | Apoyo con valor, sujeto a todas las comprobaciones actuales de cantidad, unidad y extent. El valor sigue calificado por contexto. |
| Cantidad de autonomía observada explícitamente en ruta A frente a autonomía máxima en ruta A | No responde al máximo por ser un dato observado. Contexto parcial por ruta propia solo si el atributo fuente es también `autonomy`, conserva esa diferencia y acredita contexto y pertinencia; valor de respuesta vacío. Una duración de operación no es ese carrier ni recibe esa utilidad. |
| Norma con actor explícitamente identificado, destinatario/destino/relación/control/ámbito iguales y clase del actor acreditada | Apoyo con actor como término; nunca matrícula ocurrida. |
| `normative_open_pattern` compatible, mismo anclaje destinatario/destino, clase, relación y control acreditados | Contexto normativo parcial con patrón y deudas originales. Sin actor de respuesta, sin bindings de identidad, sin proposición de respuesta completa. |
| Fuente explícitamente niega la obligación exacta para un actor/premisa ya fijado, mismo contexto completo | Fuera de alcance V1: combinación normativa negativa contextual unsupported, íntegra con deuda, sin nueva selección, corrección o refutación automática. No se retira el contexto para pasar a legacy. |
| Fuente aporta norma/contexto, pero control del actor subordinado o fuerza relacional pendiente | Candidato conservado, sin nueva selección automática por esta ruta V1. La coincidencia de dos `unresolved` no acredita compatibilidad. |
| Ruta/destino/actor/acción/ámbito distintos | Sin apoyo/corrección/contexto por esta relación. No constituye contradicción mundial. |
| Contexto fuente ausente o ilegible, o solo misma mención nominal | Deuda de contexto; conservar candidato, no respuesta completa ni selección contextual nueva. |

V1 no ofrece una ruta nueva de corrección/refutación normativa contextual: solo admite los owners normativos positivos descritos arriba y los pedidos con actor abierto de este contrato. Una fuente que niega esa obligación, incluso con actor, finalidad/control y ámbito inequívocos, queda fuera de alcance; lo mismo vale para la variante negativa de un patrón normativo impersonal. La combinación se conserva íntegra como raw/payload candidato con coverage/deuda unsupported de owner normativo no admitido y localización de la cláusula. No se la valida como proposición positiva contextual, no se publica un testigo refutador ni `answer_terms`, `answer_propositions` o bindings, y no se elimina la condición para rescatar una ruta histórica. En el interceptor corresponde a `pending` con motivo explícito de combinación unsupported; no a `answer_match` ni `partial_match`. Un error de esquema independiente sigue siendo invalid, no se oculta como esta limitación semántica.

`not(required A)` (negación de que A sea obligatorio) y `required(not A)` (obligación de no realizar A) son proposiciones deónticas distintas. Cambiar únicamente `atom.polarity` bajo `judgment:required` no representa la primera, ni autoriza traducir una a otra. Este contrato no añade un operador de negación deóntica para cubrir la brecha y no convierte una negativa personal en negación universal. Si el contrato entregado carece del carrier exacto, el texto y las unidades no representables permanecen unsupported con su deuda, sin una traducción positiva fabricada.

Las comprobaciones de incompatibilidad/conflicto que ya existan no se amplían aquí: dos cantidades distintas sin exclusividad y sin mismo fundamento/contexto tampoco bastan para refutar. Un camino histórico solo sigue siendo aplicable si ya expresa y transporta íntegramente la relación y la modalidad requeridas **sin** usar o ignorar el carrier V1; la presencia del carrier nuevo mantiene la reserva obligatoria. Las preguntas truth/premisa normativa negativa con actor fijado y una corrección contextual ejecutable requieren una futura capacidad explícita, crítica y verificación. Esta limitación reduce el alcance de V1; no cierra el objetivo general de corrección normativa ni acredita una mejora medida.

Los términos de respuesta solo salen de una proposición fuente completa con slot real y testigo íntegro. En las rutas parciales, `answer_terms`, `answer_propositions` y bindings de identidad/actor permanecen vacíos; toda cantidad o norma fuente se conserva como evidencia contextual etiquetada, nunca como respuesta implícita. Ninguna ruta convierte propósito en logro, condición en permiso, límite en cantidad exacta, ni duración de operación en autonomía.

Para magnitudes, una cantidad genérica sin contexto explícito no es una ley universal que pueda especializarse a cualquier ruta. Para patrones normativos, un filtro de clase fuente no demuestra pertenencia de un nombre candidato. Una coincidencia textual no descarga esas obligaciones.

## Sustitución privada del actor y control subordinado

El interceptor puede leer el slot real de una proposición fuente completa para construir una ligadura **tentativa y privada**, antes de publicar una respuesta. Este es el orden obligatorio: (1) validar ambos owners y dependencias; (2) comprobar acción, destinatario, destino, relación, ámbito y condiciones independientes del slot; (3) obtener `T` exclusivamente del argumento `actor` presente en el átomo fuente completo; (4) comprobar pertenencia de T al filtro pedido y correspondencia contextual mediante mapa local; (5) resolver la tabla siguiente; (6) revalidar testigo/contextos y solo entonces publicar T. No se toma T de `Reference.name`, del primer tutor, de una asociación ni de un patrón abierto sin actor.

`Same` exige identidad acreditada y ámbitos/guards compatibles; `Unknown` no satisface igualdad y `Difference` bloquea, incluso a través de aliases. La ligadura tentativa no rellena el raw ni los bindings públicos de una salida parcial.

| Participant pregunta | Participant fuente | Regla de control |
| --- | --- | --- |
| `reference:P` | `reference:U` | Exigir `Same(P,U)` dentro del mapa de ambos owners; Different bloquea, Unknown deja pendiente. |
| `owner_slot:actor` | `reference:U`, fuente completa con actor T | Sustituir privadamente el slot por T y exigir `Same(T,U)`. T=T o alias Same pasa; T=U con Difference o aliases contradictorios bloquea; identidad Unknown queda pendiente. |
| `reference:P` | `owner_slot:actor` de patrón fuente impersonal | No existe T. Para parcial exigir que el patrón normativo exprese la misma ligadura estructural sin identificar P con un actor anónimo; si P es individuo fijo, no hay prueba de que ese individuo sea el actor del patrón: pendiente, sin selección nueva. |
| `owner_slot:actor` | `owner_slot:actor` de patrón fuente impersonal | Correspondencia de roles abiertos y clase/relación/control idénticos; como máximo `partial_match`, sin T ni binding de identidad. |
| Cualquiera | `unresolved` o viceversa | `pending`, con reason y evidencia originales; dos unresolved no equivalen ni habilitan selección. |

Una pregunta con `Participant.reference:P` y una fuente con actor T y subordinado P puede responder T si las demás condiciones pasan: no se exige T=P cuando el texto no afirma ese control. En cambio, `owner_slot` pide expresamente el mismo actor del slot, y T acompaña mientras U se matricula no la satisface. Renombrar IDs y permutar tablas no cambia estas reglas.

## Salida parcial propia y elegibilidad global

`partial_match` es una transición operativa propia del interceptor hacia `requests[].partial`, no un permiso para llamar a una ruta legacy que desconoce el body. Requiere: ambos documentos válidos para el carrier, mismo anclaje y pertinencia documental demostrados, contexto entero compatible, ninguna ambigüedad o dependencia pendiente relevante ni conflicto. Puede quedar sin individuo/valor de respuesta por el diseño mismo de la representación. Produce un witness con kind nuevo `partial_request_context_v1`, `claim_ids` fuente originales y los qualifiers/scopes originales; el contenido nuevo se transporta en `context_transport` cerrado:

```text
ContextTransport = {
  kind:"normative_open_pattern"|"observed_same_attribute_extent_pending",
  source_owner:{kind:"claim",id:Id},
  question_owner:{kind:"request",id:Id},
  source_context_bindings:ContextBinding[],
  question_context_bindings:ContextBinding[],
  source_body:OriginalBody,
  question_target:OriginalPattern,
  reference_map:[{source:RefId,question:RefId}],
  unresolved_answer:"individual_not_supplied_by_pattern"|"requested_extent_not_established",
  requested_identity_inferred:false,
  requested_value_inferred:false
}
```

`ContextBinding` es la unión exacta ya definida; `OriginalBody` y `OriginalPattern` son copias íntegras de nodos ya validados de esos owners, no JSON libre elaborado durante matching. El mapa solo contiene correspondencias probadas de referencias conocidas, nunca el actor abierto. Todos los campos son obligatorios; arrays del mapa pueden estar vacíos si no hay referencias que mapear, pero los anclajes exigidos sí deben estar probados. El witness conserva además la evidencia de ambos owners/contextos sin inventar citas fuente para la pregunta. Este DTO nuevo de transporte requiere validar/copiar owner, scopes, qualifiers y evidence del witness actual; no se puede reemplazar por flags true/false sin payload.

En esta salida `answer_terms:[]`, `answer_propositions:[]`, `bindings:[]` y ningún término oculto en un campo singular de respuesta. `selected_claim_ids` puede incluir el claim de contexto parcial; la fila lleva `coverage:partial`, el par `complete_answer:false` y `requires_qualification:true`. `requires_review` conserva la política vigente para faltantes/conflictos, sin fingir que contexto parcial es una respuesta completa. La fuente íntegra se conserva y no se publica su cantidad observada como respuesta al máximo.

Una norma impersonal representada fielmente con actor abierto **por diseño**, clase, control y finalidad determinados puede tener `DocumentResult.status:complete`: no le falta un individuo que el texto nunca aportó. Igualmente, un Request con slot vacío bien expresado puede ser documentalmente completo. Esto no le da respuesta completa al par. Por separado, fuerza relacional ambigua o control perdido por extracción producen `partial`; `Participant{kind:unresolved,reason:unspecified}` fiel a un texto que omite control puede ser documentalmente completo pero la comparación queda `pending`, sin `partial_match`. Es importante no confundir completitud representacional con resolución del pedido. No se cambian dictámenes ni estados de las 116 entregas históricas mediante esta regla de diseño.

Para flat, `partial_match` publica el parcial útil local con los límites anteriores. Para envelope, se mantiene exactamente la elegibilidad global existente: ambos envelopes y todas las hojas deben validar `complete`, closure y ambigüedades deben cumplir las reglas históricas, y nunca se omiten hojas. Un patrón abierto íntegro puede satisfacer elegibilidad documental; una hoja partial por ambigüedad/pérdida no. La matriz conserva parciales y deudas de cada celda aun si el agregado queda `ineligible`. No se selecciona un subconjunto aceptado ni se cuentan parciales matriciales como apoyo agregado completo.

El agregado `complete_answer` requiere testigos completos de firma igual en **todas** las celdas y ausencia de conflictos, como hasta ahora. Si las celdas contienen parciales propios, el agregado conserva `conditional_reading` y `complete_answer:false`, con el payload íntegro en `evidence_by_reading`; no se incorpora una firma parcial a `request_signature` para convertirla en completa. Una futura selección agregada de contexto parcial invariante requeriría una capacidad/contrato adicional: este V1 permite observar y seleccionar parcialmente en flat y conserva evidencia por celda, sin afirmar esa nueva selección a nivel envelope.

## Firmas, estabilidad y conflictos

Con perfil V1, estabilidad del inventario de pedidos exige además de IDs/orden la igualdad semántica canonicalizada de modo, target, slot, extent, class_filter, unit, fundamento, ámbito, qualifiers, premisas y `context_bindings` de cada Request entre hojas. Si se pretende que distintas lecturas cambien precisamente el contexto de la misma pregunta, se conserva el inventario pero se marca inestable para respuesta invariante; no se fuerza igualdad borrando ese contexto.

La firma completa por witness incluye (a) firma actual íntegra; (b) context_bindings del request y de **cada claim owner** usado, incluso en el caso `generic_constraint`; (c) cuerpos originales de esos owners; (d) testigo de control, sustitución privada del actor, mapa de referencias entre ambos owners, scopes/binders/ramas y deudas; (e) modalidad y relación de utilidad. La excepción histórica que abstrae fundamento de una restricción genérica nunca puede abstraer el contexto nuevo. Ausencia de testigo o canonicalización no demostrada impide firma completa, no produce una firma vacía igual.

Canonicalización V1: quitar únicamente IDs locales de owner, contexto y qualifier como nombres accidentales; resolverlos antes a su contenido y pertenencia. Reemplazar RefId en contexto, actor, mapa, ámbito y payload por claves ancladas verificadas de su documento/namespace, conservando denotación, clase, tipo y relaciones restrictivas. Mantener como símbolos distintos los roles abiertos (`owner_slot:actor`) y `unresolved` con su reason. Sustitución y control usan pares de claves, nunca T/U locales. Las claves de alias se derivan de componentes Same verificados y se invalidan ante Difference/incoherencia; no se elige arbitrariamente el primer ID como clave. Los ciclos y referencias sin clave conservadora consumen presupuesto y dejan la firma pendiente, sin fabricar igualdad.

Los arrays que son conjuntos de contexto/mapa/qualifiers se ordenan por contenido canonicalizado y conservan multiplicidad indebida como defecto, no se deduplican para rescatar entradas. El orden significativo de scopes cuantificados y argumentos ligados se conserva. Citas, offsets y IDs originales permanecen en el witness de procedencia; no se confunden con igualdad semántica. Las claves de identidad existentes pueden incluir hash de texto y anclaje documental conservador: si no acreditan igualdad entre hojas, V1 no los elimina para obtener un pase. El objetivo es invariancia bajo renombrado coherente de IDs, no equivalencia libre entre documentos distintos.

La clave de grupo de conflicto incluye target y contextos canonicalizados de ambos owners y mapa/control. Contextos A y B distintos no son ni el mismo apoyo ni una contradicción entre cantidades; se conservan por separado y no generan una respuesta invariante. Para el mismo contexto sí se aplican conflictos de cantidad/modalidad actuales antes de publicar. Retirar/intercambiar una condición cambia firma, testigo y elegibilidad; nunca solo la cita. Esta canonicalización ampliada es capacidad nueva pendiente de implementación; la función `request_signature` actual no la proporciona.

## Presupuesto global V1 y preflight

Los siguientes límites son propuestos para operaciones con perfil V1, sin modificar presupuestos ni bytes legacy cuando falta el perfil. Se aplica el mínimo entre el límite vigente de la entrada y el V1: bytes de entrada completos ≤16 MiB; profundidad JSON y recursiva ≤64; ≤100000 nodos JSON en total sobre ambos lados y todas las hojas (incluidos contextos, evidencia y envelopes). Los DTO nuevos no quedan fuera de `MAX_NODES`. Por array Evidence se admiten como máximo 32 entradas, y en toda la operación como máximo 4096 entradas Evidence. Hay como máximo 512 owners claim/request en total sumando las hojas; no se reinicia por hoja.

El producto conserva hasta ocho lecturas por lado y 64 celdas. Flat cuenta como una hoja. Antes de expandir celdas, preflight calcula con enteros comprobados: número real de hojas, owners, nodos, bytes de todas las hojas ya presentes y `sum(source_claims_in_leaf * question_requests_in_leaf)` de las celdas, con techo 65536 comparaciones owner/request. No materializa clones para contarlos. Overflow, hoja extra, owner/evidence de más o producto excedido son agotamiento conservado; nunca se recortan hojas ni contextos. El preflight incluye owners sin contextos si participarán en una celda V1, evitando esconder trabajo en fallback.

Un único `ContextBudget` de la operación externa gobierna validación, clausuras de aliases/dependencias, evaluación de todas las celdas, testigos, firmas y serialización canonicalizada. Límite adicional: 1000000 pasos visitados y 16 MiB acumulados de representación canonicalizada/testigos nuevos materializados. Cada visita a nodo, arista, pareja de referencias/roles, candidato owner/request y celda debita un paso **antes** de expandir; cada clon/materialización debita sus bytes calculados antes de reservar. Una llamada repetida vuelve a consumir: no crea un presupuesto local nuevo. Un cache hit consume un paso y conserva resultados solo si pertenecen al mismo documento/owner/mapa/perfil; la caché no es una forma de omitir chequeos de scope/deuda. La serialización no expande recursivamente el grafo otra vez sin debitarlo.

La API pública flat crea un presupuesto una vez; el wrapper envelope lo crea una vez y pasa la misma instancia a validadores y a la entrada contextual interna de cada celda. Los wrappers no pueden invocar repetidamente una entrada pública que lo reinicie. Los límites actuales locales de canonical/reference_key no bastan: deben someterse también a ese presupuesto global al activar V1. Si esto requiere cambiar firmas internas para pasar el contador, se declara parte necesaria de implementación, no una propiedad actual.

Agotamiento en cualquier fase deja la operación nueva no elegible para selección, con motivo técnico de límite, candidatos/payload original conservados y `complete_answer:false`. No hay fallback legacy, firma vacía, partial_match por timeout ni respuesta de las primeras celdas. Un error de bytes/lectura se informa por el adaptador conservando la referencia/hash del artefacto recibido; no se representa como documento vacío. Estos números acotan trabajo lógico y materialización controlada, no prometen un límite físico exacto de RAM ni latencia de producción.

## Ejemplos abstractos y controles

Los JSON siguientes son fragmentos de los DTO nuevos, no documentos CLI completos. Los offsets solo se ofrecen contra sus propios textos de ejemplo; los documentos completos deben añadir referencias, owner, scopes, hashes y las evidencias del target de manera coherente.

Texto ASCII `during route A`, bytes 0..14:

```json
{"id":"ctx1","kind":"nominal_during","anchor":{"kind":"ref","id":"route_a"},"evidence":[{"start_byte":0,"end_byte":14,"quote":"during route A"}]}
```

Positivo de conservación: quantity/autonomy de dispositivo D, slot amount ausente, máximo literal intacto, y ese contexto local. Positivo de comparación: una fuente sintética independiente expresa explícitamente la misma autonomía máxima de D durante A y un valor, con igual fundamento y condiciones. Puede responder con su valor contextualizado. Negativos: fuente de D durante B; duration de operar D durante A; dato observado de D durante A presentado como máximo; ruta A ligada solo a otra cantidad del mismo claim. Ninguno se rescata eliminando el contexto.

Texto ASCII `for enrollment at course B`, bytes 0..26; el verbo nominal ocupa bytes 4..14:

```json
{"id":"ctx2","kind":"normative_action_relation","relation":{"kind":"unresolved","alternatives":["purpose","necessary_condition"],"evidence":[{"start_byte":0,"end_byte":26,"quote":"for enrollment at course B"}]},"action":{"action_kind":{"kind":"symbol","id":"enroll","evidence":[{"start_byte":4,"end_byte":14,"quote":"enrollment"}]},"actor":{"kind":"unresolved","reason":"ambiguous_control","evidence":[{"start_byte":0,"end_byte":26,"quote":"for enrollment at course B"}]},"destination":{"kind":"ref","id":"course_b"},"evidence":[{"start_byte":0,"end_byte":26,"quote":"for enrollment at course B"}]},"evidence":[{"start_byte":0,"end_byte":26,"quote":"for enrollment at course B"}]}
```

Positivo de conservación: pedido de actor de acompañamiento requerido a persona P con filtro tutor y ámbito B; actor subordinado y fuerza sin resolver quedan pendientes, sin crear claims de matrícula. Para un positivo operativo distinto, un texto sintético debe expresar inequívocamente finalidad y que P es quien se matricula; entonces se permite `relation.kind:purpose` y `Participant.reference:P`, con evidencia de ese control. Una fuente sintética con la misma ligadura y tutor T nombrado puede responder T si acredita su clase; una norma impersonal compatible solo aporta contexto.

Negativos: poner `owner_slot` solo porque se pregunta un tutor; elegir a P como actor subordinado sin control textual; crear estudiante genérico; afirmar matrícula `occurred`; sustituir finalidad por `guard`; ligar el acompañamiento a otra persona o curso; usar norma necesaria como si fuera suficiente; tratar un ámbito normativo igual como prueba de finalidad. Todos conservan el defecto original y no deben pasar como completos.

Matriz mínima futura, con ambos documentos completos y validados antes de afirmar un rechazo semántico:

1. Perfil ausente: replay histórico idéntico; extensión presente sin perfil rechazada. Perfil feature desconocida/duplicada, dependencia de feature ausente, extras, null y array vacío rechazados.
2. Nominal: dos contextos simultáneos conservados conjuntivamente como unsupported sin división en lecturas; positivo valor exacto compatible; contexto ausente/distinto; homónimo; alias Same y Difference; referencia con scope hermano; contexto de otro owner; dos magnitudes sin transferencia; máxima conservada; observación frente a máximo; autonomía frente a duración operativa.
3. Normativa: apoyo de actor explícito con clase acreditada; patrón impersonal parcial; clase no acreditada; actor/destinatario/acción/destino/ámbito/fundamento distintos; fuerza y control pendientes por separado; necesidad frente a suficiencia; negación contextual de obligación conservada unsupported sin fallback ni refutación nueva; distinción `not(required A)`/`required(not A)` sin traducción por polarity; negación personal sin universalización; reificación y ocurrido inventados.
4. Integridad: UTF-8, cita truncada o repetida mal localizada, ID externo/desconocido, rol duplicado, dependencias cíclicas/transitivas, alias contradictorios, guard pendiente, preservación de hechos independientes, orden de claims/contextos y renombrado de IDs.
5. No bypass y dispatch: probar cada fila de la tabla pública en flat y todas las combinaciones con envelope, perfil ausente/presente, validación y comparación; firmas iguales bajo renombrado de IDs y diferentes al retirar/intercambiar contexto; T=T, T/U, alias Same/Difference y patrón abierto; agotamiento preflight y global sin expansión repetida favorable; cada nueva deuda reserva la ruta antes de fallback; términos/bindings vacíos en parcial; pregunta modificada no se etiqueta como misma entrada; reportes antiguos con sus inputs íntegros siguen idénticos.

## Implementación posterior y límites

1. Someter este contrato a crítica separada de fidelidad, ambigüedad y contraejemplos, incluida la necesidad del body fuente impersonal. Congelar el diseño revisado antes de cambiar esquema o guía. No reutilizar críticas de otro contrato como aprobación de este.
2. Implementar validación optativa y grafo de operandos en `tools/evaluation`, DTO cerrado, límites y retención de deudas. No cambiar `Atom`, guards, `route`, `context`, acciones fijas ni semántica de producto. Las decisiones futuras de estructura de módulos requieren leer las reglas correspondientes antes de editar.
3. Implementar comparación y testigos primero para cantidades con contexto explícito y normas con actor explícito; añadir el operador impersonal solo con feature propia y sus controles. Ni una fase solo de validación ni un fallback que lo ignore son la implementación completa de utilidad definida aquí.
4. Preparar controles sintéticos nuevos independientes de las fuentes no consultadas. Separar validación estructural, conservación semántica, comparación y ejecución CLI. Los positivos no pueden ser solo informes construidos a mano o fixtures inválidos.
5. Ejecutar verificación proporcional autorizada y replays históricos desde entradas originales; registrar deltas de vocabulario/frame por separado y conservar fallos. Las campañas largas requieren su alcance/duración y autorización; este documento no las ejecuta ni las autoriza.
6. Preparar cualquier extracción siguiente en una versión y paquetes nuevos fuente-only y question-only, con hashes y doble crítica. Este diseño retrospectivo conocido no acredita fidelidad autónoma ni habilita medición de utilidad automática antes de las puertas vigentes.

Quedan fuera: cambios en base de datos o producto, API remota, corpus, métricas/gates, completar ambigüedades desde contrapartes, razonamiento general de planes/roles, equivalencias de magnitudes, transporte histórico y nuevas fuentes. La capacidad de representar todas las formulaciones naturales y de extraer fielmente estos DTO sigue sin comprobarse. Este documento aporta un contrato criticable y un camino de implementación; no es una mejora medida.
