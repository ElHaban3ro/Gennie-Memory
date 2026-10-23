# Complemento nominal referencial: contrato previo a implementación

Fecha: 2026-10-22. Propuesta cerrada para crítica distinta antes de implementar. Solo evaluación local; no modifica código, raws, gold, veredictos, métricas o puertas. Parte de la [adjudicación prospectiva](documentary-v3-coherent116-contract-adjudication-2026-10-22.md). No añade un operador13, dependencias Cargo, configuración runtime o política de producto. No amplía resolución posesiva: κ(U,c0) permanece rígido bajo su contrato propio.

## Decisión y semántica

Se reutiliza `Reference.links.kind=nominal_complement_de` del feature6 con un **modo explícito `referential`** y permiso opcional versionado en vocabulario. El modo histórico sin campo `mode` conserva exactamente su significado de descriptor léxico local y todas sus restricciones. No se convierte automáticamente un target nombrado en descriptor, ni se interpreta un link histórico como referencial por su evidencia.

El nuevo operador significa `NominalDeRef(H,T,form)` en esa incidencia y scope: H tiene el complemento nominal referencial T introducido por la forma española conservada. No publica qué relación mundana expresa el genitivo; no equivale a asociación incidental, propiedad, pertenencia, ubicación, material, procedencia o función. No es una variable de relación que el consumidor pueda completar. El subdiagrama dirigido, la forma y sus operandos son contenido fijo. Identificar T o H no resuelve ese subtipo.

Un target referencial puede reaparecer acreditadamente en otros roles, conservar metadata y ser complemento de varios owners. Esto lo distingue del descriptor léxico de incidencia exclusiva. Reutilización de ID local o Same requiere su evidencia ordinaria; ni la grafía repetida ni el nuevo link crean identidad global. Un target común definido conserva la referencia contextual a esa ocurrencia; el artículo no publica unicidad mundial, existencia física adicional ni identidad entre documentos. La ligadura al origen impide sustituir su contexto durante el consumo.

## Permiso y esquema exactos

El vocabulario puede añadir exactamente este objeto opcional de un campo:

```json
"nominal_referential_complement_profile": {
  "version": "documentary_nominal_referential_complement_v1"
}
```

Requiere un `request_context_profile` válido que permita `nominal_complement_de`. No modifica sus doce features ni su máximo. Objeto nulo, versión desconocida, claves adicionales o ausencia del feature requerido invalidan el vocabulario completo (`unknown_vocabulary`); no autorizan fallback. El permiso forma parte del vocabulario canónico y su digest. Ausencia del nuevo objeto no activa recorridos ni modifica la interpretación histórica. Un lector anterior rechaza el campo desconocido; no se presenta esta ampliación como un documento comprensible por lectores que solo conocen el vocabulario anterior.

El link nuevo tiene **exactamente siete campos**, todos obligatorios:

```text
{
  kind: "nominal_complement_de",
  mode: "referential",
  target: ReferenceId,
  form: "de" | "del" | "de_la",
  marker: Evidence,
  determiner: Evidence | null,
  evidence: [Evidence]
}
```

`Evidence` conserva exactamente `{start_byte,end_byte,quote}` UTF-8 semiabierto no vacío y literal. `evidence` contiene un único elemento, la construcción entera. No hay campos para relación resuelta, confianza, candidatos, respuesta o posesión. `mode` ausente sigue seleccionando exclusivamente el shape histórico `{kind,target,evidence}`; `mode:null`, `mode:lexical_descriptor` u otros modos no se admiten en esta versión. Tampoco se añaden `marker`, `form` o `determiner` a un link histórico.

La nueva forma solo se admite con el nuevo permiso. Sin él, un link con `mode:referential` produce `invalid_shape`; no se limpia el modo ni se intenta `descriptor_key`. Referencias y demás DTO V3 mantienen sus campos existentes. No hay nueva entidad física, categoría de Reference, predicado o binder generado por el link.

## Evidencia, núcleos y formas admitidas

