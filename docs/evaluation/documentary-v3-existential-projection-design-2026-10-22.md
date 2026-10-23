# Proyección existencial de una conclusión común sin identidad global

Fecha: 2026-10-22. Estado: propuesta operativa para revisión, todavía sin implementación. Alcance exclusivo del evaluador documental V3. No modifica Rust, Cargo, producto, dependencias, DB, API, raws, pregunta, expectativa, dictámenes ni resultados históricos.

## Decisión propuesta

Añadir el feature optativo `existential_conclusion_projection` al perfil `request_context_profile`, versión `documentary_request_context_v1`. Habilita un constructor de **conclusiones existenciales estructuradas**, separado de `reference_key`, de `Same` y de las respuestas que identifican globalmente un referente. Ausencia del feature conserva exactamente el recorrido y la salida históricos; feature inválido invalida el vocabulario. Si se implementa sobre el inventario actual de seis features, la lista cerrada y su máximo pasarían a siete. No es un feature Cargo.

La operación puede concluir, bajo todas las lecturas cerradas, «existe un ítem con esta descripción que ocupa este rol de este owner» sin concluir «las lecturas identifican el mismo ítem». La variable pertenece únicamente al cuerpo y a la respuesta de una conclusión de una solicitud de esta operación. No es una referencia documental, no se incorpora al grafo de identidad, no puede recuperarse como entidad y no se transporta como binding global.

El primer subset es general por forma, sin IDs, nombres, casos o atributos enumerados como excepciones: solicitud `mode=answer`, slot declarado `ref`, extensión `one_value`, soporte directo de un solo claim `fact/state` positivo con `basis=description`, conservado y compatible con la solicitud. El filtro de clase puede ser nulo o literal ya comprobado; `reference_pattern` y `described_ref` quedan `not_supported` en esta primera versión. La comprobación temprana de inventario mediante `context_owner_key` en `readings.rs` permanece intacta: el feature no rescata requests que fallen esa puerta. El rol pedido está ausente del target y recibe un único referente local anónimo de denotación `entity`. Los demás roles del átomo tienen correspondencia ya acreditada por los mecanismos vigentes. Se exige owner nombrado acreditado; no se inventa un owner para predicados que carecen de él. El vocabulario declara predicado, roles y clases; el feature no añade semántica léxica ni infiere existencia desde una mera referencia.

`one_value` permite devolver un testigo de respuesta; no prueba unicidad, inventario completo ni que todos los objetos posean esa descripción. Solicitudes de identidad, enumeración exhaustiva, cardinalidad, conjunto cerrado, múltiples slots, refutación, negación, obligación/permiso, planes, eventos, cuantificación o ramas no entran en este primer subset. Tampoco entra una respuesta que necesite combinar claims para acreditar el átomo. Una extensión futura requeriría sus propias reglas de introducción, sin ampliar este feature por fallback.

## Evidencia que motiva el diseño

El [límite nominal conservado](documentary-v3-nominal-de-common-item-limit-2026-10-22.md) documenta que ambas hojas del control manual «Senda tiene una bandeja de papel grande.» completan localmente `has_item.item`, mientras el envelope conserva `conditional_reading`. No se cambia ese resultado observado.

La lectura estática de `tools/evaluation/src/documentary_v3_io/readings.rs` confirma la causa: `request_signature` exige canonicalizar respuesta y claims; `canonical_inner` usa `reference_key` tanto para `ref` como para `described_ref`; `contextual_reference_key` exige nombre explícito. La raíz anónima de la respuesta no obtiene clave. El agregado compara firmas de todos los cruces y la ausencia de una firma impide completitud. Este diseño deja esas funciones y su significado intactos.

La capacidad nueva sería publicar una conclusión común equivalente a «Senda tiene al menos una bandeja de papel», con construcción nominal dirigida y subtipo físico indeterminado. La adjunción de «grande» sigue abierta por lectura. No se afirma una hoja/lote de papel independiente, material, función o ubicación. No basta con poner `described_ref`, texto libre o un nombre artificial donde hoy hay un `ref`.

## Dos objetos distintos: contenido y prueba local

La conclusión es un cuerpo tipado que usa los carriers documentales existentes, sustituyendo únicamente los operandos locales autorizados por un término nuevo de salida `bound_variable`. Su binder tiene `quantifier=exists`, una variable de respuesta `v0` y un alcance explícito igual al cuerpo de esa conclusión. `v0` no es un ID de `references` y ningún lector puede enviarlo a `reference_key`. La etiqueta se asigna determinísticamente; renombrarla no altera el significado.

