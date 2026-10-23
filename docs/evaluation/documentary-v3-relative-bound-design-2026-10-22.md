# Diseño mínimo de límite relativo de no ocurrencia — 2026-10-22

Estado: diseño prospectivo cerrado para implementación y controles; no acredita ejecución, extracción autónoma, fidelidad de una hoja, completitud de inventario ni adopción. Área afectada: contrato documental y herramientas de evaluación Rust; sin DTO de producto, SQL, proveedor, API ni motor temporal.

## 1. Evidencia permitida y viabilidad

Única fuente inspeccionada: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-author-corrections-v1/known116-bundle/d-eeb265867d41ca3c7ebc7f3b-input.json`, texto «No se midió la duración del reproductor Lino hasta apagarse.» y su vocabulario. La sección 3 de `documentary-v3-remaining-ambiguity-minimality-2026-10-22.md` se usa como propuesta sometida a crítica. Se inspeccionaron validación, presupuesto, dependencias, matching y firmas Rust vigentes; no se consultaron preguntas, counterpart, gold, V6, utilidad ni otro raw. No se ejecutaron Cargo ni API.

El raw propio declara evento `measurement/negated_occurrence`, sujeto `lino`, aspecto `duration`, método unspecified, resultado null y ningún tiempo. Conserva «hasta apagarse» como `configuration/terminal_condition` con valor textual. Su vocabulario ya declara `duration`, `duration_until_shutdown` y acción `switch_off`, pero carece de `request_context_profile`.

Ese carrier actual conserva la cadena y su adjunción al claim; **no codifica el contraste entre dimensión medida y alcance de la negación, ni el referente que se apaga**. `time` codifica role/value y `guard` una proposición, sin operador de intervalo relativo de no ocurrencia. Renombrar una clave de configuración no añade esa semántica a matching o firmas. No hay prueba de un carrier existente equivalente; se justifica un nuevo caso cerrado.

## 2. Semántica limitada y crítica

Lectura A: negación de medir la dimensión `duration_until_shutdown`. Lectura B: negación de medir `duration`, con límite relativo adjunto a esa no ocurrencia. El nuevo qualifier expresa exclusivamente B. No se combinan ambos mecanismos en el mismo owner en este perfil inicial.

Interpretación de B: el alcance temporal de la no ocurrencia de medición se presenta limitado por la condición designada «apagarse», cuyo sujeto controlado es el mismo referente que el sujeto de la medición. Inicio e inclusión del extremo quedan **sin especificar**. No corresponde a un intervalo cerrado/abierto calculable. No afirma que el apagado ocurrió, que se alcanzó el extremo, que hubo medición en él/después, ni valor, fecha o número de mediciones. Tampoco equivale a «nunca se midió».

El infinitivo reflexivo carece de sujeto nominal propio. Para este perfil mínimo se admite únicamente `boundary_subject == owner.body.subject` por ID exacto dentro de la hoja; no basta un referente existente en el documento ni una identidad inferida entre dos IDs. Un texto que requiera control distinto o ambiguo permanece fuera del carrier, con deuda explícita. Esta restricción limita capacidad, no demuestra automáticamente el control lingüístico: la crítica de autoría debe comprobarlo.

La construcción «no…hasta» puede sugerir pragmáticamente una ocurrencia de medición al alcanzar el límite. Este carrier conserva únicamente la no ocurrencia acotada; no adopta ni niega esa lectura positiva. Si la crítica independiente considera la ocurrencia terminal un compromiso semántico de una lectura fuente, hará falta representarla por separado o declarar deuda; no se la podrá borrar alegando que es solo pragmática. El carrier es viable sin resolver esa controversia, pero no autoriza certificar conservación completa.

Dos hojas distinguen el contraste de adjunción identificado, pero no certifican que la oración tenga exactamente dos interpretaciones posibles. La fuente sola no resuelve la lectura preferida ni la completitud del inventario. Una validación estructural verde no autoriza `complete` sin crítica independiente de conservación/inventario.

## 3. Forma y permiso exactos

Permiso: `request_context_profile.version = documentary_request_context_v1` y feature única nueva `relative_nonoccurrence_bound` en la lista cerrada existente. Mantener rechazo de duplicados, versión incorrecta, feature desconocida, campos extra y dependencias previas. Actualizar máximo de features de cuatro a cinco, sin hacer obligatoria ninguna feature adicional para este caso. El feature requiere `switch_off` en `fixed_action_rows` y `duration` en `fixed_attribute_rows`; no introduce equivalencias de atributos. El vocabulario opt-in cambia su hash; jamás reemplazar el hash de una entrada histórica.

Forma externa exacta, todos los campos obligatorios y sin adicionales:

```json
{
  "id": "bound",
  "scope_id": "root",
  "kind": "relative_nonoccurrence_bound",
  "body": {
    "relation": "until",
    "boundary_subject": "lino",
    "boundary_action": "switch_off"
  },
  "resolution": "known",
  "evidence": [{"start_byte": 47, "end_byte": 61, "quote": "hasta apagarse"}]
}
```

Los IDs del ejemplo son locales y no reglas de reconocimiento. `body` tiene exactamente esas tres claves: relación y acción son constantes; sujeto es ID string no vacío, máximo 128 bytes, de referencia válida. No null, listas, términos libres, `before`, `since`, fechas, eventos de límite, actor, variantes de inclusión/inicio o recursión. `resolution` solo `known` en este caso; una autoría incierta debe conservar candidato y deuda, no forzar este qualifier como completo. Evidencia es lista no vacía de citas literales válidas, con límites UTF-8 y los topes existentes; incluye conjuntamente el marcador de límite y la acción. La comprobación literal no valida su interpretación.

Sin feature, con perfil inválido o con feature distinta, rechazar el caso; nunca convertirlo a configuration/time. Puede interceptarse `Qualifier` en el validador opt-in antes de la selección genérica que actualmente deriva bodies desconocidos a `Time`; validar también todos sus campos externos, sin bypass de evidencia, IDs ni presupuesto. No ampliar el enum genérico para admitirlo silenciosamente en legacy.

## 4. Propiedad, alcance y dependencias

Cada qualifier tiene exactamente un owner, y cada owner como máximo uno de este kind. Única adjunción permitida: `claim.qualifier_ids` de un claim directo `body.kind=event`, `event_kind=measurement`, `status=negated_occurrence`, `aspect=duration`, `result=null`. No adjunción a scope, referencia, request, proposición anidada, otro qualifier, alternativa/branch wrapper, aspecto, evento positivo o planned. No reutilización del qualifier entre owners ni owner sin adjunción explícita. No herencia desde scopes o alternativas que cambie el owner.

Owner, qualifier y boundary reference pertenecen al mismo `scope_id`, en este perfil mínimo; no usar visibilidad ancestral como sustituto. Boundary reference es el mismo ID que `body.subject`; no `event_ref`. Las referencias y scopes siguen sus reglas habituales. Prohibir simultáneamente `terminal_condition` en ese owner evita duplicar el mismo límite con semántica genérica; también prohibir `duration_until_shutdown` evita mezclar A/B sin una fuente específica fuera de este perfil.

Aristas explícitas obligatorias, además de las ya requeridas:

- owner claim → qualifier, `role=configuration` (rol histórico de adjunción, sin equivalencia semántica con kind configuration).
- qualifier → boundary reference, `role=identity`.
- qualifier → scope, `role=binding`, para conservar contexto explícito del nuevo nodo.

Comprobar tipo, existencia y alcance de ambos extremos, rol correcto y ausencia de ciclos/duplicaciones incoherentes con reglas vigentes. No basta que el owner ya dependa del sujeto. El qualifier posee su operando y transporta sus fallos al owner. Registrar `boundary_subject` explícitamente en `validation::refs`, grafo, `dependency_missing`/operandos y todo recorrido relevante; el string plano no activa hoy el reconocimiento genérico `kind=ref`.

## 5. Matching mínimo, firmas y presupuesto

El carrier conserva representación; no incorpora razonamiento de intervalos. En esta primera implementación **ninguna vía legacy puede consumir este owner como negación global, resultado, refutación o respuesta temporal**. Un owner bounded no aporta evidencia positiva de medición ni de apagado. Aplicar el bloqueo también cuando aparece como peer, premisa, rama o dependency de un witness, no solo al claim inicial. No permitir que un fallback quite el qualifier o use solo aspect/status. La ausencia de feature/forma/dependencia no rescata matching favorable.

Puede emitirse contexto parcial dedicado si se acredita sujeto, aspecto y ámbito compatibles, conservando owner completo, qualifier, sujeto y request, etiquetado explícitamente como límite relativo sin resolver el extremo y con términos/proposiciones de respuesta y bindings vacíos. Si no se acredita compatibilidad, conservar candidato y deuda sin fabricar parcial favorable. Ese parcial no satisface un slot de valor, no se cuenta como refutación global y no se fabrica si la validación/dependencias bloquean el owner. Toda ruta de publicación debe revisar los owners consumidos. El diseño no exige nueva compatibilidad entre solicitudes ni nueva interpretación de preguntas; futuros operadores temporales requieren contrato y controles separados.

`qualifier_relation` no debe equiparar este kind a terminal_condition/time ni inventar igualdad semántica usando solo el literal. Mantener Unknown para intentos legacy de comparación es conservador. Las firmas de lectura sí deben preservar kind, relation, boundary_action, identidad canónica de boundary_subject, adjunción al owner, aspecto y status. Canonizar `boundary_subject` mediante `reference_key`, como `subject`; no comparar IDs literales entre hojas. Renombrar consistentemente IDs no debe cambiar firma; cambiar referente/owner/acción/relación debe impedir convergencia. Firma no construible implica indeterminación, nunca descarte del campo.

Reutilizar el presupuesto compartido de `context_budget::with_operation`: una operación exterior, sin reset por qualifier/hoja/witness. Cobrar validación, búsquedas de owner/aristas, traversal, firmas y copias. Mantener topes vigentes: 1 000 000 pasos, 16 MiB, preflight 100 000 nodos/profundidad64, evidencia32 por lista/4096 total, 512 claims+requests, ocho hojas, 64 combinaciones y 65 536 productos claims×requests. Agotamiento invalida la operación sin fallback ni resultados parciales favorables; conservar raw/hash según mecanismo existente.

## 6. Controles de aceptación proporcionales

Controles sintéticos locales de contrato, separados de la reautoría fuente-only y sin preguntas de benchmark:

1. A con `duration_until_shutdown` y B con `duration`+bound producen contenidos/firma distintos; ninguna afirma resultado ni apagado. Quitar bound sin quitar la cita pierde conservación aunque el raw legacy siga siendo estructuralmente válido.
2. Rechazar perfil ausente/invalidado, feature equivocada, campos extra, tipos incorrectos, acción/relación distinta, resolución unknown y citas inválidas. Legacy válido sin feature conserva comportamiento.
3. Rechazar owner inexistente/múltiple/incorrecto, segundo bound, adjunción indirecta, sujeto diferente/event_ref, scope distinto y arista ausente/rol incorrecto; defecto de boundary reference propaga al owner.
4. ID renombrado consistentemente conserva firma; referente distinto, adjunción trasladada o bound eliminado no convergen. Conservar kind/body íntegros en candidato y witness parcial.
5. Verificar que event slots, medición faltante, peers, premisas y alternativas no publican equivalencias/refutaciones legacy del owner bounded; no afirmar medición en/después del extremo. A y legacy quedan aislados de ese veto.
6. Agotamiento acumulado en validación/matching/firmas retorna límite, sin reset anidado ni rescate favorable; caso pequeño válido opera bajo el mismo presupuesto.

Estos controles verifican mecanismos y aislamiento; la crítica lingüística sigue pendiente para cualquier hoja autora. No se autoriza campaña larga, red ni promoción por este documento.

## 7. Revisión estática de la implementación inicial

Se leyeron `validation/relative_bound.rs`, la parte productiva de `relative_bound_matching.rs` y hooks de `validation.rs`, `graph.rs`, `request_context_profile.rs`, `pair.rs` y `documentary_v3_io/readings.rs`. Esta revisión no ejecuta Rust, no acredita resultados de tests y no inspecciona candidatas conocidas, preguntas de benchmark ni gold.

Conforme al diseño: permiso opt-in cerrado y prerequisitos del vocabulario; forma exacta y evidencia literal; source-only; owner único y directo; aspecto duration, negación y resultado null; sujeto ID idéntico y mismo scope; aristas explícitas; terminal_condition incompatible; canonicalización de boundary_subject; interceptación anterior a rutas legacy; filtrado de owners dependientes explícitos y wrappers por claim_ids; partial dedicado con copias íntegras, payload de respuesta vacío y compatibilidad conservadora; preservación de hechos independientes. Presupuesto usa la operación exterior y cobra recorrido/copias, sin reset propio.

**Hallazgo pendiente de cierre:** el punto fijo `bounded_nodes` de la versión inicial sigue solo `dependencies` explícitas y `body.claim_ids`. El grafo permite omitir aristas owner→reference/scope para carriers de request_context mediante `implicit_refs`, que también recorre el body de `normative_open_pattern`. Ruta estática concreta: B es claim bounded; referencia X depende explícitamente de B; C es normative_open_pattern con normative_scope=X y sin arista C→X. Esa omisión está permitida por la validación del carrier. C no entra en el conjunto filtrado. `dependency_missing(C)` incorpora operandos de context_bindings, pero no los del body normative_open_pattern; comparar normative_scope por identidad no equivale a comprobar la deuda de X. Esto deja sin garantía el aislamiento transitivo de ese canal. Cerrar el punto fijo sobre las dependencias implícitas autorizadas por validación y añadir control sintético del camino, preservando C independiente cuando no toque B. No se afirma un resultado ejecutado de esta ruta.

El hallazgo se comunicó al implementador y a root. Mientras no se cierre y verifique, esta revisión no declara cumplido todo el aislamiento. No se halló en lo leído una conversión explícita del bound a negación global ni un reset anidado de presupuesto; esto no sustituye controles ejecutados de todas las rutas.

### Cierre estático del hallazgo

Releída únicamente la corrección: `validation::effective_context_dependencies` delega al lector `request_context::implicit_dependencies` que ya usa el validador. `bounded_nodes` incorpora esas aristas para cada claim/reference/qualifier/scope, junto con dependencias explícitas y body claim_ids, en un grafo inverso. La clausura parte de owners/qualifiers bounded y visita solo sus consumidores transitivos; el conjunto de visitados corta ciclos y el presupuesto exterior sigue cobrándose.

El lector compartido recoge refs del body (incluido normative_scope), contexto/qualifiers, scope_id/parent_id y enlaces de identidad. Por tanto, en el camino señalado, B bloquea X y después C; si C usa un alias de X, la arista de identidad entra en el mismo recorrido. C ya no llega a las rutas legacy por esa omisión. El cierre resuelve estáticamente el hallazgo descrito; no es un veto global de todos los claims ni añade equivalencia temporal. No se encontró otro defecto en este delta acotado. Root informó nueve controles focales aprobados, incluida la regresión de normative_scope directo y alias; esa ejecución pertenece a root, no a esta revisión. El cierre estático no certifica fidelidad lingüística, completitud del inventario ni ejecución de workspace.
