# Calificación léxica de cláusula mediante context_bindings

Estado: diseño prospectivo, sin implementación, candidata, ejecución ni aceptación semántica. La reparación mínima propuesta reutiliza `Claim.context_bindings`: el propietario completo ya proporciona un host compuesto. No requiere una nueva variante de Body, entidad para la cláusula, envoltura, localizador arbitrario ni aristas recíprocas. Sí requiere ampliar el contrato cerrado y todas las rutas consumidoras que podrían borrar una condición.

## 1. Frontera y material

Se leyeron las dos críticas autorizadas de 2026-10-22, `nacar-source-design-v1/design-v2.md`, y exclusivamente la fuente actual Zinc de `relative-interval-host-design-v1/{candidate-input.json,design.md}`. Las otras dos frases solo se conocen por las críticas ya autorizadas: aquí no se propone repararlas. Se inspeccionó la interfaz de `validation.rs`, `validation/request_context.rs` y `request_context_profile.rs` antes de sus bloques de pruebas. No se leyó matching ni preguntas, contrapartes, gold, V6, resultados o nuevas revisiones de calidad. Para este cierre se leyó además la contracrítica autorizada `documentary-v3-lexical-clause-context-countercritique-2026-10-22.md`; sus cuatro precisiones se incorporan aquí como obligaciones prospectivas.

Incidencia de frontera: una lectura de infraestructura `context_budget.rs` mediante `cat` imprimió inadvertidamente su bloque `#[cfg(test)]`, con tres pruebas inline. También excedió la lista de archivos de interfaz estrictamente señalada. No se ejecutó ni utilizó ese bloque para justificar controles, fidelidad o resultados. Esta exposición impide declarar una frontera ciega estricta. No se ejecutó Rust, Cargo, CLI, API, campaña ni pruebas. Los controles que siguen son obligaciones de diseño, no resultados observados.

## 2. Contrato existente y coste real

La forma base de Claim es cerrada: `{id, scope_id, qualifier_ids, evidence, body}`. `validation.rs` admite además `context_bindings` cuando está presente `request_context_profile`; `request_context::contexts` valida sus miembros. La forma actual reconoce `nominal_during` y `normative_action_relation`; ninguna conserva por sí sola el valor léxico positivo de «en». Sus restricciones documentales no admiten el caso Zinc como una variante existente. Añadir una etiqueta sin modificar esas comprobaciones no funcionaría.

El perfil actual `documentary_request_context_v1` reconoce nueve features, incluida `relative_nonoccurrence_bound`; la referencia histórica a ocho está obsoleta. Se propone una décima feature optativa `lexical_clause_context`, conservando las nueve existentes y sus dependencias. No se introduce otro perfil de presupuesto ni se habilitan features ajenas para obtenerlo. El perfil actualizado y su lista efectiva siguen entrando al hash canónico de vocabulario. Ausencia de la feature conserva el comportamiento anterior; presencia de un contexto desconocido nunca autoriza quitarlo y reintentar como legacy. Un perfil mal formado es error, no desactivación silenciosa.

El host es la ocurrencia del Claim dueño, incluidos Body, calificadores internos y externos, scope y cierre efectivo. La propiedad de `context_bindings` ya establece composición estructural. El contexto no guarda otra copia del Body ni otro owner ID: la ubicación estructural decide el dueño. El identificador del contexto es local al owner; `(documento, claim_id, context_id)` identifica su incidencia. No constituye una referencia física ni una segunda raíz del grafo.

El cambio mínimo afecta validación de variante y admisión, habilitación/hash, cierre y rutas de selección/proyección/publicación. El grafo actual ya recoge referencias y qualifier_ids implícitos en el owner; debe verificarse que la variante nueva use esa misma extracción íntegra. Sin auditar consumidores no se puede prometer que solo haga falta cambiar el validador. Ese coste está pendiente; no se ha inspeccionado matching para anticiparlo.

## 3. Semántica y host Zinc