H conserva clase, nombre cuando existe, denotación y mención. Su núcleo es `name` si no es null; en otro caso `declared_class`. T conserva un `mention` propio de **esta ocurrencia complemento** y al menos `name` o `declared_class`. El núcleo T se toma de `name` si existe; si no, de `declared_class`. Los spans se toman de esos campos exactos; nunca de una búsqueda por substring en una cita amplia. Menciones repetidas tienen offsets distintos.

La versión inicial reconoce únicamente las siguientes formas, sin eliminación de diacríticos; la comparación de tokens funcionales admite mayúsculas Unicode de manera estable, conservando siempre bytes/citas originales. `WS` significa uno o más espacios Unicode, sin puntuación u otra palabra. H_core y T_core son spans no vacíos; su evidencia también valida ordinariamente.

| form | marker.quote | determiner | Tramo exacto después de H_core | target.mention |
| --- | --- | --- | --- | --- |
| `de` | `de` | null | WS + marker + WS + T_tail | T_tail |
| `del` | `del` | null | WS + marker + WS + T_tail | T_tail |
| `de_la` | `de` | evidencia exacta de `la` | WS + marker + WS + determiner + WS + T_tail | determiner + WS + T_tail |

`T_tail` es exactamente T_core, o `T.declared_class + WS + T.name` si ambos existen con spans distintos, ordenados y no solapados. En esta segunda forma `T_core` es el nombre y la clase está explícitamente conservada; no se permite otro modificador omitido entre ambos. Si la clase y el nombre cubren el mismo tramo, solo cabe la primera forma de tail y cada campo sigue sujeto a su validación ordinaria. Otros modificadores, coordinaciones o complementos dentro de T_tail quedan fuera del subset, con brecha visible; no se recortan para admitirlos.

`evidence[0]` va exactamente desde el inicio de H_core al final de target.mention y coincide con ese tramo. H.mention contiene íntegramente esa evidencia. Marker, determinante cuando existe y T_tail tienen los intervalos exactos de la tabla, sin solapar H_core. En `de_la`, determiner está dentro de T.mention y antes de sus núcleo/clase; no se duplica como otra Reference. En `del`, la contracción conserva conjuntamente `de`+artículo `el` en marker: no se inventa una evidencia separada de «el» ni se añade un artículo inexistente a target.mention. Así se conserva su realización literal además de su función determinativa. No se equiparan firmas de `del`, `de` y `de_la` eliminando el artículo.

Ejemplos genéricos de admisión: «sensor de Talía» (T nombrado), «sensor del equipo Azul» (T group, clase equipo y nombre Azul), «sensor de la estación» (T entity común definido), «sensor del depósito» (T entity común definido). No son representaciones de entradas del corpus ni prueban adjunción fiel por contener un tramo válido. «Sensor hecho de Talía» no cumple la contigüidad desde su núcleo; «sensor de Talía» tampoco acredita por sí mismo propiedad o procedencia.

## Referencias admitidas y subset

H y T pueden tener `denotation:entity` o `group`, `semantic_type` ausente o acreditado por el contrato existente, y nombre o clase explícitos. No se permite `class` o `lexical_use`. Ambos tienen `binding_scope_id:null`, `anaphora:null` y el mismo scope documental raíz; ninguna de sus componentes Same contiene binder, rama o contradicción Different. El nuevo link no añade cuantificación o distribución sobre miembros del grupo. Una entidad común sin nombre solo es target referencial en `del` o `de_la`; «sensor de metal» anónimo no se reinterpreta como complemento referencial mediante este permiso y sigue perteneciendo, cuando corresponda, al modo descriptor histórico.

