# Composición documental de medición negada y «hasta»

Fecha: 2026-10-22. Estado: propuesta semántica propia, sin implementación ni decisión de adopción. Material exclusivo:

- «No se midió la duración del reproductor Lino hasta apagarse.»
- «Cobre funcionó ochenta minutos en Nacar sin apagarse; no se midió su duración hasta apagarse.»

Este documento no evalúa extracción, respuestas, rendimiento ni calidad esperada. No usa material de preguntas ni resultados para elegir la representación.

## 1. Decisión y compromisos positivos

Conservar la cláusula de medición mediante una **composición tipada con adjunción interna abierta**. El núcleo es una medición con «se» y agente no expresado, el objeto es una descripción de duración, la negación se adscribe al núcleo de medición y «hasta» relaciona un sitio interno todavía no resuelto con un predicado de límite «apagarse». La relación de alcance entre ese límite y la negación tampoco se descarga mientras el sitio siga abierto. La composición completa, y no la negación separada, constituye la unidad afirmada por el documento.

Esto conserva información positiva: hay una construcción negativa de medición de una duración, existe una relación léxica de límite expresada por «hasta», su complemento tiene contenido predicativo de apagado y la construcción pertenece a esa incidencia de cláusula. En Lino, la duración tiene propietario explícito: el reproductor Lino. En la segunda cláusula de Cobre, el propietario se expresa mediante el posesivo «su» y permanece sin resolver. El hecho de ser una mención definida no acredita que el autor del frame conozca su antecedente.

El límite es un predicado dentro de la construcción; no es una ocurrencia de apagado afirmada. «Hasta» no introduce por sí solo medición posterior, duración numérica, comienzo del intervalo, inclusión exacta del extremo ni causa. Tampoco convierte la cláusula en una afirmación global de ausencia de toda medición bajo cualquier circunstancia.

La primera cláusula de Cobre aporta por separado funcionamiento durante ochenta minutos, el complemento local «en Nacar» y ausencia de apagado durante ese funcionamiento. Sus tres componentes conservan la misma incidencia de funcionamiento. Ni ese intervalo ni ese complemento se transfieren a la cláusula de medición por compartir documento o por estar unidos por punto y coma. Puede conservarse la fuente completa para ver ambos en su orden original.

## 2. Qué queda abierto y por qué

El material aislado deja abierta una diferencia interpretativa relevante: una paráfrasis niega la medición antes del límite de apagado; otra niega la medición de la duración delimitada por apagado. Haber medido otra duración antes del límite puede ser compatible con la segunda y contradecir la primera cuando se trate de la misma duración. Es un contraste de condiciones de interpretación, no prueba de que el texto afirme ambas. La admisibilidad de esas adjunciones en estas fuentes es juicio lingüístico local, con distinta naturalidad, no una lista exhaustiva establecida por una regla gramatical general. La lectura temporal de una negación con «hasta» puede, además, sugerir un cambio posterior que aquí no está afirmado de manera independiente. La prosodia, una formulación más explícita o contexto discursivo adicional podrían resolver parte de esto. No se conoce un conjunto exhaustivo de lecturas admisibles de estas frases y no se declara tal conjunto.

La falta de una regla de extracción no crea la ambigüedad lingüística; una regla que obligue a escoger siempre una adjunción solo ocultaría esa incertidumbre. A la inversa, no toda diferencia imaginable de frames constituye una lectura respaldada por el texto. El carrier propuesto conserva la construcción con una variable de adjunción tipada, sin convertir posibilidades analíticas en alternativas afirmadas ni en una disyunción enumerada.

Su semántica es una expresión composicional abierta: `Until(a, SwitchOff(s))` modifica un sitio `a` aún no fijado de la región cuyo esqueleto es `Not(Measure(Duration(p)))`. `p` es un propietario explícito o una mención posesiva abierta; `s` es el sujeto implícito del predicado de límite. Se denomina `c0` a ese esqueleto rígido: la resolución no puede cambiar su predicado, objeto, propietario ni adscripción de «no». La unidad interpretada es `Compose(c0, Until(a, SwitchOff(s)))`. El esqueleto muestra la adscripción de «no» a «medir», pero **no autoriza evaluar `Not(Measure(Duration(p)))` después de quitar el operador abierto**. Elegir `a` también determina cómo se compone el alcance del límite respecto de esa negación. La estructura abierta no es una nueva negación global ni una semántica temporal completa.

