# Revisión estática de indeterminación normativa

Fecha: 2026-10-22. Código inicial del módulo y hooks, antes de ejecutar controles. No se ejecutaron Cargo, APIs, matching ni pruebas. No se modificó código del implementador.

## Hallazgo que requiere corregir antes de pruebas

**Resolución `not_blocked` puede coexistir con extracción fallida.** En la versión inicial leída de `tools/evaluation/src/documentary_v3/normative_indeterminacy.rs:87`, `resolution_disposition.state` depende únicamente de `debt.is_empty()`. `debts` (línea56) recorre solo requests que cumplen `seed`; `seed` (línea29) reconoce actor ambiguous_control o scope null en un carrier normativo.

Contramodelo construido por lectura: feature activo, carrier normativo válido con scope string y actor `unresolved/extraction_unresolved`. La validación conserva `request_context_extraction_unresolved`, la representación queda partial, pero `seed=false` produce debts vacío y resolution_disposition `not_blocked`. Un documento invalid sin esas dos semillas puede recibir el mismo rótulo. Eso contradice una disposición general de resolución/elegibilidad bloqueada y la lista completa de deudas. No se observó una respuesta publicada: el matching contextual existente devuelve Unknown para ese actor. El defecto identificado es un reporte semánticamente engañoso, no un escape de respuesta demostrado.

Se notificó inmediatamente a `normative_indeterminacy_impl` y al coordinador. La corrección debe distinguir deuda de indeterminación admitida de otras deudas efectivas de validación, sin promover extraction_unresolved ni ampliar la whitelist representacional. Si el campo pretendiera medir solo deuda del feature, necesitaría nombre/alcance explícito distinto y un estado global de elegibilidad separado; `not_blocked` general no comunica ese alcance. Deben conservarse el hallazgo inicial y el contramodelo aunque el código se corrija antes de correr controles.

## Promoción representacional: condiciones observadas

`admitted` reconstruye modalidad, slot, extensión, clase no nula, fundamento exacto, scope string/null, acción positiva required/state, roles exactos, scope documental raíz sin binder/rama, contexto único y relación resuelta. Excluye premisas y qualifiers directos. La validación ordinaria sigue comprobando forma cerrada, evidencia, vocabulario y grafo; no se pretende que `admitted` sea otro validador completo.

`expected` atribuye `request_context_ambiguous` al path plural del actor y `request_context_unsupported` al path singular del request. `decorate` solo puede promover si no es invalid, cobertura no vacía con todas sus filas represented y todos los issues pertenecen a pares código/path de requests admitidos. Los issues originales permanecen en report. Esto evita el contramodelo de juicio fact/fundamento distinto que comparte unsupported: `admitted` no registra su path en allowed.

No se detectó por lectura un caso concreto que promueva ese contramodelo. Debe verificarse por comportamiento, además de relación unresolved, segundo contexto, coverage unresolved, ausencia de constructor, otro request con issue y variante extraction_unresolved. La evidencia exacta acredita forma, no la fidelidad lingüística de ambiguous_control; doble crítica fuente y gate116 siguen independientes.

## Corte operativo y cierre

El hook está como primera operación de `Engine::request` en `tools/evaluation/src/documentary_v3/pair.rs:5268`, antes de nominal possessive, relative bound, contextual y legacy. `pending` deriva semillas desde contenido, sin consultar status complete ni exigir que exista el issue del reporte. Construye cierre inverso de consumidores mediante dependencias declaradas y `validation::effective_context_dependencies` para requests, claims, referencias, qualifiers y scopes. Incluye relaciones de identidad que recoge el colector compartido; qualifiers se recorren como contenido y las referencias anidadas no se descartan por ser guard.

Para un request bloqueado devuelve support/refutation/partial vacíos, sin bindings ni términos/proposiciones de respuesta, y conserva request y documento completos en candidato pendiente. No sanea únicamente selected_claim_ids después de haber publicado witnesses. Otro request no alcanzado por el cierre devuelve None y conserva el dispatcher anterior. La lectura es consistente con la ubicación exigida para el corte; no constituye prueba de todos los patrones de alias/guard ni de no interferencia.