El contenido conserva el target completo de la solicitud, rol, fundamento requerido, extensión, unidad/filtros y todas las restricciones aplicables de la respuesta, el átomo fuente completo y su juicio, estado, polaridad, fundamento, scope/contexto, qualifiers, restricciones y enlaces. Conserva ausencias como ausencias, nunca como comodines. El descriptor léxico nominal usa el carrier de salida cerrado `local_nominal_lexical_use`, ligado a una sola incidencia de enlace y con el descriptor original completo, sin variable existencial propia. Se representa con su incidencia owner→kind→descriptor bajo `nominal_complement_de`; no se liga como una segunda entidad física existente. Solo se habilita ese carrier si también está habilitado y satisfecho su feature original. Cualquier otro referente anónimo alcanzable que no tenga regla local competente bloquea la proyección.

La prueba local conserva para **cada cruce fuente×pregunta** los documentos/hash, IDs de lectura, solicitud, claim y witness originales; textos y evidencias exactas; componentes Same locales completos; referencias y enlaces; scopes y qualifiers; cierre efectivo de dependencias; target y filtros; resultado y deudas originales. Conserva un mapa explícito `v0 → componente local de la respuesta` y el anclaje nominal por incidencia. Este mapa está indexado por lectura y conclusión. Dos entradas del mapa jamás crean una arista entre sus referentes.

Propuesta de sobre cerrado para el nuevo canal de salida:

```text
existential_conclusions = [
  {
    version: "documentary_existential_conclusion_v1",
    conclusion_id: <ID local de salida>,
    request_id: <ID original>,
    binder: {quantifier: "exists", variables: ["v0"], answer_variable: "v0"},
    request: <contenido tipado de la solicitud sin su ID técnico>,
    body: <átomo, contexto y grafo protegido tipados con bound_variable>,
    identity_proven: false,
    evidence_by_reading: [<prueba local íntegra por cada cruce>],
    residual_by_reading: [<metadata y hechos no usados, íntegros por cruce>]
  }
]
```

Los metavariables entre ángulos describen tipos, no un formato JSON aceptable ni texto libre ejecutable. La implementación deberá fijar DTOs cerrados para `body` y pruebas antes de habilitar el feature: reutilizar los enums existentes, añadir solo el término de variable ligada de salida y rechazar claves desconocidas. Ningún `body` podrá ser un string, JSON citado o una firma/hash opaca como sustituto del contenido. Un hash puede servir de índice, pero la igualdad y la prueba se comprueban sobre estructuras completas.

`answer_terms`, `answer_propositions` y `bindings` históricos no reciben un `ref` artificial ni quedan poblados con una identidad implícita. Las respuestas locales permanecen íntegras en la matriz. Todas las incidencias del componente de respuesta dentro del único átomo se sustituyen conjuntamente: `R(owner=A,left=x,right=x)` no es equivalente a `R(owner=A,left=x,right=y)`. El segundo caso solo entra si el referente `y` externo tiene correspondencia vigente propia; nunca se lo colapsa en `v0`. El request puede declarar `complete_answer=true` con `completion_kind=existential_conclusion` solo cuando existe este canal nuevo válido y el consumidor entiende su tipo. Un consumidor histórico que solo acepte términos identificados debe conservar el resultado anterior; no interpretar listas vacías como respuesta. El canal anterior sigue usando `completion_kind=identity_answer` si se decide versionar ese discriminante. La forma/versionado del reporte deberá cerrarse en la implementación; no se declara retrocompatibilidad de consumidores que no conozcan el canal nuevo.

## Regla de proyección por lectura

Sea `W(c,r)` un witness de soporte directo para la solicitud `r` en el cruce `c`. Sea `G(c,r,W)` su grafo protegido, construido desde:

1. La solicitud completa, sus premisas/filtros, el átomo que la satisface y el único claim fuente seleccionado, incluidos owner y todos sus roles.
2. El referente de respuesta con todos sus aliases Same locales, descripciones del término, clases/tipos/denotaciones y **todos los links** de todos esos aliases. El cierre de cada target sigue su carrier competente; se incluyen también aristas Different incidentes y sus objetivos, sin convertir Unknown en Difference.
3. Todos los scopes, contextos, qualifiers y restricciones alcanzables de esos elementos; dependencias declaradas y operandos implícitos transitivos, referencias internas y aliases, con detección de ciclos y deudas.
4. La evidencia necesaria para acreditar existencia local, correspondencia de owner y satisfacción completa de la solicitud. El átomo positivo justifica la introducción existencial; la mención/clase aislada no la justifica.