T nombrado puede aparecer con las tres formas si su evidencia las acredita. **El target group solo se admite aquí como grupo simple nombrado**, sin cardinalidad, miembros, partición, agrupamiento de otros grupos o cuantificadores en su cierre nominal. Un target común definido sin nombre solo se admite como `entity`; el grupo común definido queda como brecha hasta un contrato separado. «Seis parejas» no se opaca en `declared_class`, no se convierte en nombre ni se fuerza a entity: su cardinalidad/agrupamiento requiere carriers y verificación propios antes de abrir ese subset. Lo mismo vale para grupo nombrado con cantidad o miembros que sean parte efectiva de su restricción. H group conserva el permiso previo y sus restricciones existentes; este cambio no amplía su cuantificación. Los grupos nombrados simples y los targets entity comunes definidos son por tanto admitidos de forma, sin prometer resolver su identidad entre documentos. Un nombre propio no suministra semantic_type por capitalización.

Por cada **ocurrencia H** se admite un link nuevo; no se duplican dos links sobre el mismo intervalo. Varios aliases acreditados de H pueden transportar sus propias ocurrencias y deben preservar todas las restricciones, sin colapsar sus evidencias. La misma Reference T puede usarse en varios links solo cuando su mention esté contenida en todas las construcciones respectivas y el texto lo acredite; normalmente ocurrencias distintas del mismo referente usan References locales distintas con Same acreditado. Se admite que varias References T de ocurrencias distintas pertenezcan a una misma componente Same. La exclusividad owner→descriptor del modo histórico no se aplica a esa componente referencial.

Cada componente Same de H/T conserva todas las References, denotaciones compatibles, evidencias y restricciones de aliases. Una componente con entity y group incompatibles o Same y Different contradictorios bloquea. No se propaga una clase del grupo a cada miembro ni se unifican dos entidades por compartir complemento. El target no puede ser alias de su propio H; self-link o un ciclo que contenga un nuevo link sobre el cociente de aliases es inválido.

El subset estructural inicial es un solo nivel del nuevo operador: T y su cierre de referencias restrictivas no contienen ningún link nominal_de de ningún modo ni posesivo; H tampoco mezcla otro link nominal_de o posesivo en su componente. No se borra una restricción anidada para entrar. H/T pueden tener links ordinarios `owned_by`, `located_at`, `part_of`, `member_of`, `associated_with` ya acreditados; sus targets y cierres íntegros siguen las reglas V3, sin nuevos binders/anáforas o ciclos que involucren el nuevo nominal. Su presencia no determina el subtipo del genitivo. Se conserva la profundidad64 y el presupuesto global; no se abre recursión ilimitada por esos links. Una forma válida fuera de este subset produce `unsupported_nominal_referential_complement`, no aceptación parcial estructural.

Los claims ordinarios de metadata sobre T siguen siendo claims del documento. Solo entran como restricciones consumidas de H cuando lo requieren sus dependencias explícitas/efectivas o un término descrito; compartir referente no añade por sí solo una dependencia de toda su biografía. El documento/testigo conserva esa metadata, incluidas las afirmaciones independientes. Un claim directo e independiente sobre un target nombrado compartido debe seguir respondiéndose por la ruta ordinaria. El caso contrario —metadata incorporada mediante descripción/dependencia— conserva su correlación y puede bloquear el consumidor de H, sin eliminarla.

## Cierre dirigido y corte previo a legacy

La dependencia intrínseca es H→T. Se exige la dependencia explícita vigente de Reference H a Reference T con `role:identity`; la etiqueta significa restricción requerida, **no Same(H,T)**. El recorrido efectivo añade la arista aunque el autor omita declararla y reporta la omisión según las reglas ordinarias. Las dependencias propias de scope y aliases se conservan. Nuevos campos de evidencia no se interpretan como References.

El corte se siembra en los owners H del modo referencial y sus aliases, y bloquea sus consumidores transitivos explícitos/implícitos: claims, requests, qualifiers, guards, scopes, referencias descritas, reglas, ramas, cierres e invariantes. Se calcula antes de identidad ordinaria, matching de respuestas/eventos, relevancia legacy, exclusión/refutación, descarga de guards o proyección E. Ninguna ruta recibe un H desnudo. Nombre coincidente o un alias sin link no elude el corte.

