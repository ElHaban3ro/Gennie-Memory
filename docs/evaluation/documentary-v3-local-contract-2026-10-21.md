# Contrato local de representación documental V3

Estado: implementación experimental en el evaluador; verificación de comportamiento y cobertura del dominio registradas por separado. No está integrada en producto ni sustituye baseline. La [especificación prospectiva revisada](../../outputs/iromem/evaluation/representation-v3-design-2026-10-21/proposal-revised3.json) y sus críticas se conservan.

## Interfaz pura

`validate(raw, text, vocabulary)` devuelve `DocumentResult`. Solo el validador puede construirlo: se serializa para inspección, pero no se deserializa un informe externo para usarlo como validado. Comprueba forma cerrada, hashes de texto/vocabulario, evidencia por intervalos UTF-8, referencias, ámbitos, dependencias y restricciones. La validez estructural no demuestra fidelidad ni cobertura semántica frente al texto.

`classify_pair(source_result, question_result, vocabulary, pair_scope)` compara resultados producidos por el validador y expone witnesses, faltantes, conflictos y candidatos pendientes por solicitud. No ejecuta extracción ni usa gold, proveedor, SQL, servicios o estado de producto.

El vocabulario tiene seis claves básicas operativas: `version`, `predicate_rows`, `fixed_attribute_rows`, `fixed_action_rows`, `semantic_type_rows`, `relevance_links`. Cada predicado declara mapa de roles y clases de término; atributos y acciones son IDs técnicos genéricos. Tipos semánticos y enlaces de relevancia se declaran explícitamente. El SHA-256 se calcula sobre JSON UTF-8 compacto, con claves de objetos ordenadas y orden de listas conservado; no se incluye un hash autorreferencial. Unidades literales no habilitan conversiones. Las notas laterales no añaden reglas ejecutables al vocabulario.

La versión de documento es `documentary_evidence_v3`. `pair_scope` tiene `namespace`, `configuration`, `time_selector` y `closed_world:false`. La igualdad operativa de nombres exige el namespace declarado compatible; no es prueba general de identidad real.

## Adaptador de archivos

- `validate-documentary-v3 <entrada.json> <salida-nueva.json>` recibe `{vocabulary,text,raw}` y escribe el informe estructural.
- `classify-documentary-v3 <entrada.json> <salida-nueva.json>` recibe `{vocabulary,source:{text,raw},question:{text,raw},pair_scope}`. Valida ambos lados dentro de la misma operación; no acepta informes externos como sustituto de raw/texto. Escribe ambos informes y la comparación.

Los objetos de entrada son cerrados; la salida requiere archivo nuevo y no sobreescribe uno existente. El adaptador limita la entrada a 16 MiB y sincroniza la salida. Las ejecuciones son locales breves sin puerto ni peticiones a proveedores. Un fallo técnico de lectura/forma de envoltura es error CLI; un documento inválido aparece como tal en el informe, sin convertirlo en abstención fiable.

Los reportes de pares declaran que no se midió extracción automática, no se probó fidelidad semántica y no se midió latencia de producción. La representación raw de entrada se conserva en el archivo original; el resultado puede incluir documento o candidatos/errores para diagnóstico.

## Invariantes de evaluación

La ausencia de match no prueba desigualdad. Un conjunto cerrado solo excluye un valor cuando la diferencia está demostrada frente a cada miembro. Ningún guard de una rama alternativa o claim pendiente se descarga como hecho cierto. Las ramas heredan sus restricciones y un invariante conserva juicio, estado, alcance y calificadores completos.

Un identificador nuevo no prueba dos objetos distintos; cardinalidad del propietario no es cardinalidad del objeto. Permiso, obligación, prohibición y ejecución permanecen separados y con ámbito normativo explícito. Un resultado parcial fiel no completa otros slots ni autoriza rellenar unidades, clases, métodos, fechas o condiciones ausentes.

El vocabulario global inválido o su hash incoherente invalida la operación. Una fila con vocabulario desconocido conserva su candidato pendiente y los dependientes; no borra hechos independientes válidos. La propagación debe atribuir errores tanto por pointer de fila como por ID del grafo.

Las condiciones normativas son proposiciones estructuradas; no se publican como JSON disfrazado de valor textual citado. Cuando una salida necesita condiciones completas, `answer_propositions` conserva su estructura además de los términos de respuesta. La calificación distingue requisito histórico de estado efectivo y no transforma una dispensa en permiso global.

## Puertas que siguen abiertas

El inventario conocido incluye los 116 textos. Sus versiones y críticas no son etiquetas nuevas ni prueba de extracción. Los predicados neutrales, disponibilidad documental, acciones sin agente, requisitos de clase/atributo, máximos, tiempo parcial y vínculos de evidencia parcial requieren preflight operativo antes de otra extracción completa; no se arreglan rellenando argumentos inventados.

Las pruebas sobre representaciones explícitas verifican el mecanismo, no la capacidad del modelo para extraerlas. Se mantienen los cinco objetivos: extracción/revisión completa, conservación de utilidad en todos los pares previstos, comparación real con baseline, recursos/estabilidad y casos nuevos ciegos antes de cualquier adopción. Una limitación se informa con su denominador, sin excluir casos para mejorar cifras.


Verificación de esta implementación: [resultado y límites](documentary-v3-implementation-results-2026-10-21.md). Las filas de `semantic_type_rows` requieren citas de clase no vacías; tipos de cantidad/tiempo no autorizan por sí solos una clase. Ambos perfiles completos iniciales incumplen este requisito y se conservan inválidos; el control mínimo CLI usa un vocabulario propio, explícitamente manual.


## Extensión operativa posterior al primer preflight

Para proposiciones, los slots `holds_year`, `scheduled_year`, `planned_year` y `occurred_year` solicitan el año del rol temporal correspondiente; para eventos se admiten los tres últimos. El término fuente necesita `format:year`, cuatro cifras ASCII y evidencia exacta de esas cifras. Una fecha completa, día/mes o expresión relativa permanece parcial; no se convierte ni se rellena año. Los slots anteriores `*_at`/`planned_for` conservan tiempo general y no reparan una pregunta mal representada por el extractor. Los snapshots vinculan código, vocabulario, entradas y reportes: el hash del vocabulario no sustituye el del código operativo.