`LexicalClauseContext(H,P,C)` conserva que esta ocurrencia compuesta H está restringida contextualmente por C mediante la relación léxicamente anclada P. H con esta condición no equivale documentalmente a H desnudo. Cambiar dueño, complemento, preposición o frontera acreditada cambia su contenido. La ausencia de especialización conserva una condición positiva: no es metadata vacía, relación espacial, modo, condición temporal ni origen del intervalo. Una mención nominal no acredita existencia física, clase o identidad entre fuentes.

Para la única fuente actual, H es el Claim `shutdown` completo, con su átomo `action_undergoer_after_interval(subject,action_kind,interval)`, `fact/occurred/description` y polaridad positiva. Acción, sujeto e intervalo están correlacionados por ese mismo átomo; no se reconstruye una operación adicional. La cláusula externa [0,56) incluye la evidencia del host actual [0,47) y el sintagma [48,56). P es [48,50) «en» y C es [51,56) «Nacar». El punto final queda fuera de esa frontera sin contradecir contención: la evidencia actual del Claim y del átomo termina en47. Esta constatación no permite copiar ese intervalo a la reserva histórica.

La adscripción externa al Claim compuesto es un compromiso positivo. `whole_clause_only` no dice que se haya demostrado ambigüedad interna; dice que la representación no proyecta una adscripción más estrecha. `source_underspecified` añade la afirmación, que necesita revisión de fuente, de que la fuente deja indeterminada la adjunción interna dentro de ese mismo compuesto. Ninguna forma introduce un selector de subhosts ni un catálogo de lecturas. La diferencia no depende del deseo del consumidor. Una duda del autor se registra en aceptación pendiente, no se convierte en indeterminación fuente.

El origen temporal contextual no especificado pertenece a la semántica del predicado temporal actual y permanece intacto. El complemento léxico Nacar no lo rellena. El contexto original de esta ocurrencia es rígido, c0: designa el contexto fuente de ese Claim, no una variable existencial exportada, slot, alternativa o valor que pueda escoger el consumidor. Su contenido interno no determinado no autoriza sustituir c0 por el contexto conveniente de una petición. No se añade un campo seleccionable c0 ni un inventario de contextos. No se deriva encendido, reproducción, medición, autonomía o cien minutos de funcionamiento continuo. Tampoco se cambia el host actual por el antecedente histórico `duration_until_shutdown`: congelar un raw previo no lo vuelve autoridad semántica.

## 4. Wire cerrado propuesto

Notación de esquema, no una candidata. `ID` es string UTF-8 no vacío de hasta128 bytes; `SHA256`,64 dígitos hexadecimales ASCII minúsculos. `Evidence`, `Claim`, `Document`, `Vocabulary` y `Endpoint` conservan sus formas V3 vigentes. Todo objeto adicional es cerrado; claves duplicadas o adicionales, campos faltantes y referencias inválidas son errores. `Evidence` contiene exactamente `start_byte`, `end_byte`, `quote`, con fin exclusivo y coincidencia UTF-8 exacta.

```text
ClaimWithLexicalContext = Claim + {
  context_bindings: [LexicalClauseContext]
}

LexicalClauseContext = {
  id: ID,
  kind: "lexical_clause_context",
  preposition: Evidence,
  complement: {kind: "ref", id: ID},
  complement_evidence: Evidence,
  phrase_evidence: Evidence,
  external_clause_evidence: Evidence,
  internal_attachment:
    {kind: "whole_clause_only"}
    | {kind: "source_underspecified", boundary_evidence: Evidence[1..32]},
  specialization: null
}
```

La feature se incorpora a `request_context_profile.features` bajo la versión actual, cuyo máximo pasa de9 a10 y cuya whitelist añade exactamente `lexical_clause_context`. No se modifica el significado de ninguna feature anterior. La habilitación expresa una ampliación optativa del contrato, no un permiso para volver verdadero un adjunto. Un ejecutor antiguo rechazará la feature/forma nueva; esta incompatibilidad explícita es preferible a aceptar una lectura mutilada.

