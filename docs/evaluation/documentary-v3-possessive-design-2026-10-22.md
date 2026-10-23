# Diseño prospectivo: posesivo nominal y referente indeterminado

Estado: propuesta semántica y operativa local, todavía sin implementar ni aceptar por doble crítica. No modifica el motor, vocabularios, entregas, puntuaciones o revisiones históricas. No se ejecutaron Rust, Cargo, APIs, clasificación ni campañas. La aceptación de este diseño no acreditaría extracción autónoma ni mejora medida de recuperación.

## Alcance y evidencia de diseño

Fuente única: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.» Se consultaron el contrato local V3, reglas backend/testing y únicamente los tres archivos de autoría `credential-term-correction-v1/author/{candidate-input.json,rationale.md,receipt.json}`. No se consultaron pregunta de Faro, contrapartes, gold, guía V6 ni otras revisiones. El contrato contiene enlaces y resúmenes históricos; estos no se usan como evidencia de fidelidad de esta fuente. La candidata anterior sigue siendo una hipótesis parcial no aceptada.

El problema expresivo es general: una construcción posesiva nominal relaciona el núcleo y un determinante posesivo, pero no siempre identifica al referente de ese determinante. `owned_by` añade una relación distinta, cuya fuerza no se redefine. `associated_with` pierde la construcción; guardar solo «su» pierde la restricción sobre el núcleo. Asignar Ina como referente o crear una persona anónima también excede lo demostrado.

La propuesta conserva una relación posesiva **lingüística** y su argumento referencial abierto. Su semántica no afirma propiedad física o jurídica, tenencia, emisión, recepción, uso efectivo, pertenencia institucional, existencia de un poseedor concreto ni una clase universal. No determina el género, número o identidad de ese referente desde la concordancia del posesivo. El texto puede sostener una condición nominal suficiente sin afirmar que exista actualmente una credencial física.

## Carrier recomendado y compatibilidad

Se añade el feature optativo `nominal_possessive` a `request_context_profile.version=documentary_request_context_v1`, un kind de enlace del mismo nombre y la variante de denotación `lexical_use`. No se añaden campos a `Reference`, `ReferenceLink`, `Evidence` o `Dependency`, ni se inventa una entidad para el referente abierto. Se extienden dos enumeraciones cerradas: no es aceptable afirmar que los DTO actuales ya aceptan estos valores. Es una ampliación prospectiva del contrato JSON con el mismo shape y admisión expresa por feature; no es compatible con un lector antiguo que rechace variantes desconocidas.

Conservar `denotation:entity` y reinterpretarla por incidencia evitaría una variante del DTO, pero dejaría un uso léxico disponible como entidad para consumidores antiguos. Se descarta ese atajo. Añadir `lexical_use` hace explícita la diferencia en el dato y obliga a revisar los matches exhaustivos. El opt-in no autoriza otros usos de esa denotación: en esta primera forma solo aparece como target de exactamente un enlace posesivo válido. Una extensión posterior puede definir nuevos usos léxicos por forma, sin reutilizar este carrier para significados diferentes.

Esquema exacto de la extensión:

```text
request_context_profile.features += "nominal_possessive"
Reference.denotation += "lexical_use"
ReferenceLink.kind += "nominal_possessive"
ReferenceLink = {kind, target: LocalReferenceId, evidence: Evidence[]}
Evidence = {start_byte, end_byte, quote}
```

Para un enlace válido `H.links[i] = {kind:"nominal_possessive",target:U,...}`, el nodo `U` es el **uso** del posesivo en esa ocurrencia, no su referente. La semántica introduce `deictic_referent_slot(U)` como posición abierta interna del operador, no como `Reference`, `Term`, ID de persona, variable cuantificada o candidato de respuesta. Ese slot no puede aparecer en `identity`, bindings, dominios, roles, enlaces ni cierres del documento. No hay lista de antecedentes ni alternativa vacía que signifique inventario cerrado.

El significado positivo del operador es:

```text
nominal_possessive(head_use=H, determiner_use=U,
                   referent=deictic_referent_slot(U), scope=H.scope_id)
```