La variable `a` es una posición semántica de adjunción abierta dentro de esta región, sujeta a resolución posterior acreditada; no es un cuantificador existencial ni un permiso para que el consumidor escoja un sitio. Su dominio de tipos comprende posiciones de composición de la predicación y de la descripción de cantidad, con la interacción de alcance correspondiente; excluye el propietario como entidad, otros Claims, evidencias, referencias documentales y el ID del operador. Esta versión no almacena una whitelist de lecturas ni interpreta la colección de IDs como dominio semántico. El consumidor que necesita un sitio concreto debe conservar la deuda, no escogerlo por conveniencia. La tipificación de la región limita dónde podría resolverse el adjunto; no demuestra una resolución.

El sujeto de «apagarse» no está expreso en ninguno de los dos complementos. Puede conservarse como argumento implícito abierto del predicado, con la morfología reflexiva original. No se lo iguala automáticamente al agente de medir, al propietario de la duración o al sujeto de una cláusula anterior. Esto no niega la plausibilidad de una lectura correferente: evita convertirla en un vínculo acreditado sin especificar su fundamento. Lino conserva en todo caso la diferencia esencial: su propietario de duración sí es explícito.

## 3. Diferencia con el carrier disponible

El contrato local admite un límite relativo de no ocurrencia para un evento de medición negado, con sujeto del límite igual al de la medición. Ese carrier ya elige una adjunción y una correlación de sujetos. Es apropiado cuando ambas estén acreditadas; usarlo por defecto para estas dos frases resolvería información que este diseño mantiene abierta. Tampoco procede elegir directamente un atributo de duración hasta apagado.

El contexto léxico de cláusula aporta una idea reutilizable: propiedad estructural por incidencia y corte de consumidores antes de retirar el contexto. Su variante de preposición más complemento referencial no expresa un complemento predicativo «apagarse» ni la interacción con la negación. No se fuerza esta construcción dentro de esa variante.

## 4. Extensión mínima propuesta y wire cerrado

Se propone **una variante nueva de `Claim.body`**, `linguistic_composition`, para esta primera familia declarada. No se añaden eventos ocurridos, predicados con la frase entera embebida, otro documento ni un `source_says` opaco. El árbol expone medición, objeto, propietario, negación, límite y sujeto abierto por separado. La expresión no es una conjunción `H ∧ condición`: quitar `Until` de `NotMeasure` no conserva necesariamente su verdad. Por eso este contrato propuesto elige un Body que no publique `c0` como proposición ejecutable separada. Extender contexto sería una alternativa suficiente únicamente si cambiase la unidad afirmada de forma inseparable en todos los consumidores y dejase de exponer un núcleo ejecutable. No se incorporan ambos mecanismos. Esta es una decisión del contrato propuesto, no una afirmación de haber auditado todas las rutas actuales ni una comparación de costes ya verificada.

La gramática siguiente define exactamente la variante propuesta; no describe una forma aceptada actualmente. `ID` y `Evidence` mantienen el contrato V3. `Evidence` tiene exactamente `start_byte`, `end_byte`, `quote`, con fin exclusivo y coincidencia UTF-8. `RefTerm` es exactamente `{kind:"ref",id:ID}` y apunta a una referencia documental existente, con todo su scope y sus restricciones. Todos los objetos son cerrados, las listas tienen el número de elementos indicado y no hay campos opcionales implícitos.

```text
LinguisticComposition = {
  kind: "linguistic_composition",
  construction: "negative_se_measurement_with_until_v1",
  clause_evidence: Evidence,
  core: {
    id: ID,
    kind: "negation",
    evidence: Evidence,
    operand: {
      id: ID,
      kind: "measurement_predicate",
      predicate: "measure",
      head_evidence: Evidence,
      voice: {kind:"nonagentive_se", evidence:Evidence},
      object: {
        id: ID,
        kind: "quantity_description",
        attribute: "duration",
        head_evidence: Evidence,
        owner: ExplicitOwner | OpenPossessiveOwner
      }
    }
  },
  operators: [{
    id: ID,
    kind: "lexical_temporal_boundary",
    relation: "until",
    connective_evidence: Evidence,
    phrase_evidence: Evidence,
    attachment: {
      kind: "source_underspecified",
      region_root_id: ID,
      resolution: null
    },
    boundary: {
      id: ID,
      kind: "event_predicate",
      predicate: "switch_off",
      head_evidence: Evidence,
      form: "reflexive_infinitive",
      subject: {
        id: ID,
        kind: "implicit_argument",
        semantic_type: "entity",
        realization_evidence: Evidence,
        resolution: null
      }
    }
  }]
}

ExplicitOwner = {
  kind: "explicit_owner",
  relation: "duration_bearer",
  term: RefTerm,
  relation_evidence: Evidence,
  mention_evidence: Evidence
}

OpenPossessiveOwner = {
  id: ID,
  kind: "possessive_argument",
  semantic_type: "entity",
  grammatical_person: "third",
  possessed_head: "duration",
  determiner_evidence: Evidence,
  resolution: null
}
```