Se requieren controles efectivos: dependencia transitiva por qualifier/guard/alias, informe parcial sin issue esperado, scope null con actor explícito y otro request independiente. La clausura debe bloquear solamente los consumidores alcanzados y conservar hechos independientes. Nunca se deriva scope de destino ni actor de una respuesta candidata.

## Proyección y agregación

`tools/evaluation/src/documentary_v3_io/existential_projection.rs:1276` construye candidatos a partir de support de cada celda; el corte anterior deja ese soporte vacío. Además, `candidate` en línea756 exige coverage complete y en línea762 rechaza unresolved o missing_constraints. Su forma exacta de request en línea771 excluye el carrier contextual adicional. El recorrido ordinario leído no reconstruye una conclusión existencial desde el átomo de acompañamiento después del corte.

`pair.rs:5685` agrega selected_claim_ids desde support/refutation/partial, por lo que esas filas vacías no aportan IDs. No se detectó un escape concreto en esos consumidores leídos. Falta comprobarlo con la matriz real de lecturas y las llamadas a consume; no se afirma seguridad universal de APIs internas ni comportamiento verificado.

## Presupuesto y ausencia del feature

`decorate` y `pending` salen antes de recorrer documentos si `permits(FEATURE)` es falso. El registro del perfil exige normative_action_relation y admite el nuevo feature dentro de su lista cerrada. Por lectura, no hay un nuevo recorrido de documento ni copia activados sin feature; igualdad histórica por bytes sigue pendiente de reproducción.

`copied` precobra recorrido y bytes antes de clone; `pending` reserva colecciones/identificadores y usa el contador contextual existente para dependencias y propagación. Las copias anidadas de request/documento en el candidato se precobran por separado. Fallos devuelven None o finalizan decorate, pero el contador queda agotado: la seguridad global depende de que la frontera de operación contextual descarte el resultado entero. `context_budget::with_operation` comprueba exhausted al terminar, incluidas llamadas anidadas, y no reinicia el contador. La ubicación de decorate dentro de validate conserva esa frontera pública; no se ha ejercitado agotamiento en ejecución.

No se observó por lectura una copia grande nueva previa a su reserva dentro del módulo. Algunas colecciones pequeñas/strings de paths se construyen antes del cargo específico; están acotadas por IDs y número de requests y cubiertas por recorridos/cargos próximos, pero deben vigilarse con el control de límite global. Esta revisión no convierte una estimación conservadora de reserva en medición exacta de memoria.

## Alcance de la revisión

Se leyeron únicamente código de producción del módulo nuevo, sus hooks, perfil, presupuesto, colector compartido y fragmentos de los consumidores de matching/proyección necesarios. Cada lectura y búsqueda de código se truncó antes de `#[cfg(test)]`; archivos con test en el nombre fueron excluidos. No se abrieron fixtures ni sus textos, contrapartes, gold, V6, outputs u otras críticas para inferir resultados.

El formato está congelado en comentarios del módulo inicial como representation_disposition con state complete/partial/invalid y resolution_disposition con state blocked/not_blocked, deudas y flags de no inferencia. El hallazgo anterior exige precisar/corregir la segunda disposición antes de congelación final. La implementación puede acreditar estructura y corte una vez verificada; no acredita por ello fidelidad automática, resolución del tutor, utilidad del par, gate116 ni adopción.

## Addendum: relectura de la reparación congelada

Se releyó exclusivamente producción de `normative_indeterminacy.rs`, truncada antes de tests, después de que el implementador notificara la reparación y el coordinador comunicara fmt y controles focales en curso. Esos controles no se ejecutaron ni inspeccionaron desde esta revisión; no se acredita CLI ni resultados de pruebas.