Esta notación explica la semántica; no añade una proposición o un JSON paralelo. La relación restringe la interpretación nominal de H a la construcción con ese determinante y ese slot. No se descarga como `owned_by(H,X)`, `has_item(X,H)`, igualdad o cuantificación existencial. El slot permanece abierto porque la fuente no fija su valor y esta forma conserva esa apertura, no porque una extracción haya fallado. Su dominio semántico es el de referentes contextuales admisibles para la construcción: puede tratarse de una institución, curso, grupo u otro referente, sin imponer persona, cuerpo físico o existencia de un objeto. Los rasgos de persona/número solo se conservan cuando los licencia el lexema; no se usan para elegir un valor. Dos slots distintos son ocurrencias distintas, sin probar Same o Different de sus posibles referentes.

Ejemplo de nodos/enlace nuevos para la fuente, conservando el enlace nominal «de» existente:

```json
{
  "references": [
    {
      "id": "credential",
      "scope_id": "root",
      "mention": {"start_byte":60,"end_byte":83,"quote":"su credencial de alumna"},
      "name": null,
      "declared_class": {"start_byte":63,"end_byte":73,"quote":"credencial"},
      "semantic_type": null,
      "denotation": "entity",
      "links": [
        {"kind":"nominal_complement_de","target":"student_descriptor","evidence":[{"start_byte":63,"end_byte":83,"quote":"credencial de alumna"}]},
        {"kind":"nominal_possessive","target":"possessive_use","evidence":[{"start_byte":60,"end_byte":83,"quote":"su credencial de alumna"}]}
      ],
      "binding_scope_id": null,
      "anaphora": null
    },
    {
      "id": "possessive_use",
      "scope_id": "root",
      "mention": {"start_byte":60,"end_byte":62,"quote":"su"},
      "name": null,
      "declared_class": null,
      "semantic_type": null,
      "denotation": "lexical_use",
      "links": [],
      "binding_scope_id": null,
      "anaphora": null
    }
  ],
  "dependencies": [
    {"consumer":{"kind":"reference","id":"possessive_use"},"requires":{"kind":"scope","id":"root"},"role":"binding"},
    {"consumer":{"kind":"reference","id":"credential"},"requires":{"kind":"reference","id":"possessive_use"},"role":"identity"}
  ]
}
```

Es un fragmento de diseño, no un documento V3 completo ni candidata validada. Los IDs del ejemplo son reemplazables. El vínculo nominal «de», el ámbito, la dependencia claim→credential y el resto del documento se conservan. `role:identity` mantiene la etiqueta vigente de dependencia de una restricción referencial; declara que el consumidor requiere esa estructura, **no** que H y U sean Same. El motor debe seguir la arista implícita del enlace aunque falte la declarada y comprobar las reglas de declaración vigentes; nunca inferir igualdad desde una dependencia.

## Admisión por forma y propiedad de uso

El feature habilita una construcción, no un ID, predicado, caso o palabra suelta. En la primera forma de sintaxis española se admiten determinantes prenominales átonos `mi`, `mis`, `tu`, `tus`, `su`, `sus`, `nuestro`, `nuestra`, `nuestros`, `nuestras`, `vuestro`, `vuestra`, `vuestros`, `vuestras`, con comparación de mayúsculas Unicode estable y sin eliminar diacríticos. Esta lista reconoce morfología; no enumera referentes o lecturas semánticas. Voces posnominales, elipsis y otras lenguas requieren otra forma validada, manteniendo todo el objetivo final pendiente; no se degrada silenciosamente su semántica al primer patrón.

Para admitir este carrier deben verificarse conjuntamente:

1. H es una referencia nominal con núcleo acreditado por `declared_class` o `name`, sin convertir ese núcleo en clase universal. Su mention contiene el núcleo y U. No se exige que el núcleo sea la primera palabra del resto del grupo: pueden existir modificadores intermedios preservados por la representación; la evidencia acredita el tramo completo.
2. U cumple exactamente el shape del ejemplo: `denotation:lexical_use`, mention no vacía de un determinante admitido, `name/declared_class/semantic_type/binding_scope_id/anaphora:null`, `links:[]`. Sus campos nulos no significan anáfora fallida. Su `scope_id` es el de H. Su mention y el núcleo no se solapan, y el determinante precede al núcleo dentro de H. El intervalo que los une es evidencia íntegra; validar intervalos no demuestra por sí solo adjunción sintáctica fiel.
3. El enlace tiene una evidencia única, igual al mention de H. Debe conservar todos los complementos y modificadores representados dentro de ese intervalo. Cita exacta y cobertura no prueban la corrección semántica de la elección; la doble crítica sigue siendo necesaria.
4. U tiene exactamente una incidencia posesiva propietaria. Se calcula sobre todo el documento, sus aliases y dependencias efectivas; no basta contar `links` del primer nodo. H puede tener otros enlaces restrictivos acreditados. Para esta forma cada H tiene un solo enlace posesivo; duplicados, scopes diferentes o incidencias ambiguas dejan el carrier no soportado. Un guard o una rama no permite trasladar U a otro ámbito.
5. U no participa en Same/Different, grupos, dominios, binders, `reference_pattern`, roles de proposición/evento, slots, descripciones ni cierres. Tampoco es target de un enlace diferente. Una referencia a U en esos lugares es un uso inválido con sus consumidores pendientes; no se convierte en persona o entidad por tolerancia.
6. Sin feature, el nuevo kind/denotation queda no soportado según el contrato; no se ejecuta su recorrido semántico ni se altera una entrega histórica. Un vocabulario global mal formado sigue invalidando la operación. La presencia del feature no perdona problemas de evidencia, grafo, scope, cobertura o consumidor.

La propiedad de uso corresponde al par H→U y su ámbito documental. Dos ocurrencias «su» tienen dos usos y dos slots; no se unen por palabra, posición relativa o contexto nominal semejante. Ni siquiera un mismo referente posible permite unir los slots sin una forma de resolución autorizada. Esta versión no incorpora resolución. Si la fuente fija explícitamente un antecedente, una extracción que deje solo este carrier abierto pierde ese contenido y no puede declararse completa: debe representarlo mediante una forma de resolución revisada o quedar fuera de esta forma con brecha visible. Conservar el antecedente como hecho independiente sin su enlace no subsana esa pérdida. La fuente actual no autoriza resolver el slot a Ina.

## Completitud semántica con un argumento abierto

Una representación puede ser fiel y completa **de esta construcción** al conservar íntegramente el operador y la indeterminación referencial. Completitud de representación y capacidad de responder quién es el referente son ejes distintos. El operador contiene un argumento semántico abierto como parte de su significado, no un dato supuestamente conocido que falte en el raw. No hay `ignore_issues`, `source_uncertainty_ok`, `complete:true` o lista de lecturas para borrar deudas.

El validador admite la forma y conserva `coverage:represented` sobre U y la construcción cuando las reglas y la crítica la acrediten; el campo de cobertura no decide por sí solo la fidelidad. El matcher, al intentar usar esa restricción para una identidad o respuesta, deriva `possessive_referent` pendiente desde la estructura. La deuda de respuesta no depende de que el extractor recuerde poner un flag. Un enlace roto, antecedente inventado, omisión o evidencia incompleta sigue originando issues normales. Otros issues del consumidor tampoco desaparecen porque su posesivo tenga forma válida.

La candidata previa con `owned_by` y `anaphora:unresolved` no se reetiqueta ni se sanea en memoria. Un nuevo documento puede sustituir esa hipótesis, con nueva procedencia/hash y revisión, y conservar la fuente; el informe viejo y su causa permanecen. No basta borrar `anaphora` del objeto `entity` para conseguir aceptación: el nuevo uso exige la denotación, el kind y todas las invariantes de incidencia. El documento entero no se declara complete hasta resolver también sus otras brechas.

No se cierran las interpretaciones posibles del referente. Una lista vacía de antecedentes no significa «ninguno»; aquí no existe tal lista. El slot tampoco es un testigo existencial anónimo ni un universal sobre todos los poseedores. El significado conservado es una expresión dependiente del contexto cuyo referente esta fuente no fija. Esa indeterminación no vuelve verdadero cualquier valor que se proponga para ella.