No se usa todo claim que mencione casualmente la raíz como nueva premisa del átomo. En cambio, ningún claim consumido por filtro, qualifier, guard, restricción, dependencia o descripción puede pasar al residual para eludirlo. El cierre es dirigido desde el soporte hacia aquello de lo que depende: `has_item → tray → paper` incluye la construcción nominal; `local_descriptor → tray/paper` es un consumidor entrante independiente y no entra por esa sola incidencia. Si existe `has_item → local_descriptor` explícita o implícita, el descriptor sí pertenece al cierre y no puede residualizarse. El cierre no recorre indiscriminadamente consumidores entrantes. El closure sigue la semántica de aristas directas/efectivas del [contrato vigente](documentary-v3-local-contract-2026-10-21.md), no solo `depends_on` explícitos. Un conflicto relevante sobre ese soporte bloquea aunque no esté declarado como dependencia; no se limpia una contradicción desplazándola al residual.

La proyección `P(c,r,W)` existe solo si el cruce y los documentos son elegibles, la solicitud y el witness son completos y sin deudas/conflictos relevantes, el grafo entero cumple el subset, el owner está acreditado y todas sus referencias externas tienen la correspondencia vigente requerida. Se reemplaza el componente local anónimo de respuesta por `v0` en **todas** sus incidencias del grafo protegido, sin capturar otras variables; el resto del grafo permanece íntegro. No se llama a la identidad global para `v0` ni para el descriptor nominal local.

La transformación es una introducción existencial sobre un testigo que ya satisface la solicitud, no una intersección que quite restricciones discrepantes. Si una clase, enlace, qualifier, condición, fundamento o dependencia efectiva difiere, las proyecciones difieren o la forma no está soportada. No se selecciona un subgrafo favorable después de observar el desacuerdo.

En este subset, binder/rama no documentales, anáforas sin resolver, ciclos, guards, aplicación de reglas, referencias a episodios, clausuras universales o aliases incompatibles conservan deuda y bloquean el candidato. Los qualifiers solo entran si todo su cuerpo y alcance se pueden representar y comparar con el mapa local completo; el recorrido no borra `scope_id` antes de resolverlo. Una forma hoy no soportada, como qualifiers recursivos sin canonicalizador competente, sigue bloqueada. No se convierte una negación en existencia positiva.

## Equivalencia formal y publicación común

Para dos proyecciones `P1` y `P2`, `P1 ≡α P2` si existe una biyección tipada de sus variables ligadas y componentes Same locales descriptivos autorizados (no una biyección de raw IDs, porque un componente puede tener distinto número de aliases de representación) que:

- fija todas las anclas externas acreditadas y conserva el owner y la posición de cada rol;
- conserva literalmente predicado, juicio, estado, polaridad, fundamento y requisitos de la solicitud;
- conserva clases/tipos/denotaciones, descripciones, todas las aristas tipadas y su dirección/multiplicidad, restricciones Different y estructura de scopes/contextos;
- conserva qualifiers y sus propietarios, dependencias y restricciones completas, incluida estructura de ausencia, unidades y filtros;
- no mapea eventos o referencias externas a variables ni mezcla raíces locales distintas;
- compara evidencia como procedencia válida en cada prueba, sin exigir que los IDs técnicos o spans entre lecturas sean la identidad de la variable.

Solo se ordenan conjuntos semánticamente declarados sin orden. Listas con orden significativo y multiplicidades se conservan. IDs técnicos, orden de filas y etiquetas de variable pueden variar, pero no contenido, incidencia o alias. La evidencia no desaparece de la prueba por excluir offsets de la igualdad semántica. Compartir cita, clase o ID local nunca sustituye esta prueba.

Sea `C = source_leaves × question_leaves`, no vacío. Para publicar una conclusión `K` se exige: envelopes elegibles **cerrados** y sin dimensiones abiertas que afecten su validez; mismo inventario de solicitudes y contexto acreditado; y, para cada `c ∈ C`, al menos un witness completo `Wc` tal que `P(c,r,Wc) ≡α K`. Se conserva la lista de witnesses examinados y la prueba escogida por cruce. No se omiten hojas inválidas, parciales, refutatorias o sin respuesta. Una ambigüedad de adjunción representada por dos hojas cerradas puede permanecer como metadata abierta sin invalidar una conclusión cuyo grafo protegido es idéntico.

No se exige igualdad del inventario de todos los witnesses alternativos: cada lectura debe demostrar el mismo contenido de conclusión con una prueba local completa. Tampoco se fabrica una conclusión por intersección de trozos de witnesses. La comparación conserva el patrón de coocurrencia: dos apariciones del mismo componente local deben seguir siendo la misma variable; dos componentes locales distintos no colapsan en una variable. Los candidatos `K` son proyecciones **enteras** construidas mecánicamente antes del cruce; la operación solo agrupa por equivalencia completa. Si hay varias conclusiones distintas comunes, se publican separadas o se conserva ambigüedad según la extensión de la solicitud. Nunca se elige una por ID/orden ni se afirma unicidad.