La preposición es una sola secuencia no vacía de letras Unicode, sin espacios ni puntuación; no se convierte a minúsculas ni se sustituyen lexemas por sinonimia. P y C son subintervalos no solapados, en ese orden, del sintagma; el sintagma cubre desde el inicio de P al final de C y entre ambos solo admite espacios Unicode. La evidencia de C debe ser la mención o nombre de ese referente local y esa ocurrencia, no un homónimo encontrado por nombre. El complemento conserva el referente completo y su cierre; esta versión solo admite ref nominal sin anáfora, binder, links o aliases de identidad. `described_ref` requiere otro alcance, nunca se desenvuelve para hacerlo pasar.

La cláusula externa contiene el sintagma y toda evidencia propia de Body, Claim y calificadores que semánticamente pertenecen al host. No se exige que contenga menciones de dependencias externas que tengan evidencia legítima en otra parte del documento; esas dependencias permanecen en el cierre. La adscripción externa requiere revisión semántica: la mera contención de bytes no la prueba.

## 5. Admisión, aceptación y procedencia separadas

La primera admisión operativa exige: documento source; owner de Body proposition; atom positivo; judgment fact; status occurred; basis description; normative_scope null; un solo contexto en ese owner; scope documental único sin parent, binder o branch; y cierre sin guards, antecedentes, alternativas, cuantificación, operadores negativos heredados o dependencias no resueltas. No se relajan esos límites para admitir hosts históricos con assignment. Otros claims independientes conservan su propio estado y no heredan el adjunto.

Formas fuera de ese alcance no se promueven a hechos positivos. El documento conserva su carrier, evidencia y deuda con resultado `unsupported_scope`; no hay fallback que consuma el Body desnudo. Errores de forma, IDs, ciclos reales, evidencia o referencias son errores de validación. Agotamiento de recursos es error de operación. Falta de crítica aceptada es aceptación pendiente. Son estados diferentes, sin conversión de errores en éxito parcial.

El complemento y host necesitan revisión fuente de su contenido, no solo validación estructural. El registro mínimo de revisión se cierra así; `Endpoint` usa los tipos y referencias V3 existentes y se resuelve exclusivamente en el documento derivado ligado:

```text
SemanticReviewRecord = {
  version: "lexical_clause_semantic_review_v1",
  source_input_sha256: SHA256,
  derived_input_sha256: SHA256,
  review_anchor_id: ID,
  owner_claim_id: ID,
  context_id: ID,
  reviewed_internal_attachment: "whole_clause_only" | "source_underspecified",
  host_fidelity: "accepted" | "pending",
  external_attachment: "accepted" | "pending",
  internal_attachment: "accepted" | "pending",
  complement_fidelity: "accepted" | "pending",
  incidences: ReviewIncidence[]
}
ReviewIncidence = {
  id: ID,
  affected_owner: Endpoint,
  affected_context_id: ID | null,
  origin:
    {kind: "coverage", owner: Endpoint | null, span: Evidence}
    | {kind: "review", evidence: Evidence[1..32]},
  disposition:
    {kind: "unresolved"}
    | {kind: "resolved", evidence: Evidence[1..32], justification: string}
}
```

`incidences` es el universo exacto de incidencias de esta revisión y documento, no una lista discrecional de vetos. IDs únicos/no vacíos de hasta128 bytes; lista de hasta100000 miembros dentro del límite global de nodos; justificación no vacía de hasta4096 bytes. Cada origen coverage localiza una única fila por coincidencia exacta de owner y span dentro del documento derivado. Si no existe o hay dos filas indistinguibles, el registro es inválido. El origen review identifica una deuda semántica adicional acreditada por la revisión, no inventada por el consumidor. `affected_context_id` no nulo debe resolver al contexto del affected_owner; no puede señalar otro Claim por reutilización de ID.

Toda fila coverage `unresolved` o `unsupported` del documento tiene exactamente una incidencia correspondiente; filas duplicadas de resolución son inválidas. Toda deuda semántica adicional vigente del universo de revisión autorizado tiene exactamente una incidencia review. El contexto confiable exterior liga el registro completo, incluido ese universo; omitir o añadir una deuda cambia el registro y no coincide con el autorizado. Una deuda afecta la selección si su owner pertenece al cierre efectivo, o identifica ese contexto; las restantes se conservan sin convertirse en veto global. Una incidencia unresolved efectiva impide el positivo. Los cuatro accepted y una deuda efectiva unresolved son incompatibles: registro inválido, nunca elegir el primer accepted. La modalidad interna debe coincidir exactamente con `reviewed_internal_attachment`, y la review se liga al documento derivado completo.