## Corte previo, aliases, guards y consumidores

Antes de cualquiera de las rutas legacy, de relevancia, exclusión, descarga de guards, invariantes, proyecciones o publicación, se construye una vista tipada que distingue H y U. El recorrido parte de todos los usos léxicos y sus propietarios, recorre componentes Same completos y dependencias efectivas directas/implícitas, incluidos qualifiers, scopes normativos, referencias descritas y ramas. La arista H→U es restrictiva; no se elimina para que un nombre coincida. Toda restricción de aliases de H participa en ambos sentidos; una identidad declarada H↔otra entidad no resuelve el slot.

El consumidor que alcanza esta construcción se retira de la ejecución ordinaria y se conserva íntegro como candidato. Solo una ruta que conozca explícitamente este operador puede usarlo. Un guard que lo requiere no queda demostrado por su mero carrier; los claims que dependan de ese guard siguen pendientes. Un issue sobre U, H o un qualifier relevante se propaga a consumidores alcanzables. Un hecho independiente sin ese cierre conserva su utilidad. No hay abstención global de todo el documento porque una de sus construcciones tenga referente abierto.

Los targets léxicos no pueden satisfacer argumentos físicos ni producir identidad, derecho, posesión, localización, clase, Different o membresía. La protección se aplica a rutas estrictas, respuestas de eventos, restricciones, cierres, exclusión, premise correction, contexto, proyección y cualquier futura ruta; no solo al camino de respuesta directa. Si un consumidor no implementa el operador, queda `not_supported`/pendiente con causa concreta, sin desenvolver H o U a términos desnudos. El desconocimiento de identidad no prueba incompatibilidad ni Difference.

## Matching útil y límites de publicación

Se permite comparar construcciones localmente con propietario y contexto íntegros; se distingue esa comparación estructural de Same. La forma mínima de contexto parcial exige una conexión de relevancia ya declarada, argumentos externos acreditados (incluidos beneficiario y ámbito), compatibilidad de clase y todos los vínculos/qualifiers de H y sus aliases, sin deuda externa de scope, rama, guard o premisa. La referencia abierta del posesivo es una deuda expresamente conservada de esta ruta; otras deudas no se rebajan para hacerla pasar.

Si una pregunta exige un poseedor identificado, la fuente abierta no satisface esa restricción ni la refuta. Puede aportar contexto únicamente cuando el resto de la conexión acreditada permite identificar su utilidad, transportando la proposición fuente completa, H, U, enlaces, ámbito, dependencias y evidencia como source constraint. El testigo conserva la restricción solicitada y la deuda `possessive_referent`, `identity` o `answer_slot` pertinente. Un referente nombrado distinto no se declara falso por comparación con el slot abierto. Dos carriers abiertos tampoco satisfacen mutuamente su identidad.

El matcher no puede elegir un valor para el slot con el propósito de satisfacer la pregunta: se comprueba la construcción entera antes de producir contexto. Como control semántico previo a implementar, dos modelos con valores contextuales distintos para el mismo slot deben satisfacer la representación; las conclusiones dependientes de elegir uno de esos valores no se siguen de ella. Esta comprobación se realiza sobre el operador y no sobre una lista finita de lecturas.

En esta primera ruta, incluso con formas correspondientes, `answer_terms:[]`, `answer_propositions:[]` y `bindings:[]`. La estructura fuente va en `source_constraint` y el objeto íntegro de contexto; no se copia como conclusión o respuesta. Se registra `possessive_referent_proven:false` como resultado derivado del matcher, no como permiso del extractor. Si el formato de testigo actual no admite el contexto sin perder datos, se requiere ampliar ese DTO de salida antes de implementar; no se usa texto o JSON incrustado como sustituto. Es contexto parcial seleccionable bajo controles, nunca respuesta global.

El emparejamiento sin enlace de relevancia o sin ancla exterior acreditada conserva candidato pendiente. Se rechazan formas incompatibles; se conserva Unknown cuando falta prueba. El alcance de suficiencia, necesidad o permiso no se fabrica desde la mera restricción nominal. Hechos independientes como la eliminación del requisito de tutor pueden seguir la ruta ordinaria cuando no dependan de la construcción posesiva.

