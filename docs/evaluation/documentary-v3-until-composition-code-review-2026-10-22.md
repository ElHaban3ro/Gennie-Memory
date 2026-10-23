# Revisión técnica estática de composición con «hasta»

Fecha: 2026-10-22. Alcance: código en curso del evaluador; no se ejecutó Cargo, fmt, CLI ni APIs. Esta revisión no juzga fidelidad de fuentes, calidad de extracción ni rendimiento. No se consultaron paquetes de revisión semántica, gold, V6 ni controles CLI de root. Se leyó el contrato `documentary-v3-until-measurement-design-2026-10-22.md`, SHA-256 `020de639d3015d387a51e1589a2ef44bbb5f1dcaecd34b649139b4aecb0b9124`, y las reglas aplicables.

## Hallazgos y reparación observada

1. **Firma bajo renombrado interno — reparado estáticamente.** `canonical_inner` capturaba `m = v.as_object()` antes de reemplazar `v` por `canonical_ast(v)`. El recorrido posterior iteraba el objeto original; cambiar coherentemente los IDs del AST cambiaba la firma a pesar de preparar el árbol normalizado. Se observó después la obtención de `m` a partir del `v` preparado. Sigue pendiente comprobarlo mediante ejecución de root. La reparación conserva objeto, propietario y operandos: normaliza direcciones internas, no referencias documentales.

2. **Reserva de alias — reparado estáticamente.** `composition_anchor` construía los dos `Vec<String>` con `to_owned()` antes de reservar sus copias. Se observó después que ambas reservas preceden a la materialización. Sigue pendiente el control ejecutado de agotamiento global.

3. **Cobertura pendiente atribuida al owner — reparado estáticamente.** La revisión inicial detectó que Coverage podía emitir un issue global `/coverage` sin llegar a `dependency_missing` del owner. La versión revisada calcula por separado `graph::closure(source,claim,id)` y `graph::closure(question,request,id)` antes de autorizar selección, y recorre Coverage en ambos documentos. Toda fila no representada cuyo owner esté en el cierre impide la selección y conserva `source_coverage` o `request_coverage`; una fila sin owner también queda pendiente porque no está acreditada su independencia. Una fila representada no bloquea, y una deuda con owner fuera del cierre no introduce veto global. El cierre reutiliza dependencias efectivas, referencias, qualifiers, scopes, dependencias explícitas, Same y anáfora singular afirmada; no se reduce al ID del Claim. Las aperturas internas tipadas permitidas por el contrato no se convierten por sí solas en filas externas de cobertura pendiente. Sigue pendiente su comprobación ejecutada por root; no se afirma reproducción CLI en esta revisión.

## Fronteras inspeccionadas

El AST usa objetos cerrados, un operador, operandos tipados y resoluciones nulas; la región de adjunción apunta al core de la misma incidencia. El propietario explícito conserva un ref documental y su mención exacta; el posesivo mantiene su argumento abierto y no supera la selección por portador. La voz no identifica experimentador con propietario ni con sujeto implícito. Las citas se validan contra límites UTF-8 y se comprueban contención y orden; la coincidencia literal no prueba análisis lingüístico.

El perfil registra la feature undécima y exige `duration` declarado. El contador escalar se habilita también sin la feature léxica décima y usa el mismo presupuesto activo. El corte calcula consumidores efectivos antes de construir la vista ejecutable; los guards del lado fuente usan esa vista incluso cuando reciben el documento original. La proyección existencial rechaza el owner y los consumidores del carrier. Un hecho independiente sigue en la vista ejecutable. Esto es evidencia estática; las pruebas de dependencias, alias, guard y no interferencia corresponden a la ejecución posterior.

La selección exige el request quantity/duration/amount/fact/state, Same y restricciones de aliases en ambos sentidos; coteja qualifiers y configuración/tiempo del par. La salida nueva no publica cifras, proposiciones de respuesta ni bindings y transporta el cuerpo íntegro. El fundamento exacto permanece como deuda: no es una afirmación de haberlo probado. No se exige resolver adjunción ni sujeto implícito para seleccionar contexto parcial, pues eso contradiría el contrato.