Resolver la fila actual de «en Nacar» exige `disposition=resolved` de esa incidencia exacta, con evidencia y justificación aceptadas contra esa versión; un accepted global no la borra. Se puede conservar la fila histórica unresolved en el raw y conservar en witness su resolución estructurada explícita. Alternativamente, corregir coverage requiere un delta declarado del raw y una nueva versión/revisión. Estas dos vías no se mezclan ni se normalizan como si fueran igualdad inversa. La ausencia fiel de especialización y el origen temporal no dicho no crean deudas artificiales cuando el alcance revisado conserva esa subespecificación.

La etapa autorizada de revisión proporciona el registro. `review_anchor_id` debe resolver, en el manifiesto/contexto confiable externo a la entrega, a la autoridad autorizada y al hash canónico exacto de ese registro, source y derived hashes, owner y contexto. El adaptador exige coincidencia; no cree autoridad por recibir un objeto accepted del mismo emisor de la candidata. No se añade servicio, firma o trust envelope. Hashes ligan versiones, nunca demuestran verdad, autonomía o ceguera. Si no hay ancla externa para la revisión, su aceptación sigue pendiente. Si hay ancla y el registro difiere, es error de revisión, no nueva autorización implícita.

La procedencia se comprueba separadamente contra un hash de input autorizado del manifiesto/contexto de campaña confiable y externo a la entrega. No se confía en dos hashes aportados por el mismo input. El adaptador usa bytes y contexto ya autorizados, conserva el hash calculado del input fuente y el hash derivado. Sin ancla externa, la procedencia no está acreditada. Integridad, procedencia y aceptación semántica son puertas independientes.

El delta admisible de vocabulario es únicamente añadir la feature y, si no existía el perfil, crear ese perfil con solo esa feature. Se comprueba igualdad estructural de todos los predicados, tipos, atributos, perfiles y relevancias anteriores. No se añade un perfil de presupuesto para conseguir admisión. Cambiar semántica temporal u otro vocabulario es otra revisión.

El primer alcance de adaptación exige owner base sin contextos previos: ausencia de `context_bindings` o array vacío conservado exactamente como se recibió. Un array vacío no se declara retrospectivamente válido si el contrato base lo rechaza; esa base requeriría corrección aparte antes de adaptar. Si contiene un contexto anterior, queda fuera de este alcance; no se reemplaza ni se borra. Los contextos de todos los demás owners permanecen idénticos. La inversión retira exclusivamente el miembro léxico añadido y reconstruye la presencia/ausencia y valor exactos del campo original, restaurando exclusivamente el hash de vocabulario anterior. Compara todo el raw resultante con toda la base autorizada, no solo Body. La variante de base ausente o vacía se obtiene de los bytes autorizados, no de una declaración del adaptador. Cobertura, evidencia y cualquier otra diferencia son deltas adicionales declarados y revisados; la resolución externa de una deuda no modifica esos bytes. No se afirma igualdad inversa si existe otro delta. Combinar contextos en un owner requiere otro alcance y conserva su caso/denominador.

La incidencia de coverage conserva también un owner nulo cuando así llegó la fila. Ese null localiza la fila por igualdad exacta de owner y span; no es un comodín. `affected_owner` sigue siendo un Endpoint válido del documento y su atribución forma parte de la revisión autorizada. Esta precisión evita cambiar el raw histórico para poder registrar una deuda.

## 6. Cierre, propiedad y límites

El cierre empieza en el owner completo. Sigue dependencias explícitas salientes e implícitas, términos por rol, referencias, scope, qualifier_ids interiores/exteriores y sus contenidos, links y aliases pertinentes. Añade el complemento del contexto como operando requerido del mismo owner. Cada endpoint se conserva completo, no solo su ID. Contexto y dueño forman un objeto calificado; no hay dependencia contexto→owner ni arista inversa owner→context serializada. Su pertenencia estructural no crea un ciclo.