En solicitudes de valor, las cantidades no exactas no completan el slot, conservando cota y unidad. Varias ocurrencias temporales supervivientes requieren desambiguación; las citas de una regla y los miembros de un invariante demostrado no son ocurrencias adicionales. Las rutas que publican evidencia rechazan contexto demostrado distinto. En relevancia estricta, aplicabilidad desconocida o fundamento incompatible conserva candidatos pendientes. Las extensiones posteriores de contexto documental y relevancia calificada permiten transportar únicamente la estructura fuente con faltantes explícitos bajo sus propios controles; no prueban la respuesta solicitada. Su utilidad global debe comprobarse antes de adoptar.

El domain3 superó forma global en CLI. Su primera matriz manual incompleta queda como histórica; el seguimiento actual corrige nueve carencias y conserva dos fuentes estructuralmente parciales, sin acreditar extracción automática ni utilidad completa. [Resultados](documentary-v3-domain-preflight-results-2026-10-21.md).


## Perfil opcional de atributo y valor

Una séptima clave `attribute_profiles` admite `{version:"documentary_attribute_profiles_v1",targets:[...],rows:[...]}`. Los targets permitidos son `quantity.amount`, `property.value` y `event.result`. Cada fila tiene exactamente `attribute_id`, `target`, `term_kinds` y `quantity_units`; tipos de término y unidades se declaran como listas finitas únicas. Los IDs de atributo deben existir, y el portador/rol debe ser válido. El perfil completo se incluye en el hash del vocabulario: no es un sidecar informativo.

Un target omitido no queda comprobado por este perfil. Para targets declarados, una fila ausente o un tipo/unidad incompatible deja pendiente el valor y dependientes. Un perfil mal formado invalida globalmente el vocabulario. Las restricciones cerradas comprueban cada `members[i]`; una solicitud con unidad explícita se comprueba incluso si omite el valor pedido. La comparación excluye claims pendientes por perfil de una vista de ejecución privada, conservando el raw y el informe original. No pueden satisfacer guards, combinar eventos ni formar invariantes; otras evidencias independientes siguen disponibles. Requests pendientes no seleccionan evidencia.

No hay conversión de unidades, validación del significado de una cita, tabla completa de77atributos ni cobertura implícita de otras aridades. Los vocabularios sin perfil mantienen compatibilidad histórica, sin promesa nueva de tipado semántico. [Resultados y límites](documentary-v3-known-lowering-results-2026-10-21.md).

En documentos de pregunta, los claims referidos por `premise_ids` son premisas hipotéticas corregibles, no hechos fuente: el mecanismo exige evidencia independiente en el documento fuente y conserva refutaciones como tales. La categoría `judgment:fact` distingue la forma de la proposición de un permiso/obligación; no prueba la verdad de una premisa del usuario.


## Patrones locales de respuesta y cierre de nueve representaciones

`class_filter:{kind:"reference_pattern",reference_id:"..."}` apunta a una referencia local de pregunta, anónima, sin anáfora ni binding de existencia. Solo se admite en `mode:answer`, para un slot de tipo ref omitido del átomo. La referencia no puede aparecer como hecho fuente de la pregunta ni en una identidad afirmada. El grafo valida referencias, ámbito y dependencias, incluido el nuevo `reference_id`. Los eventos no admiten este filtro en esta versión.

El patrón comprueba clase/tipo y vínculos direccionales conjuntamente. En los cuatro casos de red usa `part_of`; no equivale a `member_of`. La falta de prueba permanece pendiente. Se aplica también a witnesses derivados de restricciones e invariantes; los miembros acreditados de un conjunto mixto permanecen parciales. Las vías explícitas de relevancia parcial y corrección de premisa conservan su significado separado.

La correspondencia acotada de dominios universales no prueba identidad de entidades. Una cantidad exacta literal constante bajo el mismo dominio universal puede satisfacer la condición de cantidad común; un individuo, existencial, guard pendiente o rama no basta. No se igualan automáticamente políticas o grupos anónimos.

`class_suffices(subject,class)` conserva suficiencia normativa literal con beneficiario y ámbito; no ejecuta una regla ni concluye permiso, requisito necesario o propósito ausente. Las posibles correcciones de premisa con alcance pendiente se conservan como parciales, sin dato solicitado inventado. [Cierre, ejecución y límites](documentary-v3-nine-cases-results-2026-10-21.md).


## Asociación contextual restrictiva

`references[].links[].kind` admite además `associated_with`, con target local y evidencia literal como los demás vínculos. Es dirigido, débil y restrictivo; no implica propiedad, ubicación, membresía, composición, simetría ni transitividad. Un genitivo ambiguo puede conservarse así sin reforzar su significado. Compartir contexto no prueba identidad global de entidades anónimas.

Las asociaciones requeridas por una pregunta deben acreditarse antes de publicar respuestas directas, de eventos, restricciones, condiciones necesarias o dispensas. Sus alias locales conservan las restricciones. Incompatibilidad no probada queda desconocida y no demuestra diferencia. Anclas dependientes de calificadores/claims quedan pendientes en esta versión.

Los patrones descriptivos locales pueden corresponder recursivamente con asociaciones anónimas, sin convertirlas en identidad global. Exigen igual denotación (entidad/clase/grupo), clase/tipo declarados y vínculos compatibles; los anclajes nombrados requieren identidad documental. Ciclos y límites de profundidad quedan desconocidos. La igualdad estricta de conjuntos contextuales es conservadora y puede perder utilidad; no se ha aprobado globalmente.

La guía V2 precisa selección de los mismos IDs de vocabulario; su hash no sustituye al código que incorpora el nuevo tipo de vínculo. [Pruebas y límites](documentary-v3-association-guide-results-2026-10-21.md). La [extracción anterior](documentary-v3-frozen-extraction-results-2026-10-21.md) se conserva fallida y no utiliza esta revisión.


## Anclaje nominal de respuestas parciales