## Canonicalización y consumidores vecinos

`Same`, `reference_key` y canonicalización global nunca usan «su», sus rasgos morfológicos o el slot abierto como ancla de identidad. La firma de una construcción local incluye kind, patrón morfológico, papel núcleo/determinante, scope completo, restricciones del núcleo y aliases pertinentes, qualifiers, dependencias e incidencias correlacionadas. Conserva correspondencia de ocurrencias sin depender del texto de IDs. Normalizar IDs o renombrar scopes consistentemente puede conservar una firma estructural; esto no vuelve Same a dos usos de documentos distintos ni a sus referentes.

Repetir una misma forma en dos lugares debe conservar dos incidencias. La firma distingue compartir un uso inválidamente de repetir dos usos válidos; retiene el conjunto de aristas y sus posiciones estructurales. La evidencia y offsets originales se mantienen para auditoría, aunque no sean claves de identidad entre documentos. Una firma igual solo permite comparación estructural bajo un mapa local verificado; no descarga la deuda deíctica.

`nominal_complement_de` debe conservar «credencial de alumna» dentro del mismo H. Su target léxico nominal conserva las reglas existentes; no puede aceptar U como complemento ni saltarse el corte previo por reconocer primero «de». Cuando el cierre del propietario incluye el posesivo, su ruta solo puede devolver contexto que transporte ambos operadores íntegros o permanecer pendiente. Se necesita una comprobación conjunta; no borrar un enlace para aprobar el otro.

`existential_conclusion_projection` rechaza U y su slot como objetos existenciales, valores de respuesta o argumentos físicos. En su primera forma, una conclusión cuyo cierre de operandos incluya el carrier posesivo queda `not_supported`; no proyecta un subconjunto que pierda su restricción. La ampliación futura tendría que preservar el operador y el slot abierto dentro de una conclusión tipada, demostrar su semántica y volver a pasar revisión, sin fabricar un testigo para el referente. Esta recusación local no restringe el objetivo final de representar la fuente ni habilita omitirla del denominador.

## Presupuesto y errores

Índices de incidencia, aliases, cierres efectivos, evidencias, firmas, mapas, clonación y testigos se cargan al presupuesto acumulado de la operación antes de reservar/copiar. Las rutas nuevas no recorren este grafo si falta el feature. Cada nodo/arista se visita con control de ciclos; no hay recorridos repetidos sin cobro por cada consumidor. Un límite local o global produce limit técnico de toda la operación, nunca parcial exitoso obtenido de una vista truncada. La implementación debe definir diagnósticos cerrados para carrier inválido, incidencia ambigua, consumidor no soportado y agotamiento, preservando las causas originales y sin fabricar recibos de ejecución en este diseño.

## Segunda brecha: suficiencia de término

El contrato dice `class_suffices(subject,class)` y suficiencia normativa literal con beneficiario y ámbito. La candidata usa `class=ref credential`; el rol admite `term` en su vocabulario, pero eso no demuestra que la semántica vigente acepte una condición nominal particular frente a una clase. Este diseño del posesivo no resuelve esa diferencia por conveniencia.

Antes de autorizar esa extracción debe decidirse y revisarse separadamente si `class_suffices` admite un término nominal restrictivo con esta lectura, sin distribución sobre todas las credenciales, existencia física, permiso, necesidad o ejecución. Si no la admite, hace falta una relación genérica de suficiencia de término con definición y controles propios. No se añade aquí un predicado ni se modifica `distributive` para conseguir cierre. La preservación del beneficiario Ina tampoco demuestra el referente de «su». Esa segunda brecha puede impedir que el documento entero sea semánticamente completo aunque el carrier posesivo esté completo.

## Verificación futura y puerta intacta

La implementación posterior necesita controles breves de admisión/rechazo y preservación: formas morfológicas admitidas, evidencia/adjunción incorrectas, otro scope, duplicados, dos usos iguales distintos, targets reusados, Same/Different con U, aliases transitivos/restricciones ocultas, dependencia implícita por qualifier o ámbito normativo, guard pendiente, ramas, ciclos, presupuestos, hechos independientes y replay sin feature. Debe cubrir además matching parcial con restricciones completas, petición de referente conocido, otra persona, los dos operadores nominales juntos y rechazo por la proyección existencial. Los ejemplos deben variar nombres, núcleos, predicados e IDs; ningún dispatch depende de Faro, Ina, credencial o un ID de campaña.