La clave de visita incluye documento y tipo/ID; la incidencia del contexto incluye owner/context_id. El cierre se reconstruye, nunca se acepta una lista del autor que pueda omitir restricciones. La firma canónica conserva roles, correlación, lexema, complemento y contexto efectivo; es invariante ante renombrado coherente de IDs locales, no ante transferir la condición a otro owner. No se confunden documentos por namespace compartido ni se unen referencias por «Nacar» o «Zinc».

Se reutilizan las cotas de infraestructura actuales:100000 nodos, profundidad64,1000000 pasos y16MiB acumulados;512 owners,4096 evidencias y32 evidencias por lista. El primer carrier permite un contexto por owner, sin añadir límite de un único owner por documento. Las evidencias escalares nuevas también cuentan en el total4096 aunque no estén bajo una clave `evidence`; esa ampliación de contabilización es obligatoria. Cada visita, búsqueda, comparación, canonicalización y copia se precobra; adaptadores anidados no reinician el contador. El mismo contador cubre intent y revisión completos, sus evidencias/Terms, resolución de refs y de todas las deudas, comprobaciones de anclas externas, hashes, ordenación, cálculo de tamaños y copias. `boundary_evidence`, evidencia de revisión y evidencias dentro de Terms del intent cuentan en las cotas32 por lista y4096 globales. El recorrido para medir tamaño también se cobra antes de copiar: no hay preflight gratuito ni presupuesto separado por resultado. Materializar documento, cierre, intent, revisión y testigo paga cada copia. Agotamiento invalida toda la operación; no se devuelve un testigo truncado. Estas cotas son decisiones de infraestructura reutilizadas, no evidencia de RSS, latencia o calibración.

## 7. Consumo parcial seleccionable con condiciones íntegras

Una salida contextual que nadie puede seleccionar no cierra la utilidad. Se define un consumo parcial preciso y restringido, sin leer ni generar preguntas concretas. La entrada es una selección de evidencia, sin slots de respuesta:

```text
EvidenceSelectionIntent = {
  version: "lexical_clause_evidence_selection_v1",
  derived_input_sha256: SHA256,
  predicate: ID,
  grounded_role_constraints: [{role: ID, term: Term}][1..N]
}
```

N no excede los roles declarados del predicado. Cada role es único, existe en esa firma y aporta un término cerrado compatible con su tipo. Todos los términos, refs, evidencias y roles se validan exclusivamente contra el documento derivado cuyo hash exacto figura en el intent. Las refs son localizadores de referencias de ese documento: no prueban identidad externa. Al menos una restricción es una ref nominal local resuelta, sin anáfora ni binder. Un ID de otro universo o un hash de otro documento es intent inválido, nunca coincidencia por nombre/ID ni no coincidencia semántica. La selección entre documentos queda fuera del contrato.

En este alcance Term admite únicamente `ref`, `symbol`, `quantity`, `time` y `text` con las formas/tipos vigentes y evidencia exacta local cuando corresponda. Excluye `described_ref`, variables, refs fuera del documento y anáforas no resueltas; no abre razonamiento de descripciones. Todas las restricciones son conjuntas y cerradas. No hay roles abiertos, alternativas, proyecciones ni campos que elijan c0 o interpreten la indeterminación interna. IDs locales de aliases del host solo pueden coincidir mediante el componente de identidad ya admitido, conservando todas sus restricciones y pagando su cierre; no se generan identidades entre documentos.

El owner es seleccionable si su predicado coincide exactamente con el solicitado, sus roles y tipos son válidos y todas las restricciones aportadas coinciden mediante las reglas de término/identidad locales ya admitidas, conservando sus restricciones. Para symbol se exige ID idéntico del vocabulario; para quantity decimal, unit y bound idénticos; para time value y format idénticos; para text valor idéntico. Se validan las evidencias sin usar diferencias de sus offsets para fabricar o negar identidad semántica. No hay conversión de unidades, sinonimia ni equivalencia nueva. No se añade equivalencia entre `action_undergoer_after_interval` y otro predicado más débil. En Zinc, una selección del propio predicado correlacionado y un localizador de sujeto del propio documento puede recuperar el compuesto completo, incluyendo acción, intervalo y condición «en Nacar». Si se aporta una cantidad o acción incompatible, no selecciona. Esto es compatibilidad para recuperar evidencia parcial, no acreditación de valores omitidos ni contestación de una pregunta externa.