Las respuestas parciales con referentes nombrados competidores necesitan un nombre documental compartido a través de alias locales del mismo ámbito de comparación. Los vínculos de esos alias siguen siendo restricciones. Un nombre sin correspondencia conserva candidato pendiente y no publica su dato como respuesta seleccionada; no prueba desigualdad. Clase desconocida bajo nombre acreditado sigue siendo parcial.

La comparación de verdad por exclusión explícita de un conjunto cerrado mantiene su vía de prueba por diferencia. No se fuerza un nombre sobre una variable universal o referencia anónima: permanece el tratamiento parcial previo, sin prueba nueva de aplicabilidad o identidad. La revisión no resuelve identidad anónima, todos los selectores o alcance universal. [Diagnóstico completo y límites](documentary-v3-known100-utility-results-2026-10-21.md).


## Contexto documental declarado

Una relación explícita de `relevance_links` desde `documentary_availability` o `information_state` a `property`/`quantity`, con bindings sujeto→sujeto y atributo→atributo y `source.state` en carry_roles, habilita contexto documental para slots value/amount. El orden de los dos bindings no cambia su significado. Sin esa declaración se mantiene la abstención anterior. No hay predicados/atributos nuevos ni inferencia transitiva.

Se conserva `partial_relevance` con `answer_terms:[]` y la proposición fuente completa en `answer_propositions`, incluido document_scope cuando exista. No se afirma inexistencia, cero o respuesta completa. Clase no acreditada con nombre compartido conserva identidad pendiente; anclas anónimas sin identidad no se igualan por descripción. Contexto, ámbito normativo, dependencia, ramas, fundamento y premisas deben pasar. Se ignoran unidades/filtros del valor pedido porque no se publica ese valor; permanecen sin satisfacer en answer_slot. [Evidencia y límites](documentary-v3-documentary-state-results-2026-10-21.md).


## Proyección optativa de asignaciones desde usos

`assignment_projection:{version:"uses_qualifier_assignment_v1",context_keys:[...]}` es una clave opcional cerrada del vocabulario. Requiere claves únicas no vacías y los roles exactos de uses(subject:ref,object:ref) y quantity(subject:ref,attribute:attribute_id,amount:quantity). Ausencia mantiene comportamiento anterior; perfil inválido invalida el vocabulario global.

La versión declara asignación para usos positivos fact/state con basis description/assignment y una cantidad conocida en un calificador configuration del mismo claim, cuya key iguala el atributo pedido. La proyección no interpreta admisión, permiso, plan o negación como uso. Conserva sujeto, setting, cantidad, unidades, cotas y evidencia original; contextos restantes, ámbitos, dependencias, ramas y premisas deben resolverse. La petición debe ser positiva, omitir amount y tener exactamente una etiqueta de configuración conocida de las claves declaradas.

Coincidencia de nombre de ref y texto solo establece etiqueta local de configuración, no identidad global. Bajo contexto coincidente, scoped_assignment puede aportar la cantidad con basis assignment, manteniendo cualquier falta de identidad/clase/filtro. Bajo otra configuración solo se conserva partial_relevance con answer_terms vacío y la proposición original; no se refuta la posibilidad de otros usos. Una coincidencia válida del modo solicitado descarta contrastes adicionales de esta vía. Cantidades no equivalentes de la misma coordenada bloquean completitud con conflicto potencial/assignment_occurrence. Esta regla semántica nueva aún requiere revisión independiente y extracción real antes de adopción. [Prueba y límites](documentary-v3-scoped-assignment-results-2026-10-21.md).


## Corrección de premisa con anclaje nominal pendiente

Una negación de premisa con predicado/roles iguales y todos los términos no referenciales acreditados puede conservarse como partial_premise_refutation cuando referencias aún desconocidas comparten nombres/alias documentales y vínculos compatibles. No prueba clase o identidad; conserva identity, premise y answer_slot pendientes, candidato sin resolver y proposición fuente original. No publica fecha/valor ni demuestra la corrección completa. Otros actores/objetos y términos no referenciales distintos se rechazan.

## Cautela explícita de admisión

Un relevance_link admits→quantity con binding subject→subject y carry_roles que contiene source.object, junto al perfil existente assignment_projection/context_keys, habilita cautela para una petición positiva quantity.amount de actual assignment. Se transporta una admisión negativa del modo solicitado o una restricción cerrada **positiva** sobre admitidos con membresía no acreditada. Coincidencia con un miembro/alias no se trata como exclusión; nombres distintos no prueban Difference. No se usa una lista de no admitidos como lista de admitidos.

Solo se produce partial_relevance sin answer_terms, con admission_context.actual_use_inferred=false y source proposition/admission_constraint originales. Admisión no satisface basis assignment. Contexto restante, ámbito, dependencia, rama o premisa pendientes bloquean esta cautela; no se borran calificadores numéricos/de modo para aprobar una lista cerrada. Positivo/abierto no establece uso. [Evidencia y límites](documentary-v3-correction-context-results-2026-10-21.md).


## Dato común y alcance universal

Las vías positivas answer/event_slot/constraint con configuration conocida quantity_extent y valor textual «común» requieren correspondencia explícita de los binders forall pertinentes mediante alpha_map. Además, cada operando fuente en la posición universal pedida debe ser la variable mapeada o un alias local acreditado: un cuantificador envolvente con operando constante no basta. Se comprueban argumentos, sujetos/eventos, ámbito normativo y selectores referenciales de método/tiempo cuando proceden. Sin variable pertinente o prueba admitida se conserva candidato common_quantity_scope, sin publicar valor ni probar Difference.

Una pregunta por valores individuales sin ese operador conserva datos parciales de miembros. Las vías de premisa opuesta y contexto siguen separadas. No se resuelve aquí cobertura general por enumeración ni eliminación de cuantificadores adicionales; el operador es una declaración estructurada existente, no reconocimiento automático de lenguaje natural. [Controles y límites](documentary-v3-universal-scope-results-2026-10-21.md).


## Relevancia declarada con identidad pendiente