**T no es una semilla de bloqueo por el mero enlace entrante.** Se recorre para conservar sus restricciones y diagnosticar deudas de H, pero no se invierte H→T. Un hecho independiente sobre T que no consuma H ni su construcción permanece ordinario. Si T depende efectivamente de H, deja de ser independiente y queda bloqueado; si eso crea el ciclo prohibido, se diagnostica el ciclo. Conservar el documento completo en la salida no convierte cada claim copiado en un consumidor del nominal ni en una respuesta publicada.

La proyección existencial E rechaza localmente toda conclusión cuyo cierre consuma el nuevo H; no publica el núcleo omitiendo su complemento ni usa T como testigo por derivación del link. Un T que ya participa legítimamente en un hecho independiente conserva sus permisos previos. Los guards que requieran H no se descargan por identidad del nombre o por un testigo parcial; un guard independiente acreditado puede seguir validando. Se preservan y propagan issues efectivos, sin abstención global por mera coincidencia de target.

## Selector parcial real, estrecho y verificable

Se integra una ruta optativa en `classify-documentary-context-v2`; no basta rechazar todos los nominales. Esta primera versión admite **solo** fuente Claim de `body.kind:proposition` positiva, `judgment:fact`, `status:state`, predicado `quantity(subject:ref H,attribute:symbol A,amount:quantity)`, con la firma exacta registrada. Request: `mode:answer`, `slot:amount`, `required_answer_extent:one_value`, mismo predicado/atributo/juicio/estado, sin amount prellenado, con `subject:ref Hq`. No introduce una equivalencia de atributos ni conversiones de unidades. La unidad solicitada ausente o coincidente según validación existente; una unidad incompatible bloquea esta selección. A puede ser cualquier atributo ya registrado compatible con la firma, sin lista de IDs del corpus.

H fuente y Hq tienen el nuevo nominal admitido, T/Tq íntegros y denotaciones/núcleos/clases/tipos compatibles. Si ambos tienen nombres, una identidad desconocida no se convierte en Different por comparar cadenas; nombres/clases por sí solos no acreditan Same. Se evalúan los aliases completos y todos los links adicionales de ambos lados. Los links ordinarios requeridos por cualquiera de los dos cierres deben corresponder con identidad acreditada y mismo kind; los modificadores adicionales se conservan, no se descartan. En esta primera ruta no se selecciona por un subconjunto favorable de ellos. El modo lexical_descriptor no sustituye al nuevo modo.

Se necesita **al menos un ancla Same probada bajo las reglas de identidad y namespace existentes**, en uno de dos lugares: H↔Hq o T↔Tq. El ancla de H acredita únicamente identidad referencial, por prueba independiente de sus modificadores; no acredita satisfacer su descripción completa. El ancla de T permite comparar el patrón nominal H completo pero no identifica dos H anónimos. No se elimina el link para llamar al matcher ordinario y obtener Same. La implementación debe separar una prueba de identidad acreditada de la satisfacción de todas las restricciones; si el motor no puede producir esa prueba sin borrar restricciones, no hay selección. No se añade un nuevo axioma de identidad por nombres o ID iguales. La prueba se recalcula en el lector con documentos, namespace y pair_scope originales.

Al menos una de las dos relaciones de identidad debe ser Same; la otra puede ser Unknown y se conserva como deuda `identity` para este parcial. Different/Conflict de H o T bloquea la selección, sin convertirla en refutación de la cantidad. La excepción Unknown se limita a identidad del otro operando del nominal; no perdona configuración, ámbito, tiempo, guards o dependencias externos. Si ambos son Unknown, queda candidato pendiente, nunca parcial seleccionado. El subtipo genitivo no se resuelve ni se compara con owned_by/material/location para seleccionar.

Requisitos conjuntos adicionales: documentos completos por el validador; cobertura representada en ambos cierres; scopes documentales sin branches/binders/premisas; mismo contexto efectivo, configuración y granularidad temporal; guards acreditados que no dependan del nominal; fundamento exacto si la pregunta lo exige; ámbitos normativos ausentes en este subset y ningún contexto composicional adicional no soportado. Cualquier issue efectivo ajeno a las dos deudas de identidad internas expresamente contempladas bloquea la selección. Metadata o deuda de otro claim independiente no la bloquea. Si un término descrito, normativo, otro predicado o una composición consume H, el corte sigue funcionando, pero su selección queda fuera de esta versión; no se degrada a quantity.