`core` y `operators` son partes de una sola expresión, no claims independientes. `region_root_id` debe ser exactamente el ID de `core`; no puede apuntar a otra cláusula, a un alias ni a un evento externo. Los IDs internos son únicos dentro del Claim, no son IDs de referencias V3 y no tienen existencia independiente. La incidencia es `(documento, claim_id, id_interno)`. El operador tiene dueño por contención, sin arista inversa ni copia de todo el Claim. La negación tiene como operando exactamente la medición y esta tiene como objeto exactamente la descripción de duración.

Las evidencias de negación, voz, cabezas, propietario y operador están dentro de `clause_evidence`. El orden de sus subintervalos debe corresponder al orden textual. `phrase_evidence` contiene el conector y el predicado de límite. El propietario explícito necesita evidencia de la relación y de la mención; el posesivo abierto necesita evidencia del determinante en la misma incidencia que la cabeza de duración. No se acepta una palabra igual tomada de otra aparición. La contención de bytes es condición estructural necesaria, no prueba suficiente del análisis semántico.

El perfil opcional debe registrar la feature `linguistic_composition_until_measurement_v1`; ausencia conserva el contrato anterior y rechaza esta forma. Los tipos de nodo y firmas son internos del AST versionado: `measurement_predicate.object:quantity_description`, `event_predicate.subject:implicit_argument(entity)` y `quantity_description.owner:ExplicitOwner|OpenPossessiveOwner`, con atributo fijo `duration`. `quantity_description` es una descripción expresiva interna, nunca una cantidad medida, valor numérico, referencia V3 ni entidad existencial. Los literales `measure` y `switch_off` son discriminantes semánticos internos de esta variante y no filas nuevas de `predicate_rows`; no alteran firmas ni clases globales de término. `duration` debe existir como atributo declarado en el vocabulario. La feature y el vocabulario íntegro forman parte del hash; la definición versionada fija los tipos internos. El experimentador omitido y el propietario son roles distintos. `nonagentive_se` significa exclusivamente que la fuente contiene «se» y no expresa agente: no decide impersonalidad frente a pasiva refleja ni sujeto gramatical frente a complemento directo. `object` nombra el rol semántico de magnitud medida.

El vocabulario semántico de la nueva forma es pequeño y fijo: negación, medición, descripción de cantidad, propietario explícito/posesivo, límite y argumento implícito. `measure` y `switch_off` representan predicados, no hechos ocurridos por aparecer como nodos. Solo la expresión completa recibe la aserción documental del Claim. La variante no admite cuantificadores, varios operadores, negación anidada ni resolución interna en esta primera versión; esos materiales requieren otra versión o permanecen no soportados, nunca se simplifican.

## 5. Instancias y anclas de las dos fuentes

En Lino, `owner` es `explicit_owner`, `term` apunta al referente documental del reproductor Lino y la relación es portador de la duración. `attachment.resolution` y `boundary.subject.resolution` son `null`. En Cobre, la segunda cláusula usa `possessive_argument`: no incorpora `term:Cobre`, un candidato preferido ni una arista Same. La mención «su» es el argumento contextual abierto; no se crea una entidad física anónima afirmada para rellenarlo.

Las siguientes anclas exactas permiten instanciar la gramática sin una frase opaca como operando. Los offsets son UTF-8 desde cero sobre cada fuente exacta, incluidas sus tildes. La puntuación final queda fuera de `clause_evidence`.