La proposición lógica respaldada es `para todo c existe xc: Bodyc(xc)` con `Bodyc` equivalente bajo renombrado. Esto autoriza el contenido común `exists v0: Body(v0)` como respuesta bajo incertidumbre de lectura. No autoriza intercambiar cuantificadores y afirmar `existe un único x para todas las lecturas`, ni agregar identidad translectura.

## Metadata abierta, dependencias y no interferencia

En el ejemplo motivador, `has_item(owner,item)` y el link nominal de item a descriptor son parte protegida. La metadata de tamaño no solicitada ni consumida por ese soporte queda íntegra en `residual_by_reading`: una hoja la adjunta a la bandeja, otra al descriptor papel. Su diferencia no se añade a la conclusión común y no se reetiqueta como propiedad común. La etiqueta `metadata_open=true` puede informar esta diferencia sin hacer `complete_answer=false` para la única conclusión común ya probada. Si el tamaño se convierte en filtro, descripción del término o dependencia efectiva de esa misma solicitud, entra al grafo protegido y la discrepancia bloquea su completitud.

La separación es por incidencia y dependencia comprobada, no por lista de atributos que convenga ignorar. Todos los hechos originales, incluidos los independientes, permanecen en sus hojas y siguen disponibles para otras solicitudes por sus rutas existentes. Un fallo de la nueva proyección no veta una respuesta independiente válida ni elimina un witness anterior. Solo agotamiento global del presupuesto mantiene el fallo de operación que ya exige el contrato; no se presenta ese resultado como una negativa semántica.

Cada conclusión posee su propio binder. De `exists x: A(x)` y `exists y: B(y)` no se obtiene `exists z: A(z) and B(z)`. Aunque dos conclusiones usen el alias visual `v0` o sus pruebas apunten al mismo ID en una hoja, sus namespaces de conclusión son distintos. No hay joins entre conclusiones, documentos, requests o lecturas. El primer subset tampoco compone claims de dos episodios para completar un átomo, ni usa una propiedad de un testigo para llenar otro. Propiedades correlacionadas solo pueden publicarse juntas en un cuerpo común que ya tenga demostrada íntegramente esa correlación; esa composición queda fuera del subset inicial.

La afirmación de completitud se restringe al contenido estructurado de `K`, no al inventario de entidades ni a la resolución de metadata. `requires_review` debe derivarse de solicitudes sin completar y errores, separadamente de la metadata abierta del envelope. El feature no certifica el inventario exhaustivo de interpretaciones del lenguaje natural.

## Integración futura, coste y cierre seguro

El punto de integración propuesto es posterior a clasificación local y anterior a la decisión agregada de respuesta común en `readings.rs`. Un módulo puro del evaluador construiría grafo/prueba/canonicalización de conclusión. No modifica matching local, `reference_key`, `Same`, `canonical_inner` histórico, extracción ni producto. El canal se activa por feature antes de cualquier recorrido nuevo; si la vía de identidad ya acredita respuesta conserva su resultado. El fallo especializado conserva la matriz y diagnóstico, sin fallback que publique un ref desnudo o texto libre.

Todo recorrido, expansión de aliases/dependencias, comparación entre cruces, candidato, copia de IDs/evidencia, materialización residual y serialización carga el presupuesto acumulado existente de la operación. No se reinicia por lectura, request, conclusión ni fallback. Los candidatos se generan desde witnesses enteros; se evita enumerar subconjuntos de propiedades. El máximo de hojas vigente permanece; límite agotado nunca permite truncar a un subconjunto favorable. La salida nueva tendrá límite explícito dentro del presupuesto compartido, sin convertir referencias externas a archivos mutables en prueba íntegra.

El coste nuevo es un tipo de conclusión y prueba, un recorrido de grafo acotado, alpha-equivalencia tipada y consumo nuevo de salida. No es solo cambiar una comparación de firmas. La implementación deberá cerrar el esquema de reporte y sus consumidores, ampliar validación del feature, conservar replays y medir coste local antes de declarar capacidad operativa. Este documento no ejecuta ni autoriza por sí mismo esa ampliación.

## Controles de falsación propuestos; no ejecutados