El resultado seleccionado es `partial_relevance`: contexto íntegro pertinente a esa petición, sin apoyo/refutación completos. Siempre conserva `answer_slot` y `genitive_relation`; añade `identity` cuando cualquiera de H/T no tenga Same entre lados. `genitive_relation` significa **subtipo mundano no publicado**, no pérdida de `NominalDeRef` ni permiso para resolverlo. Incluso con ambos Same, se publican `bindings:[]`, `answer_terms:[]`, `answer_propositions:[]`, `requirement_states:[]`. Que la fuente incluya un amount no permite extraerlo por una ruta ordinaria lateral. Una ampliación futura puede habilitar otras inferencias solo con conservación y controles propios.

## Testigo exacto

El testigo `kind:nominal_referential_complement_context` tiene exactamente los catorce campos base de Witness existentes: `id`, `kind`, `claim_ids`, `request_id`, `bindings`, `evidence_refs`, `qualifier_ids`, `branch_ids`, `missing_constraints`, `answer_terms`, `answer_propositions`, `requirement_states`, `conclusion_judgment`, `normative_scope`. Sus tipos siguen el DTO vigente; `claim_ids` contiene solo el Claim seleccionado, `request_id` el Request seleccionado, `conclusion_judgment:fact`, `normative_scope:null`, `branch_ids:[]`; los cuatro arrays de publicación anteriores están vacíos. `evidence_refs` y `qualifier_ids` conservan los de la proposición y su contexto efectivo, sin inventar prueba desde una quote.

Añade exactamente: `source_constraint` (Claim íntegro), `request_constraint` (Request íntegro), `source_document`, `question_document` (V3 íntegros antes de filtros privados), `vocabulary` (vocabulario completo), `source_references`, `question_references`, `source_scopes`, `question_scopes`, `source_qualifiers`, `question_qualifiers`, `source_dependencies`, `question_dependencies` (arrays exactamente iguales a los documentos), `effective_source_closure`, `effective_request_closure` y `nominal_referential_context`. No admite `source_body` separado, `possessive_context` ni campos extra.

Los cierres son arrays ordenados lexicográficamente por `(kind,id)`, sin duplicados, de endpoints exactos `{kind,id}` de cada grafo efectivo, con Claim/Request propietario incluido. El espacio de kind conserva sus categorías; no se fusiona un id de Reference con un id de otro tipo. El cierre incluye H/T, sus aliases, links, qualifiers y dependencias efectivamente requeridos, no todos los hechos entrantes sobre T. Los documentos íntegros preservan los datos restantes. `missing_constraints` es un array ordenado sin duplicados, derivado; para un seleccionado sus valores son exactamente `answer_slot`, `genitive_relation` y opcionalmente `identity`. Deudas adicionales impiden selección y permanecen en el candidato pendiente íntegro.

```text
nominal_referential_context = {
  version: "documentary_nominal_referential_context_v1",
  source_head: ReferenceId,
  question_head: ReferenceId,
  source_complement: ReferenceId,
  question_complement: ReferenceId,
  source_form: "de" | "del" | "de_la",
  question_form: "de" | "del" | "de_la",
  head_identity: "same" | "unknown",
  complement_identity: "same" | "unknown",
  anchor: "head" | "complement",
  genitive_relation_resolution: "not_published",
  nominal_construction_preserved: true,
  full_answer_supported: false
}
```

No es entrada de autoría: todos los campos son resultados recalculados del selector. Los cuatro IDs resuelven exactamente los dos links en los documentos. Si head_identity es same, anchor es head; de lo contrario debe ser complement y complement_identity same. Form conserva cada realización, aunque las identidades se acrediten por otra prueba. No se cambian de/del/de_la para hacer idénticos los documentos. El lector comprueba todo el testigo y sus redundancias, no solo los flags.