En un enlace de relevancia de un salto ya declarado, una proposición positiva fact/state puede conservarse ante una pregunta answer/fact si todos los bindings son estrictamente iguales o descriptivamente compatibles y al menos uno tiene anclaje nominal acreditado. Para un binding referencial desconocido: nombre compatible mediante alias local con partial_named_anchor, o ambas referencias anónimas con answer_reference_pattern completo; clases/tipos explícitos distintos y anáforas bloquean esta vía. No se cambia la identidad global ni se equiparan individuos por clase.

Esta vía conserva controles de contexto, asociaciones requeridas en los bindings declarados, ámbito, dependencias, rama, fundamento y premisas. Los argumentos de destino no ligados se mantienen pendientes según la extensión siguiente. Solo transporta la proposición original con answer_terms vacío, context_identity_proven=false y candidato con identidad/slot pendientes. No publica el origen pedido ni demuestra ausencia de red. Los caminos estrictos conservan su comportamiento. [Medición y límites](documentary-v3-qualified-relevance-results-2026-10-21.md).


## Contexto parcial de restricciones cerradas y bindings declarados

Una restricción fact/state positiva, cerrada, no vacía y con existencia afirmada puede aportar contexto mediante un enlace de relevancia declarado. Se conserva source_constraint íntegra con answer_terms/answer_propositions vacíos; el answer_role variable no puede participar como binding fijo. No se afirman miembros individualmente ni se transforma cierre en ausencia de otra alimentación. Caminos reservados de admisión y estado documental siguen separados.

En relevancia calificada se comprueban los roles mapeados por el enlace con sus patrones/anclajes, no todos los argumentos de la pregunta bajo iguales nombres en otro predicado. Los roles no ligados quedan explícitos como unbound_target_roles/unbound_target_context y no se infieren. Contexto, ámbito, dependencia, rama, fundamento y premisas siguen condicionando selección. Se declaran dos enlaces parciales dirigidos del dominio: receives_from→upstream_origin y supplied_by→receives_from, con destino ligado; no son equivalencias ni autorizan inversión/transitividad. La pertinencia fuera de los controles conocidos, especialmente otros bienes o sustancias, requiere revisión adicional. [Medición y límites](documentary-v3-supply-context-results-2026-10-21.md).


## Perfil optativo de aplicabilidad de relevancia

relevance_profiles:{version:"documentary_relevance_profiles_v1",rows:[...]} añade condiciones explícitas por link_id, sin modificar los enlaces ni la configuración de producto. Las filas, alternativas y roles son cerrados y sin duplicados. Ausencia preserva la semántica anterior; perfil mal formado invalida globalmente el vocabulario.

class_context exige clases declaradas fuente/objetivo de las listas configuradas; es una declaración de contexto, no identidad ni ontología descubierta. answer_filter exige que todos los términos del rol fuente (todos los miembros cuando corresponde al answer_role) cumplan el filtro/patrón pedido. corroborating_claim admite una sola proposición fact/state positiva elegible con bindings source/target→evidence referenciales; igualdad fuente es local acreditada y el objetivo usa identidad o patrón anónimo demostrado. Sin rama, binder, dependencia/contexto pendiente, ámbito/fundamento incompatible o negación equivalente en el mismo contexto. No hay búsqueda recursiva ni conclusión transitiva.

El apoyo auxiliar queda en relevance_profile.support_claim_ids y evidence_refs; no se publica como respuesta ni se completa unbound_target_roles. Fallo conserva candidato relevance_profile sin selección ni Difference. Estos perfiles protegen los dos enlaces declarados de abastecimiento: clase depósito de lluvia/agua, filtro de proveedor y apoyo de contexto acuático. Conservan las33útiles manuales y corrigen contraejemplos conocidos; todavía no acreditan calidad autónoma o generalización. [Evidencia y límites](documentary-v3-relevance-domain-results-2026-10-21.md).


## Contexto optativo de medición no realizada

measurement_context:{version:"negative_measurement_context_v1",rows:[...]} declara un aspecto de evento→atributo quantity, configuraciones fuente/objetivo exigidas y bindings de etiquetas de contexto. Forma cerrada, atributo conocido, IDs/claves únicos, configuración objetivo no vacía y bindings sin solapamientos. Ausencia conserva comportamiento anterior.

Solo measurement/negated_occurrence sin resultado y basis description/observation ante answer/fact/state quantity.amount positivo. Sujeto con identidad o anclaje nominal calificado, configuración conocida, condiciones restantes/contexto/premisas/fundamento compatibles, sin dependencia pendiente, rama o binder. Candidato conditional solo se admite tras acreditar esas condiciones. Las etiquetas explícitamente ligadas pueden igualar nombre referencial/texto sin probar identidad global; se conserva identity_proven:false y ambos IDs de calificador.

partial_measurement_context transporta source_event/source_qualifiers originales y no publica términos/proposiciones de valor. requested_value_inferred:false y unknown_value_inferred:false: no medir no implica valor desconocido, cero, máximo ni uso efectivo. Cotas de funcionamiento vecinas no completan la autonomía. Los calificadores consumidos para la comparación siguen íntegros en la salida. [Prueba y límites](documentary-v3-measurement-gap-results-2026-10-21.md).


## Contexto documental optativo con referente pendiente

documentary_anchor_context:{version:"anchored_documentary_context_v1",rows:[...]} identifica un enlace documental explícito, atributo, target_status occurred, source_link_kind/question_link_kind en owned_by/part_of y configuración fuente conocida no vacía. Forma cerrada, IDs/destinos únicos y referencias de vocabulario existentes; no registra equivalencia de relaciones.

La vía opera sobre grupos anónimos de clase/tipo compatibles, sin anáfora/binding y con un único vínculo estructural directo por lado. Sus padres requieren anclaje nominal compatible; clase explícita distinta o contexto adicional bloquean. Configuración, contexto restante, fundamento, premisas, dependencias, ámbito, ramas y binders se comprueban; perfiles ausentes conservan comportamiento anterior.