## Límites de esta revisión

El código estaba en curso durante la revisión inicial. Tras el aviso de congelación, se inspeccionaron de nuevo las tres reparaciones en disco. No quedan hallazgos abiertos entre esos tres puntos; su reparación estática no equivale a una ejecución aprobada. No se certifica cobertura exhaustiva de consumidores, cierre de la campaña ni adopción del mecanismo. No se leyeron controles de root ni sus resultados de ejecución.

## Huellas de la segunda inspección

- `tools/evaluation/src/documentary_v3/linguistic_composition_matching.rs`: `311d68380c87566d5f6540ecb2f40eb231fca45a8431cbfa04b0f83cc787e2e9`.
- `tools/evaluation/src/documentary_v3_io/readings.rs`: `8da8de45dbb2e4896a5aed7272769626195a79d23cc3d7cacca0a8c083635d49`.
- `tools/evaluation/src/documentary_v3/lexical_clause_context.rs`: `2af2df9296eec61a912eb42a91e7d7b76affb7711a5671f19a78c475dac2daef`.
- `tools/evaluation/src/documentary_v3/validation/request_context.rs`: `d15ba1d851f1f6d0760a56bd8380c5e39239136696fd9953ae46523d518e7a88`.

## Tercera inspección: independencia de la respuesta ordinaria

La ruta inicial acumulaba candidatos abiertos del nuevo carrier en el mismo `Evidence` que decidía completitud ordinaria; así una respuesta directa independiente podía quedar incompleta por la presencia contextual de otra construcción. La reparación inspeccionada mantiene intacto el cálculo ordinario hasta después de `premise_refutations`, `constrain_answer_pattern`, `coverage_candidates`, `contradictions`, el corte por deuda del request, la extensión all_values y el tratamiento histórico de branch_choice. No elimina ni limpia deudas ordinarias para declarar completitud.

Solo después de calcular `complete` crea un acumulador separado, `composition_context`, y ejecuta la selección contextual. Añade sus parciales con el Claim completo; cada witness conserva `missing_constraints` propios y los arrays de respuesta/bindings vacíos. Cuando la prueba ordinaria no está completa, añade también los candidatos y deudas nuevos al request. Cuando ya hay prueba ordinaria completa, estos candidatos abiertos no la degradan. En ambos casos el bucle global sigue publicando los owners bloqueados y sus consumidores con `source_constraint` y `source_document` íntegros en `unresolved_candidates`: no desaparece la construcción del informe. Por ello el documento puede mantener estado global unresolved y tener una respuesta ordinaria completa, sin convertir el carrier en prueba de esa respuesta.

El movimiento no cambia la vista ejecutable: composición y consumidores siguen fuera de `executable_source.claims` y de `pending_claims`; la ruta parcial usa el documento original únicamente para transportar la construcción y comprobar su cierre. Un dependiente no vuelve a entrar por calcular la completitud antes del merge. La proyección existencial conserva su veto propio. No se inspeccionaron controles de root ni se ejecutaron pruebas. No se identificó un hallazgo abierto en esta reparación estática; no se presenta como prueba ejecutada de independencia.

Huellas en esta inspección:

- `tools/evaluation/src/documentary_v3/pair.rs`: `e99dc028447f7bbc210d0243ed94a1bf183196af54139ad8e48de1dd00980e9a`.
- `tools/evaluation/src/documentary_v3/linguistic_composition_matching.rs`: `0c5b2aa3cfe5eb996345f8c465427376849478b015f02028e9ec8dfc8c4c89c9`.
- `tools/evaluation/src/documentary_v3_io/existential_projection.rs`: `747c37805f7cb4d6f73a46946b03d2288c3cf3c49f753c214f61f6bda5be87e8`.