| Campo de evidencia | Lino: `[inicio,fin)` y cita | Cobre: `[inicio,fin)` y cita |
| --- | --- | --- |
| cláusula | `[0,61)` «No se midió la duración del reproductor Lino hasta apagarse» | `[55,95)` «no se midió su duración hasta apagarse» |
| negación | `[0,2)` «No» | `[55,57)` «no» |
| voz | `[3,5)` «se» | `[58,60)` «se» |
| medición | `[6,12)` «midió» | `[61,67)` «midió» |
| duración | `[16,25)` «duración» | `[71,80)` «duración» |
| relación de propietario | `[26,29)` «del» | no existe este campo en la variante posesiva |
| mención explícita | `[30,46)` «reproductor Lino» | no existe este campo en la variante posesiva |
| posesivo | no existe este campo en la variante explícita | `[68,70)` «su» |
| conector | `[47,52)` «hasta» | `[81,86)` «hasta» |
| sintagma de límite | `[47,61)` «hasta apagarse» | `[81,95)` «hasta apagarse» |
| predicado/sujeto implícito | `[53,61)` «apagarse» | `[87,95)` «apagarse» |

Usar la misma evidencia para la cabeza del límite y la realización de su argumento implícito conserva la morfología, sin fingir una mención nominal independiente. Ninguna ancla del primer segmento de Cobre sirve para resolver automáticamente ese argumento o el posesivo. La falta de resolución es parte de la estructura representada, no una pérdida de la palabra «su».

## 6. Consumo parcial útil y cierre

La composición puede estar íntegramente representada y seguir teniendo adjunción, alcance interno y correferencia sin resolver. Esas deudas bloquean su ejecución como negación global, dato numérico, prueba de apagado o prueba de medición posterior. No invalidan por sí mismas la conservación de la construcción como contexto documental.

Se reutiliza exclusivamente el transporte `classify-documentary-context-v2`; no se propone otra CLI, manifiesto, selector de intents ni protocolo de revisión. La variante fuente queda en el Body original del Claim transportado. La regla siguiente añade selección parcial a ese transporte conservando sus envolturas y sus deudas; no redefine los DTO de request ni de salida.

**Regla positiva cerrada de selección por portador.** Son requisitos conjuntos:

1. Feature válida `linguistic_composition_until_measurement_v1` y composición fuente válida en el documento fuente. `owner.kind` debe ser `explicit_owner`; posesivo o sujeto implícito no satisfacen este requisito.
2. Request V3 existente, válido y elegible, de modo `answer`, slot `amount`, predicado `quantity`, atributo exacto `duration`, con `subject` referencial y sin valor `amount` ya presente. El juicio pedido es `fact`, estado `state`; no se admite aquí un request de evento, verdad, planificación, capacidad o permiso. El operador `one_value` no elimina la deuda del slot: esta ruta nunca lo completa.
3. El `subject` del request y `owner.term` deben corresponder por `Same` acreditado bajo el namespace y ámbito efectivos; no basta compatibilidad descriptiva, igualdad de cadenas o nombre cercano. Cada vínculo restrictivo y todos los aliases pertinentes de ambos extremos deben quedar acreditados y compatibles. Unknown conserva candidato pendiente, nunca habilita selección.
4. Se conservan y comprueban completos configuración, scopes, calificadores, dependencias y guards de fuente y request. No se elimina un filtro para seleccionar. Una configuración ausente o desconocida frente a una requerida, una condición ajena, conflicto o deuda externa conserva candidato pendiente o rechazo según el contrato V2; no publica un parcial seleccionado. Los campos ausentes no son wildcards.
5. La selección no exige resolver adjunción, sujeto del límite ni episodio de medición: esas tres deudas son el contenido abierto que la ruta conserva. Las deudas de identidad/slot/fundamento que V2 mantenga no se descargan mediante esta selección. No se infiere `actual_assignment`, resultado de medición ni realización de apagado.

El resultado es únicamente `partial_relevance` en la salida existente V2, con `answer_terms:[]`, `answer_propositions:[]` y `bindings:[]`. Transporta el Claim completo con el mismo ID, `sourceBody` igual a la composición íntegra, evidencia y cierre efectivos originales, además del request íntegro y todas las deudas. `sourceBody` describe el contenido conservado en la envoltura V2, no exige renombrar un campo histórico. La selección acredita pertinencia contextual de duración para ese portador; no acredita cifra, verdad de `c0`, sujeto del apagado ni una respuesta completa.

Ejemplo genérico de la regla, sin introducir otra pregunta: si el término fuente explícito `r_source` y el sujeto del request `r_request` tienen prueba `Same(r_source,r_request)`, el atributo de ambos es exactamente `duration` y pasa el contexto efectivo completo, se conserva la composición fuente como contexto parcial. Sustituir `explicit_owner` por un `possessive_argument` con `resolution:null` impide esa selección aunque haya un referente cercano de igual nombre. Lino puede satisfacer la regla mediante su propietario explícito cuando exista esa prueba; no se declara que cualquier request concreto la satisfaga.