partial_documentary_anchor_context transporta la proposición y source_qualifiers originales, sin answer_terms, con identidad/ocurrencia/slot pendientes. link_kinds_equated:false, identity_proven:false, requested_event_occurrence_inferred:false, requested_value_inferred:false, unknown_value_inferred:false. No se certifica mismo grupo ni entrega; no consta en un registro no implica valor desconocido en el mundo. No se alteran los links de los frames ni se publica formato/páginas. [Prueba y límites](documentary-v3-documentary-anchor-results-2026-10-21.md).


## Dependencia de fidelidad antes de utilidad automática

Una selección útil sobre frames manuales no se transfiere automáticamente a una extracción distinta del mismo texto. Antes de evaluar utilidad automática, el protocolo congelado exige completar las116representaciones y los cuatro juicios faithful de cada crítico para todas las entradas. El cruce por tipo/text_sha256 con críticas previas describe requisitos pendientes; no mide recall, no garantiza matching ni autoriza evaluar solo el subconjunto aceptado. No se repara raw ni se cambia el juicio desfavorable para aprobar. Las tres omisiones requieren preservar uso neutral, asociación/purpose, dominio de cierre y atribución local antes de ampliar semántica del comparador. [Auditoría](documentary-v3-fidelity-readiness-results-2026-10-21.md).


## Dominio explícito de un cierre

En `Body.kind = value_restriction`, `answer_domain` es opcional para compatibilidad histórica. Formas cerradas: `{"kind":"all"}`, `{"kind":"lexical","class_quote":Evidence}` o `{"kind":"semantic","type_id":ID,"evidence":[Evidence]}`. Citas ligadas al texto fuente; tipos y citas semánticas registrados en el vocabulario. Los dominios de clase requieren `answer_role` declarado `ref` en esta versión. `null` significa dominio no resuelto. El perfil optativo de vocabulario `restriction_domain_profile = {"version":"explicit_answer_domains_v1"}` exige campo explícito; ausencia o null producen `unsupported_combination` local al claim, sin bloquear hechos independientes. Ausencia de campo y de perfil conserva semántica histórica, sin certificar fidelidad.

Excluir un valor exige pertenencia positiva en la pregunta; completar un conjunto exige coincidencia del filtro de clase. Una consulta sin clase no queda cerrada por un dominio de clase. No se infiere disjunción ni equivalencia entre clases/tipos. Configuración textual aislada no prueba pertenencia para la cautela de admisión. Testigos directos transportan `restriction_domain`; contextos transportan el cuerpo completo y nunca publican una restricción con dominio desconocido. [Controles, límites y evidencia](documentary-v3-closure-domain-results-2026-10-21.md).


## Recepción de entregas congeladas

`audit-documentary-v3-delivery <packet> <expected-frozen-sha256> <delivery-path> <new-report>` enlaza bytes del paquete con el hash del protocolo, sobre/IDs/orden y raws sin reparar. Lote mal vinculado conserva todas sus filas como unavailable; raw inválido conserva su diagnóstico y no descarta filas válidas independientes. Tipo/namespace deben coincidir con el paquete. UTF8/bytes y hashes de entrega se preservan; informe nuevo exclusivo, límite16MiBpor entrada. Acceso/límite fallido aborta sin un informe exitoso: el orquestador debe conservar fallo y denominador. No acredita fidelidad, aislación de contexto, unicidad de entrega histórica ni calidad. [Evidencia](documentary-v3-delivery-receipt-results-2026-10-21.md).


La extracción estricta sobre116textos, con guíaV3y dominio explícito, se ejecutó sin reparar sus entregas:98aceptadas por ambos críticos y101completas estructurales. No supera la puerta116/116; no habilita utilidad automática ni adopción. [Resultados y límites](documentary-v3-strict-extraction-results-2026-10-21.md).


## Ensamblado de entregas semánticas

`assemble-documentary-v3-delivery <packet> <expected-frozen-sha256> <semantic-items> <new-derived>` recibe `{version:"documentary_v3_semantic_items_v1",input_packet_sha256,items:[{id,raw}]}`. Exige inventario y orden exactos; no repara raws. Conserva bytes UTF8 originales, digest y valores JSON en `documentary_v3_assembled_delivery_v1`. Rechaza claves duplicadas recursivas, literales numéricos no enteros o fuera de rango y entradas/salida mayores de16MiB. Cantidades decimales usan strings.

Para nuevas campañas, `audit-documentary-v3-delivery <packet> <expected-frozen-sha256> <derived> <new-report> --assembled-only` vuelve a comprobar originales y valores, rechazando downgrade; sin flag conserva compatibilidad histórica. Integridad no verifica autoría/frescura/fuente-only ni fidelidad. Fallos y denominadores requieren registro de campaña; no se recuperan entregas ausentes de tandas antiguas. Publicación sin reemplazo por hard link; fallo de limpieza posterior puede dejar destino publicado y debe inspeccionarse, sin reintento ciego ni garantía de durabilidad de directorio. [Pruebas y límites](documentary-v3-semantic-handoff-results-2026-10-21.md).


## Descripciones locales de referencia optativas

Con `term_description_profile = {"version":"local_reference_descriptions_v1"}`, Term admite `{kind:"described_ref",id:ID,constraints:Proposition[],resolution:"known"|"unknown",evidence:Evidence[]}` en roles ref y miembros de cierre cuyo rol declarado es ref. Una a16restricciones únicas no recursivas, fact y basis description/specification, normative_scope null; cada una cita directamente la raíz como argumento ref. IDs, evidencias, ámbitos y dependencias se validan sin crear claims autónomos. Otros portadores escalares no admiten este término.

La conjunción pertenece exclusivamente a esa ocurrencia; no afirma existencia, uso, recepción ni procedencia. La igualdad exige ancla y todas las restricciones/contextos equivalentes, independiente del orden. Cantidad exacta distinta con misma coordenada/unidad/contexto o P/no-P del mismo átomo/contexto prueban incompatibilidad adicional; cierre exige pertenencia positiva e incompatibilidad con todos sus miembros. Ref desnudo, restricción ausente, unidades distintas o procedencias diferentes no prueban exclusión. No hay subsunción ni conversiones.