El resultado positivo necesita forma validada, admisión, procedencia acreditada y revisión semántica aceptada de ese owner/contexto. No necesita resolver el sentido especializado de «en» ni elegir una adjunción interna. Devolver el contexto íntegro es precisamente el consumo positivo de la restricción. Una demanda externa de lugar, modo, origen temporal o lectura interna concreta no queda satisfecha por esta operación y no descarga ninguna condición.

```text
LexicalClausePartialWitness = {
  version: "lexical_clause_partial_witness_v1",
  source_input_sha256: SHA256,
  derived_input_sha256: SHA256,
  domain_namespace: ID,
  owner_claim_id: ID,
  context_id: ID,
  source_text: string,
  vocabulary: Vocabulary,
  document: Document,
  owner_claim: ClaimWithLexicalContext,
  effective_closure: Endpoint[],
  selection: EvidenceSelectionIntent,
  matched_roles: ID[],
  semantic_review: SemanticReviewRecord,
  status: "selected_partial_contextual_evidence",
  answer_terms: [],
  answer_propositions: [],
  full_answer_supported: false,
  full_text_coverage_claimed: false
}
```

Documento, owner, vocabulario, review y cierre son salidas comprobadas del mismo cálculo, no depósitos independientes: owner es exactamente el Claim del documento; review referencia esos hashes e IDs; cierre único ordenado por tipo/ID es exactamente el reconstruido. Texto y vocabulario permiten verificar citas y semántica declarada del predicado. `matched_roles` contiene exactamente los roles de la selección, ordenados; no presenta condiciones no evaluadas como coincidencias. La copia íntegra conserva coverage, issues y claims independientes, sin declararlos todos miembros del host ni todos aceptados. La selección no altera texto, raw o coverage.

No es una full respuesta disfrazada: conserva un único compuesto fuente relevante, con condiciones y deudas completas, pero no produce valores de respuesta. A la vez, no es bloquear todo: la condición léxica aceptada permite seleccionar y mostrar esa evidencia contextual junto con su proposición correlacionada. La posibilidad y utilidad están especificadas; su funcionamiento sigue pendiente de implementar y verificar. La unión completa de resultados es cerrada; solo la variante selected contiene witness. Los conjuntos de IDs son únicos, ordenados lexicográficamente, no vacíos donde se indica; `effective_incidence_ids` apunta al universo exacto de SemanticReviewRecord, nunca a otra revisión.

```text
SelectionResult =
  {kind: "selected", witness: LexicalClausePartialWitness}
  | {kind: "no_match", derived_input_sha256: SHA256,
     owner_claim_id: ID, context_id: ID,
     reason: "predicate_mismatch" | "role_constraint_mismatch"}
  | {kind: "pending", derived_input_sha256: SHA256,
     owner_claim_id: ID, context_id: ID,
     reasons: PendingCode[1..3], effective_incidence_ids: ID[]}
  | {kind: "unsupported", derived_input_sha256: SHA256,
     owner_claim_id: ID, context_id: ID,
     reason: "unsupported_scope"}
  | {kind: "error", category: "structural" | "operation",
     code: ErrorCode}
PendingCode = "unverified_provenance" | "unverified_review" | "semantic_pending"
ErrorCode = "invalid_document" | "invalid_intent" | "invalid_review"
          | "invalid_provenance" | "request_context_limit"
```

Esta unión no añade otro sistema de reportes: es el resultado mínimo de la selección especificada; issues y coverage V3 permanecen en el documento. Las variantes no selected no contienen respuestas, valores inferidos ni witness parcial truncado. El pending conserva dueño/contexto por sus localizadores ligados al documento de la operación y las incidencias efectivas; si falta review autorizada, la lista de IDs queda vacía y `unverified_review` impide fingir que se calculó un universo aceptado.