El candidato no seleccionado conserva Claim/Request/documentos íntegros y deudas concretas existentes más `nominal_referential_complement` cuando falta soporte. No produce un archivo de referencia exitoso por el mero hecho de conservarlo. `invalid_shape`, `dangling_id`, `unsupported_nominal_referential_complement`, `unknown_vocabulary` y `request_context_limit` conservan su diferencia entre forma, referencias, subset, permiso y agotamiento; ningún rechazo es un negativo semántico correcto por sí solo.

## Transporte de origen y lector real

Se propone **`source-context-reference-v3 bind|consume INPUT MANIFEST OUTPUT`**, extensión explícita del módulo existente. V1 y V2 mantienen sus formatos, permisos y salidas históricas; no aceptan el nuevo testigo. V3 admite los testigos que V2 ya admite bajo sus permisos previos y el nuevo testigo bajo `nominal_referential_complement_profile`. No se reinterpreta otro kind como el nuevo.

El manifiesto continúa exactamente `documentary_source_context_manifest_v1` con `origins:[{origin_id,source_input_sha256}]` y `deliveries:[{delivery_id,origin_id,delivery_input_sha256}]`, no vacíos, hasta512 filas por tabla, IDs únicos. Delivery tiene exactamente `{version:"documentary_source_context_delivery_v3",origin_id,delivery_id,selection:{request_id,claim_id},pair:{vocabulary,source:{text,raw},question:{text,raw},pair_scope}}`. Consume tiene exactamente `{version:"documentary_source_context_consume_v3",delivery:DeliveryV3,reference:ReferenceV3}`.

ReferenceV3 tiene exactamente `{version:"documentary_source_context_reference_v3",origin_id,delivery_id,manifest_sha256,source_input_sha256,delivery_input_sha256,selection:{request_id,claim_id},source_input:{vocabulary,text,raw},witness:Witness,interpretation:{kind:"source_context_reference",origin_id},answer_terms:[],answer_propositions:[],bindings:[]}`. IDs no vacíos hasta128bytes; digests SHA-256 hex minúscula de64caracteres con serialización canónica vigente del evaluador. El source_input digest cubre exactamente `{vocabulary:pair.vocabulary,text:pair.source.text,raw:pair.source.raw}`; delivery digest cubre el DeliveryV3 completo. Manifest digest cubre el manifiesto íntegro. PairScope mantiene su contrato vigente sin nueva tolerancia.

Bind revalida manifiesto y entregas, valida ambos documentos completos, ejecuta la ruta real de clasificación y selecciona exactamente un testigo permitido del Claim/Request indicados. Ausente, múltiple, pendiente o de tipo no permitido: rechazo local sin referencia exitosa. Verifica documentos, Claim, Request, arrays, cierres, IDs, formas, deudas e identidades derivadas. No acepta un Witness suministrado por el caller como sustituto de clasificación. Consume vuelve a ejecutar bind con la entrega original, y compara **la referencia completa** bajo el mismo presupuesto. Desprender el wrapper o añadir respuestas invalida la referencia.

El origen pertenece a una ocurrencia asignada por el runner, no al nombre de T ni al hash del texto. Varias entregas registradas para preguntas distintas mantienen el origen y el mismo nominal fuente; textos iguales de orígenes distintos no se unifican. No existe argumento c0 de destino ni resolvedor de relación; cambiar la pregunta no cambia la fuente o especializa su genitivo. El documento íntegro no autentica autoridad maliciosa, verdad, frescura ni comportamiento de un LLM que ignore el lector. El lector acredita transporte contextual verificado en evaluación, no integración de producto o autonomía final.

## Integración y coste mínimo

La inspección estática localiza estos puntos: `request_context_profile.rs` conserva su feature6; `validation.rs::vocabulary` necesita admitir y validar el nuevo perfil cerrado y el digest lo incluye; `validation/nominal_de.rs` despacha por modo sin relajar el histórico; `nominal_de.rs::descriptor_key` permanece exclusivamente léxico y no acepta referentes; las funciones de colección/guard distinguen modos. `validation/graph.rs` y dependencias efectivas recorren H→T preservando scope y aliases. Las evidencias escalares marker/determiner deben entrar en validación, inventario de evidencias, hash, límites y canonicalización, además de evidence[].