Antes de implementar, doble crítica del operador debe comprobar por modelos la conservación de la restricción sin colapso de referentes, ausencia de existencia/propiedad y prohibición de satisfacer guards mediante elección oportunista del slot. Esta revisión no prueba extracción.

Estas pruebas comprobarían el mecanismo local. Después hacen falta nueva autoría fuente-only, procedencia y hashes, doble crítica separada y extracción completa sobre los 116 textos, sin borrar los fallos previos, relajar aceptación, excluir textos difíciles o llamar éxito a `not_supported`. La fidelidad completa y la utilidad parcial son medidas distintas; conservar una construcción correctamente no garantiza recuperar toda su utilidad. Mantener también comparación real con baseline, recursos/estabilidad y casos nuevos ciegos antes de adopción. Un diseño o un ejemplo manual aceptado no cambia ninguna de esas puertas.

## Addendum normativo previo a implementación

Se incorpora al vocabulario de la candidata la fila exacta `{id:"condition_suffices",roles:{beneficiary:"ref",condition:"term"},distributive:false}`. Su significado es suficiencia normativa del término-condición íntegro para ese beneficiario dentro del régimen/ámbito declarado por la proposición. No añade consecuencia concreta no expresada, existencia física, presentación, permiso, necesidad ni distribución sobre una clase. No condiciona retrospectivamente la eliminación ya afirmada del requisito. `class_suffices` y sus datos históricos quedan intactos; no hay equivalencia ni backmapping automático.

La salida especializada usa el testigo parcial existente con `kind:"nominal_possessive_context"`. Conserva campos ordinarios y añade exactamente `source_constraint` (Claim íntegro), `request_constraint` (Request íntegro), `source_references`, `question_references`, `source_scopes`, `question_scopes`, `source_qualifiers`, `question_qualifiers`, `source_dependencies` y `question_dependencies` (arrays íntegros de cada documento), además de `source_document` y `question_document` (DocumentaryV3 tipados íntegros validados, antes de derivar vistas privadas). Estos documentos conservan todos los claims, requests, identity, namespace, qualifiers, scopes, cierres y dependencias; los owners/arrays redundantes del testigo deben coincidir exactamente con ellos. No son texto ni JSON citado, y toda copia se cobra antes de materializar. `bindings`, `answer_terms` y `answer_propositions` son arrays vacíos. `possessive_context` tiene exactamente `{version:"documentary_nominal_possessive_context_v1",source_owners:ReferenceId[],source_uses:ReferenceId[],referent_resolution:"source_indeterminate",possessive_referent_proven:false}`. Todos los IDs resuelven referencias del mismo testigo/documento, no personas. La estructura no es texto ni JSON citado y no elige un valor contextual.

La referencia léxica inválida, evidencia o enlace roto usa `invalid_shape`/`dangling_id` según el validador vigente; una forma estructural no soportada usa `unsupported_nominal_possessive` y conserva deuda. La causa derivada de matching es `nominal_possessive_referent`; nunca es un flag suministrado por el extractor. `request_context_limit` sigue siendo error técnico de toda la operación y conserva entrada íntegra. Ninguno de esos resultados cuenta como acierto de calidad o negativa semántica por sí solo. Un issue previo no se elimina por habilitar el feature.

En el perfil cerrado hay ahora ocho features conocidos. `lexical_use` solo se admite con `nominal_possessive` válido y con su incidencia autorizada; ningún otro lector recibe permiso implícito. No cambia la versión de la estructura V3, pero es una ampliación optativa de sus enums que un lector anterior debe rechazar. El coste nuevo de validación, cierre, matching y copia se cobra al presupuesto compartido. Los mensajes y artefactos de implementación deben declarar exclusiones, sin eliminar casos del denominador ni atribuir utilidad autónoma al control manual.