**Hallazgo DTO cerrado por lectura.** En la versión reparada, `debts` (líneas161–175 al leer) añade cada issue restante como `existing_validation_issue`, conserva objeto issue/path íntegros y declara provenance validation_issue. Usa request_id null para no inventar atribución a un request. `decorate` (línea228) solo informa not_blocked si no hay deuda y el reporte no es invalid. Así, el contramodelo extraction_unresolved con scope string conserva ahora deuda y blocked; invalid también queda blocked. La disposición representacional continúa condicionada al constructor y whitelist exactos anteriores. Esto corrige la causa del reporte engañoso identificado; no demuestra el comportamiento por ejecución ni borra el hallazgo inicial.

**Profundidad y precobro reforzados.** `charge_depth` (líneas22–34) corta a profundidad mayor de64 y agota el contador contextual; `expected` (líneas108–131) reserva antes de construir paths; `decorate` (líneas186–194) reserva ocho veces los bytes de documento+issues antes de colecciones/copias, y el DTO final reserva su sobrecoste. Las deudas agregadas reservan bytes del issue antes del JSON que lo copia. Son garantías estáticas adicionales, no una medición exacta de memoria o una prueba del límite global.

**Nuevo hueco concreto de presupuesto de recorridos.** `debts` busca el issue de cada deuda reconocida mediante `issues.iter().find` (líneas141–143) sin cargo por elemento comparado. Después, para cada issue, hace `out.iter().any(|d| d["issue"] == *issue)` (líneas163–165), también sin cargo por comparación. Con N issues distintos no reconocidos, out crece y esa segunda búsqueda hace aproximadamente N(N−1)/2 comparaciones; el contador solo recibe un cargo por deuda añadida, además de los recorridos iniciales. El precobro de bytes no sustituye pasos acumulados. Cada igualdad también puede recorrer contenido anidado del issue.

Contramodelo de recursos prospectivo: documento opt-in con muchas incidencias válidamente serializadas y todas distintas. Puede consumir trabajo cuadrático sin aproximarse proporcionalmente al contador de steps, aun si el presupuesto de bytes acota N. No se ejecutó ni se afirma un tiempo de bloqueo observado. La reparación mínima debe cobrar cada scan/comparación antes de hacerla, incluyendo contenido comparado, o utilizar índices/identidades de issues reconocidos con recorridos y operaciones cobrados; no necesita modificar la semántica de deuda ni ampliar promoción. Se notificó al implementador y al coordinador. Este hueco queda abierto al cierre de esta relectura y requiere corrección antes de afirmar que todos los nuevos recorridos respetan el presupuesto.

No se releyeron fixtures, contrapartes, gold, V6 o outputs; no se modificó código. El cierre estático del DTO y la mejora de precobro no acreditan fidelidad, resolución, aceptación de116 ni validación del mecanismo completo.

### Evidencia vacía: segundo defecto pendiente de constructor

El implementador comunicó, después de la relectura, que la evidencia vacía del owner/atom/actionKind no se rechaza genéricamente. Se confirmó la causa por lectura de producción: `validation.rs:69` acepta listas vacías, y su dispatcher en líneas475/488 remite evidence a ese método; `admitted` leído no exige evidencia no vacía de esos tres nodos. El helper contextual sí exige evidencia no vacía para otros portadores, pero no cubre por ello todo el constructor.

Contramodelo: constructor normativo con scope null o actor ambiguo, coverage toda represented, y evidence del request o átomo principal vacía. Si no genera otro issue, la nueva disposición puede promoverlo a complete pese a haber perdido el anclaje positivo exigido. Debe exigirse evidencia no vacía de cada nodo positivo pertinente antes de admitir el constructor y conservarse la parcialidad cuando falta. No se arregla rellenando coverage ni añadiendo una cita ajena. El implementador espera ventana del coordinador para corregir y ampliar controles; esta revisión no acredita todavía esa reparación ni su prueba.

## Addendum final: cuatro reparaciones releídas y congelación