| Control | Resultado exigido |
| --- | --- |
| Dos hojas completas con raíz anónima y cuerpo protegido idéntico; IDs y orden distintos | Misma conclusión existencial, mapas locales separados, ninguna referencia global. |
| Control manual nominal conservado, sin cambiar pregunta/expectativa | Conclusión has_item con descripción nominal común; tamaño residual por hoja; informe histórico intacto. |
| Renombrar todos los IDs locales y permutar listas declaradas como conjuntos | Contenido común equivalente; procedencia vinculada a IDs originales correctos. |
| Mismo ID, cita o clase, pero owner distinto o rol invertido | Ninguna conclusión común; no identidad por coincidencia superficial. |
| Cambiar papel por otro descriptor, borrar un link, cambiar kind/dirección o añadir link en un alias | Proyección distinta o pendiente; nunca limpiar la arista discrepante. |
| Añadir Different interno al componente Same o contradicción efectiva sobre el soporte | Candidato bloqueado con conflicto íntegro; hechos independientes conservados. |
| Descriptor reutilizado entre owners distintos o target nominal convertido en sujeto global | Rechazo por su contrato original; ninguna identidad/existencia física de papel inferida. |
| Cambiar polaridad, modalidad, estado, fundamento, tiempo, configuración o scope | Ninguna conclusión común simplificada; ninguna conversión permiso/plan→hecho. |
| Qualifier igual en texto pero propietario, ámbito o dependencia implícita distintos | Equivalencia rechazada o pendiente, sin borrar incidencia. |
| Deuda en request, alias, qualifier, filtro o dependencia alcanzable | Sin completitud de ese candidato; pruebas/deudas íntegras. |
| Metadata independiente cambia de adjunción | Conclusión protegida estable y residual distinto; ninguna propiedad común nueva. |
| La misma metadata pasa a filtro/descripción/dependencia efectiva | Ya no puede quedar residual; discrepancia impide completitud. |
| Dos ítems por lectura: uno tiene A y otro B; hojas intercambian roles | Ninguna conjunción A+B sobre un solo testigo por join; binders por conclusión. |
| Dos episodios aportan cada uno solo parte de un átomo o respuesta | Ninguna composición; primer subset exige soporte directo entero. |
| Una hoja sin soporte, parcial, refutatoria, inválida; envelope abierto; inventario de requests distinto | Nunca completar eliminando esa hoja/cruce. |
| Witnesses alternativos distintos pero cada cruce tiene una prueba íntegra del mismo K | Publicar K con prueba por cruce; no exigir igualdad de inventario accidental. |
| Dos conclusiones comunes no equivalentes para one_value | No escoger por orden/ID ni afirmar unicidad; conservar alternativas o revisión. |
| Referencia sin claim positivo; negación o cuantificador fuente | No introducir existencia; mantener fuera del subset. |
| Reutilizar v0 desde otra conclusión/request/documento | Referencia fuera de binder inválida; ningún binding global. |
| Evidencia UTF-8/hashes adulterados o mapa a una raíz distinta de la respuesta real | Validación/prueba fallida; no aceptar una firma equivalente sin origen. |
| Feature ausente, incluso con documentos que no necesitan proyección | Replays históricos por bytes; cero recorridos nuevos. |
| Límite justo de nodos/bytes, aliases expansivos y muchas comparaciones | Cargo acumulado, fallo explícito, ninguna publicación parcial favorable. |
| Otra solicitud independiente completamente respondida junto a un candidato bloqueado | Su respuesta y procedencia sobreviven; el fallo no se convierte en veto documental. |

La aceptación requiere controles sintéticos que cambien una sola coordenada, revisión del esquema cerrado y replays con feature ausente. No se propone una campaña grande, llamadas API ni datos nuevos del banco en esta fase. Las exclusiones estrictas se reportan como `not_supported`, conservando su denominador; no se cuentan como éxitos de calidad ni negativos correctos. Hasta implementar y ejecutar esos controles, el estado sigue siendo diseño revisable y el fallo conocido de agregación permanece vigente. El diseño no demuestra extracción autónoma, fidelidad semántica general, utilidad medida, latencia de producción ni adopción.

## Addendum normativo: DTOs y versionado cerrados antes de implementar

Este addendum sustituye las opciones de esquema/versionado abiertas en las secciones anteriores. Es decisión de diseño, no una descripción de código ya disponible. La revisión root y del crítico de este esquema y su coste precede a cualquier implementación.

### Versionado y completitud

Con feature válido y habilitado, el reporte usa exactamente `version="documentary_reading_pair_report_v2"`. Sin feature, conserva `documentary_reading_pair_report_v1` y la salida histórica por bytes, sin campos ni recorridos nuevos. Un vocabulario/perfil inválido conserva el error de validación; nunca habilita V2 por mera presencia textual del nombre del feature.

V2 conserva los campos del reporte V1 y la matriz completa e inmutable. Cada elemento de `result.requests` tiene exactamente sus cuatro campos anteriores (`request_id`, `coverage`, `complete_answer`, `evidence_by_reading`) más:

```text
completion_kind: "identity_answer" | "existential_conclusion" | "conditional_reading" | "none"
existential_conclusions: ExistentialConclusion[]
projection_diagnostics: ProjectionDiagnostic[]
```

Ambos arrays están siempre presentes en V2. `identity_answer` exige que pase la agregación histórica; `existential_conclusion` exige la regla común de este documento. Ambos implican `coverage=complete` y `complete_answer=true`. `conditional_reading` y `none` implican `complete_answer=false` y sus valores homónimos de coverage. Si pasa la identidad histórica se elige `identity_answer`, con `existential_conclusions=[]`, sin ejecutar proyección innecesaria. Si identidad no pasa pero la proyección común sí, se elige `existential_conclusion`. No hay rescate existencial de un request de identidad: únicamente del slot referencial `one_value` ya satisfecho localmente. No acredita unicidad, enumeración, identidad translectura o un objeto compartido entre mundos.

Si no hay conclusión común, el array de conclusiones queda vacío y se conserva la evidencia condicional original. Un diagnóstico no sustituye evidencia ni cambia un fallo de validación en negativo semántico. `result.complete_answer` sigue exigiendo completitud de todas las solicitudes; `requires_review` se deriva de elegibilidad y solicitudes incompletas, no de metadata independiente divergente. El reporte conserva los flags de ausencia de medición automática/latencia/fidelidad.

### Sobre cerrado de conclusión y prueba

`ExistentialConclusion` tiene exactamente estos campos:

```text
version: "documentary_existential_conclusion_v1"
conclusion_id: string
request_id: string
binder: {quantifier: "exists", variables: ["v0"], answer_variable: "v0"}
request: NormalizedRequest
body: ConclusionGraph
identity_proven: false
origin_maps: OriginMap[]
```

El namespace de cualquier ID local es `(request_id, conclusion_id)` dentro del reporte; la variable `v0` está ligada exclusivamente por ese binder. Ningún ID o variable permite joins con otra conclusión, solicitud, lectura o reporte. `conclusion_id` se asigna después de ordenar cuerpos normalizados estructuralmente; no depende de IDs fuente, pregunta o caso. El ID es transporte, no prueba de equivalencia.

`OriginMap` tiene exactamente:

```text
source_reading_id: string
question_reading_id: string
matrix_cell_index: nonnegative_integer
request_index: nonnegative_integer
support_witness_index: nonnegative_integer
node_origins: NodeOrigin[]
record_origins: RecordOrigin[]
```

Cada conclusión tiene exactamente un `OriginMap` por cruce real de la matriz, sin omisiones ni duplicados. La entrada apunta al witness de `support` dentro de la solicitud/celda indicada. Se comprueban índices, IDs de lectura, request, completitud y vínculo con documentos validados; no basta encontrar el mismo claim ID en otra celda. La celda conserva íntegramente fuente/pregunta validadas y sus raws/textos a través de los campos `source`/`question` del reporte y los IDs de lectura. La comprobación resuelve estos vínculos dentro del mismo reporte inmutable, nunca contra archivos externos mutables o un informe deserializado como si fuera validación fresca.

`NodeOrigin` tiene exactamente `{node_id, document_side, reference_ids, canonical_reference_key}`. `document_side` es `source|question`; `reference_ids` contiene todos los miembros originales del componente Same local correspondiente. Puede haber varias filas para un nodo con anclaje fuente/pregunta acreditado. `canonical_reference_key` es la clave histórica completa para un `named_anchor`; es `null` para `bound_variable` y `local_nominal_lexical_use`. La clave no sustituye el descriptor estructurado del nodo ni habilita identidad de los otros tipos.

`RecordOrigin` tiene exactamente `{record_kind, local_record_id, document_side, document_pointers}`. Los kinds son `request|claim|scope|qualifier|restriction|edge`; `document_pointers` es una lista no vacía de JSON Pointers dentro de la hoja original indicada por ese OriginMap. Se incluyen todas las filas/aristas/evidencias consumidas por el cierre protegido y el witness original. Las citas y los offsets originales se conservan allí; los pointers permiten comprobar el cuerpo normalizado contra su origen completo. No se aceptan pointers ausentes, fuera de la hoja o a un registro incompatible.

La metadata y hechos residuales no tienen una segunda copia parcial: **la matriz y los documentos originales íntegros son el residual autoritativo**. Todo registro no consumido sigue allí, con su adjunción, evidencia, consumidores y resultado propios. No se elimina ni reescribe un registro consumido tampoco. El addendum sustituye `residual_by_reading` y `evidence_by_reading` dentro del sobre conceptual de conclusión por `origin_maps`; la evidencia y el residual completos permanecen en el reporte matriz. No se permite construir una matriz reducida a las pruebas favorables.