En `pair.rs` y `nominal_de_matching.rs`, `association_context_matches`, `identity_depth`, `answer_reference_pattern` y los guards no pueden confundir ambos modos por compartir kind. Reutilizar cierres tipados y corte de `nominal_possessive.rs`/`lexical_clause_context.rs` como infraestructura no incorpora semántica posesiva. El selector necesita una ruta especializada y una prueba de ancla sin retirar restricciones; la ruta ordinaria sigue cortada. `quantified_match.rs` no amplía su whitelist. `linguistic_composition_matching.rs`, `nominal_possessive_matching.rs` y `nominal_event_context.rs` usan hoy descriptor_key o supuestos del modo léxico: ante el nuevo modo en una composición no soportada deben conservar/declarar la deuda, nunca reinterpretarlo o desenvolverlo. La proyección E necesita el corte por nuevo H y sus dependientes antes de inferir.

Canonicalización/firma conserva kind, mode, form, dirección, clase/denotación/nombre, identidad y topología de H/T/aliases, roles de dependencia, scope y todas las restricciones. Renombrado consistente de IDs y orden de tablas/links no cambia la estructura; cambiar target, modo, artículo/contracción, owner o incidencia sí la cambia. Evidencias y offsets se conservan para auditoría aunque no sean identidad entre documentos. No se aplica la deduplicación de un descriptor léxico al target referencial compartido. La comparación no equivale NominalDeRef a associated_with.

`documentary_v3_io/source_context_reference.rs` incorpora ProtocolV3, su whitelist de testigos y validador cerrado del nuevo Witness, sin modificar ProtocolV1/V2. Se reutilizan parseo JSON estricto, manifiesto, digests, reejecución y escritura a archivo nuevo. El adaptador CLI registra el nuevo comando. La ampliación no es solo un cambio de enum: su coste mínimo incluye perfil/forma/evidencia, cierres y corte dirigido, firmas, selector parcial positivo, DTO de salida y bind/consumeV3. No requiere un motor general de genitivos, resolvedor de posesivos, nuevos predicados o servicios.

## Presupuesto y atomicidad

Se conservan límites de16MiB acumulados, profundidad64,100000nodos,512owners,4096evidencias,32por lista y1000000pasos de la operación compartida. Marker/determiner cuentan también como evidencias; redundancias del testigo cuentan donde corresponda. Leer entrada+manifiesto acotados antes de parsear el segundo íntegro; JSON sin claves duplicadas. El preflight agrega ambas raíces sin clonarlas en un objeto conjunto y mantiene contadores acumulados.

Precobrar antes de reservar/copiar índices, componentes Same, colas/sets de cierre, ordenación, mapas de correspondencia, strings y documentos/testigos/arrays redundantes. Cobrar al menos un paso por nodo/arista/comparación y bytes de cadenas/evidencias/hashes, con overflow que agote, no sature en éxito. Medir serialización y reservar cada copia antes de materializarla, incluida salida; reejecución consume usa el mismo saldo, sin reiniciar por head, target, claim, modo o fase. La búsqueda de correspondencia no enumera productos o backtracking no cobrados. Un límite invalida la operación entera aunque ya exista un parcial positivo; no recorta un cierre ni publica salida favorable. Mantener la entrada del llamador y no sobrescribir archivos. Las cotas son unidades conservadoras, no latencia o RAM medida.

Sin el nuevo perfil no se recorren helpers nuevos ni se cobra su trabajo. La corrección de un déficit de contabilidad detectado al integrar debe declararse por separado si cambia el saldo histórico; no prometer igual capacidad límite de una versión que no contabilizara memoria. Sin agotamiento, los replays históricos V1/V2 y léxicos deben conservarse exactamente.

## Controles genéricos requeridos antes de aceptación