Precedencia obligatoria: agotamiento del contador produce error operation/request_context_limit y domina cualquier resultado; las demás categorías de ErrorCode son structural. Tras validar forma de documento, vocabulario, intent y registros/anclas aportados, cualquier contradicción o invalidez estructural produce error. Si hay varias, se informa por orden invalid_document, invalid_intent, invalid_provenance, invalid_review. Una ancla ausente es pendiente; una ancla existente que contradice bytes/hash/registro es error. Luego, una forma válida fuera del alcance produce unsupported; para la forma admitida, cualquiera de las tres puertas pendientes produce pending con todos los PendingCode aplicables ordenados, antes de considerar coincidencia. `semantic_pending` incluye estados pending o incidencias efectivas unresolved en una review autorizada y coherente. Solo con validación y puertas completas se evalúan restricciones: discrepancia de predicado produce no_match/predicate_mismatch; si coincide predicado pero falla una restricción, no_match/role_constraint_mismatch; si coinciden todas, selected. No se usa no_match para refs inválidas, universo desconocido, deuda o falta de autorización. Todas las rutas y sus copias pagan el mismo presupuesto y conservan denominadores.

## 8. Legacy, proyección y deudas independientes

Antes de cualquier selección ordinaria, un owner con esta feature se trata como compuesto calificado. Las rutas que no transporten su condición no pueden consumir su Body para respuesta, exportación, índice semántico, relevancia, conclusión existencial o witness abreviado. Deben conservarlo íntegro como pendiente para esa ruta. Tampoco basta añadirlo al DTO final después de haber tomado una decisión con Body desnudo. Una proyección más débil exige regla explícita distinta, marcado de cobertura parcial y revisión propia; este diseño no la habilita. Los claims independientes sin este carrier siguen sus reglas, sin transmisión de contexto por compartir sujeto.

Los contramodelos de diseño incluyen borrar el contexto, cambiar `en` por otro lexema, sustituir el referente, transferirlo a otro Claim, separar intervalo/acción, reinterpretar Nacar como origen temporal y consumir el Body mediante legacy. Todos cambian contenido o pierden una condición y deben impedirse. También debe existir el caso positivo de selección por proposición y ancla con testigo completo; una batería que solo rechace todo no demuestra implementación útil. Estos son requisitos, no tests ejecutados.

La reserva permanece fuera de esta propuesta hasta su crítica fuente específica; no se da por resuelta su frontera interna ni «nueva». El funcionamiento de Cobre, la semántica de «sin apagarse» y el límite relativo de la medición negada conservan sus deudas independientes. La corrección temporal actual Zinc conserva su propio estatuto de propuesta: su hash, carrier y lectura de interfaz no prueban fidelidad aceptada, temporalidad ejecutable o extracción automática. La puerta116 permanece íntegra: no se eliminan casos, denominadores, errores ni formas no soportadas; no se reclasifica bloqueo como acierto.

La brecha real remanente no es propiedad del host: Claim ya la proporciona. Es la obligación transversal de conservar y consumir la condición, revisar su fidelidad y ofrecer la selección parcial especificada. La ampliación de una unión cerrada y del permiso es pequeña; demostrar que ninguna ruta borra la condición requiere trabajo adicional acotado pero todavía no realizado. Si una ruta actual no permite transportar el Claim calificado íntegro, esa ruta sigue sin soporte hasta corregirse; no justifica añadir por defecto un Body wrapper ni prometer soporte sin inspección. Este documento entrega únicamente ese diseño y sus límites.

## 9. Costura CLI autorizada y archivos exactos

La única costura de implementación aprobada es el comando experimental del evaluador `select-documentary-lexical-context-v1 INPUT TRUST_MANIFEST OUTPUT`. El runner suministra separadamente TRUST_MANIFEST; la candidata no elige su ruta ni autoridad. No se añade servicio, endpoint, contrato de producto ni sistema de confianza. El comando lee ambos archivos y escribe exactamente un `SelectionResult` completo en OUTPUT, sin modificar entradas. Un fallo técnico de lectura/escritura se informa como error CLI, sin afirmar que exista un OUTPUT válido.