### Grafo cerrado y normalización tipada

`ConclusionGraph` tiene exactamente:

```text
atom: NormalizedAtom
judgment: "fact"
status: "state"
basis: "description"
root_scope: string
nodes: ConclusionNode[]
scopes: NormalizedScope[]
qualifiers: NormalizedQualifier[]
restrictions: NormalizedRestriction[]
context: NormalizedContext
edges: ConclusionEdge[]
```

`NormalizedAtom`, `NormalizedRequest`, `NormalizedScope`, `NormalizedQualifier`, `NormalizedRestriction` y `NormalizedContext` reutilizan **únicamente** las variantes/campos cerrados de los carriers documentales V3 vigentes que entran en el subset inicial. No admiten mapas arbitrarios ni texto/JSON citado como cuerpo. La transformación `N` conserva todos los campos semánticos de la variante de entrada y reemplaza exclusivamente: (a) IDs de registros por IDs locales canónicos; (b) operandos de referencia, incluidos strings de roles tipados como referencia, por `NodeOperand={node_id:string}`; (c) `qualifier_ids`, scope/parent/dependency IDs por referencias locales verificadas a registros de sus arrays; (d) evidencias/offsets por su conservación íntegra en `RecordOrigin`. No elimina valores `null`, listas vacías ni campos semánticos de ausencia. Todo campo que no tenga regla de transformación tipada produce `not_supported`; no se copia como texto ni se omite.

`NormalizedRequest` conserva todos los campos de la solicitud salvo su ID y evidencia, incluidos target, modo, slot, extensión, unidad, filtros, fundamento, premisas y contexto. Los campos de referencia usan `NodeOperand`; IDs de premisas apuntan a registros normalizados conservados en `restrictions`. `NormalizedScope` conserva clase, parent, binder/rama y demás contenido de su carrier; binder/rama no soportados siguen excluidos y no se convierten a null. `root_scope` apunta a uno de esos scopes, nunca a un scope raw omitido. Qualifiers conservan body, resolution, scope y cualquier propietario/adjunción; una variante no soportada bloquea. Contexto conserva namespace, selectores y ámbito de comparación completo, sin convertir ausencias en comodines.

El átomo principal usa la dirección implícita `{record_kind:"claim",local_id:"c0"}` y la solicitud normalizada `{record_kind:"request",local_id:"request0"}`; no hay otros claims de soporte en este subset. `atom` conserva también polaridad y todos los operandos del carrier original, sin deducirlos de `judgment` o `status`. Los IDs locales (`n0`, `s0`, `q0`, `r0`, etc.) son únicamente direcciones internas del grafo normalizado; la igualdad no depende de su escritura. Cada argumento se resuelve por su tipo de vocabulario: un string literal no se interpreta como referencia por coincidir con un ID.

`ConclusionNode` es exactamente una de estas formas:

```text
{kind: "bound_variable", node_id: string, variable: "v0", descriptor: ReferenceDescriptor}
{kind: "named_anchor", node_id: string, descriptor: NamedAnchorDescriptor}
{kind: "local_nominal_lexical_use", node_id: string,
 owner_node_id: string, descriptor: ReferenceDescriptor}
```

`ReferenceDescriptor` tiene exactamente `{denotation, declared_class, semantic_type, scope_id, alias_descriptors}`. Denotación/clase/tipo son sus carriers V3 completos sin evidencias, preservadas en origen; no solo labels o hashes. `alias_descriptors` reúne las restricciones descriptivas semánticas de todos los miembros Same: cada entrada tiene los mismos cuatro primeros campos, sin IDs raw, conservando diferencias y deduplicando únicamente entradas estructuralmente idénticas. Las restricciones incidentes de todos los aliases se conservan en edges/scopes/qualifiers y no desaparecen al deduplicar descriptores. `NamedAnchorDescriptor` añade exactamente `{domain_namespace, name_anchors}` a esos cinco campos; `name_anchors` contiene el contenido literal de todos los nombres acreditados del componente, con evidencia original en prueba. No es solo un `identity_sha256`. Menciones fuente y evidencias completas están en origen; no se las usa como identidad global del anónimo.

Solo el componente Same de **la respuesta real** produce `bound_variable`. Todas sus incidencias repetidas usan el mismo `node_id`; no se crean variables por rol. Cualquier otro anónimo es `not_supported`, salvo el uso léxico local target de una incidencia nominal válida. Ese target conserva `denotation=entity` del contrato documental pero su node kind de conclusión no tiene binder ni afirma existencia física de papel/hoja/lote. No se comparte entre owners. Los externos deben ser `named_anchor` con descriptor completo, clave histórica acreditada en prueba y todos sus links/aliases protegidos.