Se transporta host completo en source_constraint; ante faltantes, answer_terms vacíos, sin desenvolver descripciones. Contexto interno temporal/configuración se conserva; guards internos quedan unsupported. No se promete matching general de calificadores con alpha-renombrado, instanciación ni proyecciones de propiedades. Campos/perfiles históricos siguen reproducibles. [Controles y límites](documentary-v3-local-descriptions-results-2026-10-21.md).


La revisión crítica de descripciones añade corte previo de guards internos y estado `unsupported_description`: el host no se ejecuta ni selecciona como útil, pero queda como candidato con source_constraint/request_constraint y todas sus deudas. Las contradicciones locales no producen soporte completo. Se conserva el receptor de respuestas described_ref para reference_pattern; conjuntos mixtos pueden retener miembros enteros acreditados con extensión incompleta.

Una mezcla ref/described no demuestra subsunción: solo conserva contexto parcial si hay ancla nominal y asociación requerida acreditadas, con answer_terms vacíos. description_anchor transporta anchor_evidence (IDs y Evidence name originales) y términos íntegros; same_description usa comparación completa/mapa verificado y no es identidad global nueva. El descargador común universal reutiliza mapped_term y exige la variable real, sin instanciar individuos. Esa revisión dejó pendientes parciales anónimos cuantificados y coherencia global de aliases; la corrección posterior y sus límites se detallan a continuación. [Correcciones y evidencia](documentary-v3-description-review-results-2026-10-21.md).


Los términos descritos exigen coherencia de clases/tipos/denotaciones de todos los miembros de cada componente Same, incluidos dominios y operandos acompañantes. Una diferencia impide acreditar correspondencia y no prueba disjunción. Un parcial bajo cuantificador puede reutilizar el mapa alpha completo verificado aunque conserve deuda externa; no basta una constante o una raíz individual anónima. Un alias explícito se resuelve a una única pareja de variables, conserva los términos y documenta alpha_alias_roots sin extender identidad global. Sus vínculos restrictivos se heredan de todo el componente, con correspondencia por tipo y objetivo verificado y sin exigir duplicación en cada representante. Aristas different a terceros conservan su veto; calificadores alpha-renombrados generales siguen pendientes. [Verificación y límites](documentary-v3-description-boundaries-results-2026-10-21.md).


Para premisas fact/occurred del mismo átomo completo que una petición occurred_at/occurred_year, la coordenada conocida solicitada puede acreditar localmente la ocurrencia sin exigirse otra vez. Se verifican toda la cadena documental sin binders, selector externo exacto si existe, ausencia de otro tiempo exigido por la pregunta, identidad/modalidad/fundamento/ámbito/dependencias/elegibilidad y todos los demás calificadores. No se eliminan tiempos de claims/testigos: solo de una vista local de comprobación. Un año requiere format:year; no se convierte fecha, no se aplica a planes o estados ni se instancia cuantificación. [Controles y límites](documentary-v3-temporal-premise-results-2026-10-21.md).


La resolución de dependencias distingue aristas estructurales directas y cierre efectivo. El consumidor declara lo que contiene directamente; el qualifier posee referencias/descripciones/ámbito normativo/calificadores de su body. El motor sigue esos operandos implícitos y las dependencias declaradas alcanzables sin modificar raw ni exigir duplicación de aristas. Propaga incidencias por nodo desde el documento original del validador, detecta ciclos implícitos y conserva las deudas; un issue ajeno no bloquea. La deuda del request impide toda publicación/selección fiable, conservando request_constraint/source_constraints/pending_evidence en candidato unresolved. DocumentResult estructural no acredita por sí solo elegibilidad efectiva. No se reetiquetan revisiones históricas bajo esta aclaración. [Evidencia](documentary-v3-qualifier-dependencies-results-2026-10-21.md).


## Candidato prospectivo: atributos neutrales y modo operativo

El vocabulario de la próxima extracción, todavía sin adoptar, añade `frequency`, `delivery_location` y `operates_mode(subject: ref, mode: ref)`. La especialización como muestreo o inscripción requiere evidencia explícita. Los miembros descritos conservan parámetros locales íntegros; una condición exterior no sustituye esa restricción. El cierre necesario no afirma existencia/suficiencia; toda capacidad afirmada se conserva separadamente con su fundamento, sin promoverla a ejecución efectiva. Las citas de clase pertenecen al referente de la cláusula correcta. Guía V5 y revisión6 requieren revisión separada antes de extracción; las revisiones/puntuaciones anteriores permanecen intactas. [Controles, fallos y puerta](documentary-v3-neutral-configurations-results-2026-10-21.md).


La guía prospectiva V6 corrige igualdad literal universal frente a testigos compartidos, dependencia directa/cierre efectivo/aplicación, clase literal/sentido y pérdidas expresivas visibles. Separa instrucciones de campaña de reglas semánticas. Su revisión iterativa permite experimentación, no certifica fidelidad; la extracción nueva conserva cuatro objeciones y dos pérdidas de aceptación, sin adopción. No se cambian raws o puntuaciones históricas. [Resultados](documentary-v3-neutral-extraction-results-2026-10-21.md).


## Contexto parcial experimental separado

`classify-documentary-context-v2` conserva el recorrido estricto y añade testigos parciales con el evento o proposición fuente íntegros. Un patrón asociado anclado puede probar correspondencia contextual sin probar identidad de individuos o episodios. No une eventos, no produce términos del valor solicitado y conserva las deudas de identidad/evento/fundamento/premisa. Los enlaces de variables se comprueban sin usar su propio atajo alfa; los dominios nombrados no se equiparan estructuralmente. Un plan no satisface una petición de realización con alcance `truth_status`. Los estados del soporte documental conservan sus calificadores en el testigo y no se convierten en condiciones del valor desconocido ni acreditan el fundamento pedido. El comando anterior conserva sus informes por bytes. [Resultados y fallos preservados](documentary-v3-context-utility-results-2026-10-22.md).


`configuration_slot` permite consultar una clave registrada mediante el perfil optativo `documentary_configuration_slots_v1`; requiere target completo, estado y `one_value`. La clave solicitada no puede repetirse como restricción directa de la pregunta. Se conservan guards/contexto y unicidad antes de filtros; valores pendientes/conflictivos/negados bloquean la publicación. [Verificación](documentary-v3-configuration-slots-results-2026-10-22.md).