Wire exacto aprobado; todos los objetos son cerrados y se rechazan claves duplicadas:

```text
Triple = {vocabulary: Vocabulary, text: string, raw: Document}
INPUT = {
  version: "lexical_clause_selection_input_v1",
  base: Triple,
  derived: Triple,
  owner_claim_id: ID,
  context_id: ID,
  domain_namespace: ID,
  selection: EvidenceSelectionIntent,
  semantic_review: SemanticReviewRecord | null
}
TRUST_MANIFEST = {
  version: "lexical_clause_trust_manifest_v1",
  authorized_source_input_sha256: SHA256 | null,
  review_anchors: [{
    review_anchor_id: ID,
    review_sha256: SHA256,
    source_input_sha256: SHA256,
    derived_input_sha256: SHA256,
    owner_claim_id: ID,
    context_id: ID
  }]
}
```

SHA de cada Triple y del registro de revisión significa SHA256 del JSON canónico del evaluador: serialización compacta serde de un valor JSON con claves de objetos ordenadas lexicográficamente, listas con orden conservado, strings intactos y sin normalización de valores. Los hashes fuente en este contrato identifican la triple base canónica, no el archivo físico con sus blancos. La preservación inversa es igualdad estructural exacta de la triple y presencia/ausencia de campos, no igualdad de formato del archivo. Este cierre sustituye cualquier lectura anterior que exigiera conservar blancos físicos. No hay campos autorreferenciales: derived hash cubre solo INPUT.derived, sin intent ni review.

Se exige igualdad de textos, namespace de base/derived e INPUT.domain_namespace; `raw.text_sha256` y `raw.vocabulary_sha256` se verifican por sus contratos vigentes. La base se contrasta con la inversión y delta de vocabulario de sección5. Su hash canónico calculado debe coincidir con authorized_source_input_sha256 si no es null. El hash derivado calculado debe coincidir con selection.derived_input_sha256 y con los hashes de semantic_review si existe. owner/contexto localizan un único Claim/contexto del derivado y coinciden con los del registro y ancla que se utilicen.

El manifiesto es confiable por el contexto externo del runner que lo proporciona; el nombre del archivo no autentica su contenido. `review_anchors` contiene exactamente registros autorizados por la etapa de revisión del runner: IDs review_anchor_id únicos/no vacíos y hasta100000 entradas, sometidas además al límite global de nodos y presupuesto. No se necesita authority_id ni otro servicio. La autoridad se comprueba resolviendo review_anchor_id exclusivamente en ese manifiesto externo, nunca en un objeto que la candidata incorpore a INPUT.

Si authorized_source_input_sha256 es null, procede unverified_provenance, nunca positivo; si es no nulo y contradice el hash base, invalid_provenance. Si semantic_review es null o su review_anchor_id no tiene ancla, procede unverified_review. Si tiene ancla, SHA canónico del registro completo debe igualar review_sha256 y todos los source/derived hashes, owner/contexto deben coincidir con el cálculo y el ancla. Una contradicción produce invalid_review. El hash de registro incluye su universo exacto de incidencias/resoluciones; copiar un ID o accepted no transfiere autoridad. El manifiesto no certifica verdad o ceguera por llevar hash: acredita que esta versión exacta recibió esa revisión autorizada.

Un manifiesto mal formado, IDs duplicados o hashes inválidos produce invalid_provenance estructural si puede escribirse resultado; imposibilidad técnica de leerlo es error CLI. La semántica de ancla ausente y contradicción es la de sección7. La operación completa incluye INPUT, base/derived, manifiesto, intent, revisión, hashes, búsquedas de anclas, inversión, resolución de deudas y OUTPUT en el mismo contador. Ninguna fase reinicia presupuesto. Todos los IDs respetan128 bytes; evidencias y tamaños respetan las cotas anteriores. Esta sección fija la costura mínima para la implementación separadamente autorizada, sin afirmar que el comando exista o haya funcionado.