| Control con DTO completos salvo rechazo de forma indicado | Resultado exigido |
| --- | --- |
| Nombre propio de/del, grupo simple nombrado y entity común definida de la/del | Admisión del modo y evidencias; no propiedad/material/localización ni distribución/miembros inferidos. |
| Link histórico válido con y sin nuevo permiso | Misma semántica/salida léxica; no migración implícita o modo añadido. |
| Nuevo modo sin permiso, perfil inválido, keys extra, modo null/desconocido | Rechazo de vocabulario/forma, sin fallback. |
| Metal anónimo con de; class/lexical_use como T; anáfora/binder/rama; grupo común definido o seis parejas | Fuera del modo/subset; no coerción a nombre, descriptor o classstring que oculte cantidad/agrupación. |
| de la sin la, del descompuesto con offset inventado, marker ajeno, span incorrecto/UTF-8, modificador omitido | Rechazo de forma/evidencia; no admitir mediante substring. |
| Dos T de ocurrencias distintas con Same acreditado; varios H y mismo referente T | Admisión sin exclusividad léxica, ocurrencias y aliases conservados; no unión por grafía. |
| H↔T Same, ciclo con nuevo link, doble incidencia sobre mismo H | Rechazo con causa, sin cortar una arista para aprobar. |
| Alias H sin link, alias T con restricción adicional, metadata incorporada por dependencia | Restricción y cierre íntegros; ningún bypass de nombres, alias o core. |
| Metadata independiente sobre T compartido frente a claim consumidor del mismo H | Primera ruta ordinaria disponible; consumidor H cortado antes de legacy/guard/E. |
| Quantity genérica y request amount correspondientes; T Same, H Unknown | Parcial real con nominal/documentos/cierres íntegros; answer_slot/genitive_relation/identity y cuatro arrays de publicación vacíos. |
| Quantity correspondiente; H Same independiente, T Unknown | Parcial anclado en H con identidad T pendiente; no responder o elegir T para satisfacer pregunta. |
| H y T Same | Parcial sin deuda identity; genitive_relation y answer_slot siguen, sin respuesta lateral. |
| Ambos Unknown; ancla acreditada solo por clase/string/ID; Different/Conflict; atributo/unidad ajenos | No selección positiva ni refutación fabricada; conservar causa/candidato. |
| Scope/configuración/tiempo/fundamento/guard/cobertura efectivos pendientes | Bloqueo de selección; deuda independiente en otro claim no causa abstención global. |
| Petición de owned_by, location o material usando solo genitivo | Fuera del selector; no especializar relación ni publicar respuesta. |
| V3 bind/consume positivo; múltiples preguntas de un origen; dos orígenes con texto igual | Referencia íntegra verificada y origen rígido, sin resolver subtipo. |
| Mutar T/H/modo/marker/artículo, docs/arrays/cierres/identidades/deudas, manifiesto/origen/entrega, añadir bindings | Consume rechaza; referencia desnuda y salida existente tampoco producen éxito. |
| Intentar nuevo testigo en ProtocolV1/V2 o composición léxica histórica | Rechazo local, sin reinterpretar otra forma; replays anteriores idénticos. |
| Renombrado consistente/reordenación, target cambiado, metadata/cierre cambiado | Firma estructural estable solo en los primeros; cambios semánticos o de incidencia detectados. |
| Agotamiento tras positivo, índice/copia/hashing/reclasificación, evidencias escalares y límites conjuntos | Error técnico de toda la operación, sin reserva sin cobrar ni output parcial exitoso. |

La crítica distinta debe comprobar estos contramodelos y el esquema, no dar por aprobado un raw porID. Después de su cierre, la implementación requiere controles breves proporcionales, verificación de rutas legacy y replays; no se ejecutan en esta propuesta. Una revisión manual, un mecanismo local correcto o una referenciaV3 consumible no demuestra extracción autónoma de116, utilidad frente a baseline, recursos/estabilidad o casos nuevos. Se preservan íntegramente resultados y dictámenes previos.