El contexto documental V2 admite prueba de tipo por clase literal exacta, única en el vocabulario, únicamente para el filtro de relevancia de hechos positivos en estado y scope/fundamento aplicables, sin binder/rama, anáfora, conflicto o tipo explícito incompatible. Conserva el tipo original ausente y solo selecciona evidencia parcial con términos vacíos; no cambia `class_match` ni acredita extracción. Comprobar esos límites y negaciones con configuración pendiente que bloqueen respuestas completas, sin veto global sobre contextos distintos. [Resultados y límites](documentary-v3-progress-results-2026-10-22.md).

La ruta contextual V2 de admisión admite también fundamento `capability` para la declaración explícita `admits→quantity` y consulta de asignación, conservando todas las condiciones y términos vacíos. La ampliación comprueba vínculos restrictivos y alias en ambos sentidos, incluidos anclajes anidados y deudas; no infiere uso ni cantidad. Verificar 72 combinaciones de fundamento/control/adaptador, correspondencia directa/anidada y contraejemplo de propietario distinto. El adaptador histórico conserva su comportamiento; contexto V2 también aplica la comprobación de vínculos a `description`. [Resultado y límites](documentary-v3-admission-capability-results-2026-10-22.md).


Contexto normativo anónimo experimental V2: conserva proposición, qualifiers y scopes fuente con términos de respuesta y bindings vacíos. Exige clase registrada única, anclajes nombrados y vínculos restrictivos compatibles en ambos sentidos, alias/dependencias limpios y alcance cuantificado demostrado. Mantiene deudas de identidad, objetivo normativo, slot y membresía cuando corresponda; no instancia universales ni publica el plazo. El caso individual frente a plazo común universal debe rechazarse. Verificar 49 controles críticos (incluidos alias de variables/dominios) y los cuatro pares normalizados; separar ese piloto de las 100 entradas anteriores y de fidelidad automática general. [Resultados y límites](documentary-v3-retention-context-results-2026-10-22.md).


El contexto V2 de suministro transporta únicamente relaciones explícitas hacia un destino nombrado usando relevance links y perfiles declarados: no identifica globalmente agua anónima ni compone origen transitivo. Conserva proposiciones/ámbitos/calificadores fuente y premisas/propósito pedidos; answer_terms, answer_propositions y bindings quedan vacíos, con deudas explícitas de identidad/origen/dato y premisa/propósito/fundamento cuando correspondan. Comprobar restricciones y ámbitos de todos los alias, contradicción local compatible, perfil de corroboración completo y bindings sin dependencia del orden. El filtro de patrones de respuesta solo exceptúa esta ruta parcial cuando esos campos están vacíos y los flags de origen/identidad son false. Verificación: tres omisiones originales, 49 controles principales y dos alias en rama, con fallos previos conservados. [Resultados y límites](documentary-v3-water-context-results-2026-10-22.md).


El perfil opcional de evaluación `admission_request_context_v1` declara contexto de admits→property con atributo/slot/setting y una premisa documental operates_mode específica que queda pendiente. Validar forma/IDs/roles/enlace; no activa producto ni equivale quantity con property. Solo contexto V2 con fuente negativa o restricción positiva cerrada, dominio modo en todos los miembros/alias y descriptor exacto permitido antes de vista temporal ref. Conservar cuerpo fuente y premisa/dependencia/evidencia exactos; términos/proposiciones de respuesta y bindings vacíos, flags de inferencia false. Comprobar igualdad del modo pedido a través de todos sus alias para no emitir cautela sobre un miembro ya acreditado; Unknown no prueba Difference. Reservar la ruta antes de fallback genérico. Verificar controles semánticos completos separados de parciales/inválidos y comparar ambos contratos explícitamente: los inputs derivados cambian vocabulario y hash, sin nueva extracción. [Contrato, resultados y límites](documentary-v3-admission-request-results-2026-10-22.md).


La consolidación de representaciones previamente revisadas se mide ejecutando el banco entero con procedencia y hashes por fila. Distinguir base, delta de vocabulario y reemplazos de frames; conservar namespace/ámbito por par y perfiles especiales. No sumar resultados de pilotos ni llamar extracción autónoma a reutilizar cuatro documentos. La siguiente guía conserva operates_mode para capacidad con su fundamento; uso efectivo solo restringe la premisa assignment correspondiente. Congelar paquetes fuente/pregunta separados, hashes, intento y doble crítica antes de dispatch; crítica y preflight del código no prueban fidelidad de116. Mantener diagnósticos con etiquetas fuera de los paquetes. [Resultados y protocolo](documentary-v3-retention-consolidation-results-2026-10-22.md).


`relative_nonoccurrence_bound` es un feature contextual V1 optativo solo del evaluador: qualifier fuente de medición negada, aspecto duration, límite until/switch_off y sujeto igual al evento, adjunción única y aristas explícitas. El límite no afirma apagado ni medición posterior. Validación/grafo/canonicalización consumen boundary_subject como referencia; consumer qualifier solo se habilita para este carrier. El veto previo a legacy recorre dependencias efectivas explícitas/implícitas y aliases; conserva hechos independientes y contexto parcial compatible con owner/límite/request íntegros y respuestas/bindings vacíos. Verificar forma, propietario, referencias, firma invariante a IDs, no proyección a negación global, dependencia normativa implícita, presupuesto compartido y replay histórico. El operador válido no acredita inventario de lecturas ni extracción autónoma. [Evidencia y límites](documentary-v3-relative-bound-results-2026-10-22.md).

## Complemento nominal «de» optativo

El feature `nominal_complement_de` del perfil contextual V1 habilita un `Reference.links` dirigido con `{kind,target,evidence}`. Conserva la construcción local núcleo→complemento «de», más específica que `associated_with`, sin decidir material, uso, posesión, localización ni existencia física independiente del término. No reemplaza una relación más fuerte explícita. El perfil admite ahora hasta seis features conocidos; su ausencia conserva los contratos anteriores.