No hay equiparación automática con `autonomy` ni con otro atributo. Una relevancia cruzada solo podría operar mediante el perfil de relevancia ya declarado existente, con bindings y carry_roles acreditados completos y todas sus restricciones, sin añadir aquí tal fila ni inventar una cantidad. Fuera de la regla exacta anterior esta propuesta no habilita otra selección autónoma.

El posesivo de Cobre puede conservarse al transportar el documento o su cláusula completos, mostrando el argumento abierto. Esa conservación documental no se etiqueta como selección acreditada de duración para Cobre por esta regla. Ningún argumento pendiente se resuelve por proximidad.

El corte antecede a matching ordinario, proyecciones y cualquier eliminación de contexto. Incluye los consumidores efectivos explícitos/implícitos y aliases del owner; ningún dependiente puede usar el esqueleto negado como si el operador no existiera. Los hechos independientes, como el funcionamiento expresado en la primera cláusula de Cobre, permanecen disponibles con su contexto propio. No se unen ambas incidencias por la proximidad textual.

Canonicalizar conserva adscripción de negación, objeto, tipo de propietario, evidencia posesiva, variables abiertas y pertenencia del límite. Renombrar coherentemente IDs internos no cambia significado; sustituir el posesivo por Cobre, igualar el sujeto implícito a Lino o mover el límite a otra incidencia sí lo cambia. El cierre y las copias se cobran al presupuesto compartido vigente; no se abre un presupuesto por operador ni se devuelve una estructura truncada cuando se agota.

## 7. Contramodelos y límites de inferencia

Estas situaciones muestran conclusiones que la representación no debe imponer; no constituyen una enumeración de lecturas ni una batería de evaluación:

- Puede no haberse realizado una medición de esa duración y tampoco haber ocurrido un apagado observado. Conservar el límite predicativo es compatible con ello; afirmar un evento de apagado no lo sería.
- Puede haberse medido una magnitud o una duración con otra delimitación, sin que se haya medido la duración mencionada en la cláusula. La composición no permite negar todas las mediciones del aparato.
- Un relato con contexto adicional podría acreditar una medición posterior. La frase aislada no la afirma; el operador no crea ese evento ni su valor.
- Una duración de funcionamiento sin apagado durante ochenta minutos puede coexistir con duración total hasta apagado desconocida. Los ochenta minutos no completan la magnitud total ni cuantifican automáticamente la cláusula de medición.
- Un contexto anterior puede proporcionar otro antecedente para «su». El hecho de que Cobre sea la mención nominal cercana no constituye por sí solo la acreditación documental exigida para resolverlo. Una resolución posterior válida debe conservar la mención y añadir su prueba, no borrarla.
- Dos apariciones de «hasta apagarse» con palabras iguales pueden pertenecer a mediciones distintas. Igualdad léxica no demuestra identidad de incidencia ni habilita intercambio de límites.

No se concluye un límite inferior numérico de duración total para la segunda cláusula a partir de la primera sin probar continuidad, mismo portador y relación entre el funcionamiento y esa duración. Tampoco se niega esa relación: permanece sin acreditar.

## 8. Coste y decisión previa a cualquier implementación

El coste mínimo fiel incluye una variante nueva de Body, una declaración optativa versionada y su admisión/hash, tipificación de los operandos internos, cierre/canonicalización por incidencia, integración de la regla parcial en `classify-documentary-context-v2` y corte de todos los consumidores que podrían omitirla. El transporte existente evita añadir CLI, manifiesto o endpoints; no evita revisar su admisión y publicación de esta nueva variante. El carrier posesivo está anidado en la misma composición: no requiere introducir una referencia física o un resolvedor de anáforas general. No se necesita motor temporal, enumeración de lecturas ni elección automática de antecedente.

El coste no se reduce a admitir una clave JSON: habrá que revisar todos los puntos que ejecutan o proyectan Claim.body. Si esa revisión no puede garantizar rechazo/corte íntegro, la forma debe mantenerse fuera del recorrido ejecutable y conservarse como propuesta; no degradarse a un evento negativo ordinario. Este documento propone esa frontera antes de código y no modifica contratos operativos, fuentes históricas ni mecanismos existentes.