Tras aviso de freeze final del implementador se releyó la producción modificada. El coordinador comunicó 14 controles focales pasados de la revisión anterior; no se inspeccionaron sus fixtures, tests ni outputs, y esa comunicación no prueba por sí sola el código del freeze posterior. Esta revisión no ejecutó Cargo, fmt, CLI o APIs.

1. **Presupuesto de scans cerrado por lectura.** La búsqueda de issue en debts cobra por elemento; la deduplicación recorre y cobra los árboles de ambos issues antes de comparar. Los scans de scope, contextos y coverage y las copias de tuples para allowed tienen cargo explícito. La complejidad puede seguir siendo cuadrática, pero deja de ser trabajo invisible al contador acumulado; el agotamiento debe invalidar la operación en su frontera. No se afirma coste/latencia medidos.
2. **Evidencia positiva vacía cerrada por lectura.** admitted exige evidencia no vacía del request, átomo principal, action_kind del target y action_kind subordinado. La exigencia de roles exactos evita el pase vacuo del filtro de action_kind sin ese argumento. Se mantienen los controles previos de evidencia del carrier, relación, acción y participante del validador. El cambio no inventa citas ni declara fidelidad de ellas.
3. **Índices de deuda corregidos sin ampliar promoción.** expected enumera ahora cada context_index real y construye el path de actor ambiguo de esa incidencia. Un input parcial con dos contextos y ambigüedad en el segundo conserva `/context_bindings/1/action/actor` y su issue original, sin inventar deuda de posición0. admitted continúa exigiendo exactamente un contexto: corregir procedencia diagnóstica no convierte la forma múltiple en complete. Este defecto fue señalado por el coordinador antes del freeze y se conserva aquí como reparación adicional.
4. **Profundidad y disposición de resolución conservadas.** charge_depth mantiene límite64 y agotamiento global, expected precobra paths, y las colecciones/copias reservan antes de materializar. La disposición blocked incluye issues restantes y documentos invalid; extraction_unresolved no se promueve. Los campos actor_inferred y normative_scope_inferred permanecen false.

Al terminar esta lectura hay **cero bloqueantes estáticos conocidos dentro del alcance revisado**. Se comunicó el freeze al coordinador para continuar sus verificaciones. Esto no significa mecanismo validado, ausencia demostrada de todo defecto, prueba CLI, fidelidad del texto, cumplimiento del gate116 ni adopción.

Hashes SHA-256 de los archivos exactos presentes al leer el freeze; el hash identifica el archivo entero, incluidos bloques de tests que no se mostraron ni leyeron semánticamente. Un fmt o cambio posterior requiere actualizar la identidad antes de atribuirle esta lectura:

| Archivo | SHA-256 |
| --- | --- |
| `tools/evaluation/src/documentary_v3/normative_indeterminacy.rs` | `54917a06f860747a9a39a0e6a009bb02e9e7a955904d769a572258447de2bf60` |
| `tools/evaluation/src/documentary_v3/pair.rs` | `39ea841034b425514f7db6ea48810f65ed7c7f4e5de637ee734a5e104656d96c` |
| `tools/evaluation/src/documentary_v3/validation.rs` | `be0737d35d2be38da7c8371483dec1aad38fa7da65982565127c7e02485343db` |
| `tools/evaluation/src/documentary_v3/request_context_profile.rs` | `2ba206f2fe9381334db18dc713a4f23e92799c9087999970abb14de7dd9b4bfc` |
| `tools/evaluation/src/documentary_v3/context_budget.rs` | `829494d97dd14e8b0de02d88b7ff2543ac7b979fadb9254a43e04238020056f0` |
| `tools/evaluation/src/documentary_v3_io/existential_projection.rs` | `d53aa20ca76a9e65c311538b90b90a0e04341b71934bcd7ea10cd45082966e14` |

Se preservan los hallazgos, estados pendientes y reparaciones intermedias de los apartados anteriores como registro temporal; este addendum es el dictamen estático vigente de esta revisión. No se leyeron fuentes/contrapartes, gold, V6 o outputs para cerrarlos.