La evidencia única debe cubrir exactamente el núcleo declarado (nombre si existe, clase en otro caso), espacio Unicode, palabra `de`, espacio y clase del complemento. El complemento es una referencia entity local anónima, sin binder, anáfora ni enlaces, en el mismo ámbito, con una sola incidencia propietaria considerando aliases. Comparar su descripción solo bajo ese enlace no demuestra identidad global. Los enlaces restrictivos adicionales del propietario y sus aliases siguen participando; ni el nombre compartido ni una asociación genérica sustituyen la construcción. El matching cuantificado no incorpora esta relación.

Un claim `property(subject=term,attribute=nominal_qualitative_descriptor,value=text)` de hecho/estado/descripción sobre ese término conserva metadata léxica local: puede ser completo como representación sin afirmar una propiedad universal del papel. Se filtran ese claim y sus consumidores efectivos explícitos, implícitos y por aliases antes del motor ordinario. Una consulta local compatible puede recibir contexto parcial ligado a su propietario, sin cifras, proposiciones de respuesta ni bindings fabricados. Los hechos independientes conservan su utilidad. La canonicalización compara el término únicamente al recorrer el enlace nominal; no crea una clave global para el anónimo.

La implementación debe cobrar recorridos y copias al presupuesto acumulado de la operación y no activar esos recorridos bajo otro perfil que carezca del feature. El diseño y sus límites están en [nominal-de-design](documentary-v3-nominal-de-design-2026-10-22.md). La aceptación de una representación conocida no constituye nueva extracción autónoma, mejora medida de recuperación ni adopción en producto.


## Conclusión existencial optativa de evaluación

El feature `existential_conclusion_projection` amplía el perfil contextual V1 a siete features conocidos. Con permiso válido, el agregador puede publicar `documentary_reading_pair_report_v2` con `completion_kind=existential_conclusion`, cuerpo estructurado con binder local y mapas por cada cruce. No modifica `Same`, `reference_key` ni `canonical_inner`. Sin el feature conserva V1 por bytes y no recorre el nuevo grafo. El consumidor reconstruye las pruebas desde documentos validados y matriz completa antes de completar la petición.

La proyección conserva owner, roles, incidencias correlacionadas, descripción, enlaces y cierre efectivo saliente; metadata independiente queda íntegra en originales. Cada ancla externa conserva las claves históricas por lado/nodo. Un uso nominal léxico no adquiere existencia física ni puede usarse como argumento físico o Different. No se unen variables entre conclusiones. Campos, variantes y diagnósticos están cerrados en el diseño; formas no soportadas conservan `not_supported` sin contar como éxito. La primera forma admite un apoyo directo positivo fact/state/description, un claim, slot ref/one_value y scopes documentales. Las exclusiones y errores técnicos globales se describen en el addendum.

Copias, IDs, claves y comparaciones consumen presupuesto acumulado; todo limit, incluido cap local, agota la operación completa. V2 conserva `text` y `original_raw` de cada lado con cargo previo. Orígenes se resuelven por extremos y etiquetas completos, nunca por un solo kind o primer ID coincidente. [Diseño](documentary-v3-existential-projection-design-2026-10-22.md), [precisiones](documentary-v3-existential-projection-implementation-addendum-2026-10-22.md) y [evidencia](documentary-v3-existential-projection-results-2026-10-22.md).


`nominal_possessive` es el octavo feature optativo del perfil contextual V1, solo en evaluación. Admite `Reference.denotation=lexical_use` únicamente como determinante prenominal de incidencia única, con frontera léxica, evidencia y scope íntegros; no es entidad ni antecedente resuelto. El operador conserva el argumento contextual abierto, corta sus consumidores/alias efectivos antes de rutas ordinarias y proyección existencial, y publica solo contexto parcial con documentos fuente/pregunta completos y arrays de respuesta/bindings vacíos. Correlaciones entre predicados se validan por una fila completa de relevancia; no se mezclan bindings. Guards consultan claims ejecutables del lado correcto. Todo recorrido/copia se precobra y un límite invalida la operación entera. `condition_suffices` es vocabulario dinámico declarado con semántica de condición particular; no redefine `class_suffices`. Diseño y controles locales no acreditan fidelidad autónoma. [Contrato y evidencia](documentary-v3-nominal-possessive-results-2026-10-22.md).


`normative_question_indeterminacy` es el noveno feature contextual V1 y requiere `normative_action_relation`. Solo separa representación completa y resolución bloqueada para el constructor normativo exacto de pregunta; conserva todos los issues originales y sus paths. Actor ambiguo y scope no explicitado no son extracción fallida ni datos resueltos. Scope null no prueba igualdad con destino ni funciona como wildcard. El corte por contenido precede posesivo/límite relativo/legacy y conserva request/documento/deudas íntegros, sin soporte/refutación/respuestas/bindings ni conclusión existencial. Cobertura no vacía/toda representada, evidencia positiva, forma/grafo exactos y atribución de cada issue son obligatorios; los demás errores no se promueven. Verificar DTO parcial/inválido también bloqueado, presupuesto/depth64/scans/copia, independencia y replay sin feature. Un requires=request es inválido en el grafo vigente, no un control semántico de dependencia. La disposición estructural no acredita fidelidad; exige doble crítica para contabilizar una corrección. [Contrato](documentary-v3-normative-indeterminacy-contract-2026-10-22.md), [resultados](documentary-v3-normative-indeterminacy-results-2026-10-22.md).

El décimo feature opcional `lexical_clause_context` conserva una condición léxica positiva del Claim completo y su contexto original rígido, sin especialización por consumidor. La selección local usa intent ligado a la triple derivada, inversión exacta de la adición y manifiesto externo que ancla procedencia y revisión. Una deuda de coverage se conserva con resolución explícita; otras deudas efectivas no se descargan por accepted. Las salidas selected/no_match/pending/unsupported/error son distintas y comparten presupuesto. Solo selected contiene evidencia completa, sin respuesta; no prueba identidad entre documentos, extracción automática ni política productiva. [Contrato cerrado](documentary-v3-lexical-clause-context-design-2026-10-22.md).