`ConclusionEdge` tiene exactamente `{edge_id, kind, from, to, role, link_kind, identity_kind, dependency_role, dependency_origin}`. `from` y `to` son `RecordAddress={record_kind,local_id}` con kind `request|claim|node|scope|qualifier|restriction`; los IDs se resuelven en el grafo. La unión cerrada de edges es:

| kind | Campos no nulos adicionales | Significado |
| --- | --- | --- |
| `role` | `role` | Incidencia del átomo/registro en un rol declarado, con toda repetición preservada. |
| `reference_link` | `link_kind` | Link dirigido de los kinds existentes y habilitados, incluido nominal_de; todos los aliases participan. |
| `dependency` | `dependency_role`, `dependency_origin` | Dependencia efectiva saliente con rol y procedencia conservados; nunca consumidores entrantes por coincidencia. |
| `identity` | `identity_kind` | Restricción local `same|different`; Same se resuelve por componente, Different conserva sus objetivos acreditados. |
| `scope_owner` | ninguno | Registro→scope efectivo que lo contiene. |
| `scope_parent` | ninguno | Scope→parent, sin perder la jerarquía. |
| `qualifier_owner` | ninguno | Registro→qualifier aplicado, sin perder propietario o adjunción. |

`dependency_role` conserva exactamente uno de los ocho roles V3 para aristas explícitas: `identity|guard|configuration|temporal|binding|premise|normative_target|branch_membership`; `dependency_origin` vale `explicit`. Para requisitos implícitos del lector efectivo que no tienen fila Dependency propia, el rol técnico es `implicit_operand` y el origen `implicit`; el registro normalizado conserva íntegro el operando que lo origina. Una arista implícita nunca sustituye ni deduplica una explícita por compartir extremos. Un campo u operando sin regla competente da `not_supported`. Ambos campos son null en las otras variantes.

Los demás campos de la unión son `null`, nunca ausentes. `role`, `link_kind` e `identity_kind` solo aceptan IDs/variantes declarados y habilitados, no texto libre. Referencias a registros no representables, aristas incidentes que exijan otro anónimo o una restricción no soportada bloquean. La multiplicidad significativa se conserva; no se transforma repetición de roles en un conjunto. Un componente Same se refleja por un solo nodo con sus descriptores/links reunidos y origen completo; no se exige una arista same entre cada alias raw para comparar grafos. Different nunca se infiere de falta de igualdad. Un Different interno al componente o una incompatibilidad de aliases bloquea.

La normalización recorre el **cierre efectivo saliente completo también de named anchors**, incluidos scopes, descripciones, qualifiers, enlaces y restricciones de todos sus aliases. No toma una clave histórica o un nombre coincidente como permiso para omitir restricciones. La conclusión conserva la estructura completa; el hash no ocupa el lugar del grafo.

### Comparación determinista y diagnósticos

Se construye un candidato entero por witness elegible, se agrupan candidatos por isomorfismo tipado con variable `v0` fija y anclas externas acreditadas, y se ordena la serialización estructural mínima bajo los renombrados locales permitidos. Los conjuntos semánticos se ordenan, las secuencias/multiplicidades no. Cada tentativa y copia consume presupuesto: no hay enumeración ilimitada, ni coincidencia de hash como decisión final. Si el procedimiento no puede decidir dentro del límite, queda `limit` y no una equivalencia aproximada.

`ProjectionDiagnostic` tiene exactamente `{code, source_reading_id, question_reading_id, request_id, support_witness_index, document_pointers}`. IDs de lectura, índice y pointers pueden ser `null` únicamente cuando el fallo es del inventario/envelope global; `request_id` siempre corresponde a la solicitud. `code` es uno de `not_supported|ineligible_document|incomplete_request|incomplete_witness|relevant_conflict|unresolved_dependency|different_conclusion|open_reading_set|invalid_origin|limit`. No se añade evidencia seleccionada a través de este canal. `not_supported` conserva su denominador y nunca cuenta como acierto de calidad.

Antes de Rust, root y crítico deben comprobar este esquema con controles de correlación repetida, named-anchor con alias/link extra, dependencia hacia metadata, descriptor independiente residual, origin-map cruzado y agotamiento compartido. Después se implementan validación, serialización y consumidores V2 juntos. No se cambia `reference_key`, `canonical_inner`, `Same` ni su interpretación; sin feature se exige replay V1 por bytes. La decisión de implementar queda separada de la existencia de este diseño cerrado.
