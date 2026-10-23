# Contracrítica de contexto léxico de cláusula

Estado: dirección mínima justificada; contrato todavía bloqueado por cuatro precisiones enumeradas abajo. El propietario Claim permite conservar la condición positiva sin Body wrapper ni entidad de cláusula. No se acepta una fuente, un mecanismo ejecutado ni la puerta111/116. No hubo código, Cargo, API, matching o pruebas.

## 1. Material y alcance

Se leyeron el diseño `documentary-v3-lexical-clause-context-design-2026-10-22.md` y únicamente text/raw de `relative-interval-host-design-v1/candidate-input.json`, junto con su `design.md`. Se usa la crítica V1/V2 propia ya autorizada. No se abrieron resultados/validación, gold, preguntas, contrapartes, V6, otras críticas ni módulos del validador. Las incidencias de exposición histórica declaradas en las revisiones anteriores no desaparecen; esta lectura no añadió pruebas inline ni vocabulario embebido a su material.

La fuente actual de Zinc conserva apagado, sujeto e intervalo como un único átomo `action_undergoer_after_interval`. Su significado declarado es ocurrencia tras un intervalo con origen contextual no especificado. Esta revisión no vuelve al antecedente `duration_until_shutdown`, no afirma cien minutos de operación y no acredita que el predicado temporal esté validado o aceptado contra la fuente.

## 2. B1–B6 y mínimo arquitectónico

| Bloqueante | Dictamen de diseño |
| --- | --- |
| B1: guardar palabras sin restricción | Reparado conceptualmente. El contexto es condición positiva del compuesto, distinta del dueño desnudo. La salida conserva preposición, complemento y todos los operadores; no impone significado espacial ni modo. |
| B2: deuda autoral como indeterminación fuente | Reparado si la revisión externa acredita el modo exacto. `whole_clause_only` afirma adscripción externa sin inventar ambigüedad; `source_underspecified` exige revisión adicional y no puede elegirse al consumir. |
| B3: host e incidencias | Claim es host suficiente para esta adscripción externa. Acción/sujeto/intervalo siguen correlacionados; no hace falta exponer paths internos. La frontera Zinc `[0,56)` contiene la evidencia actual `[0,47)` y el PP. |
| B4: ownership/cierre | Pertenencia en context_bindings más cierre del Claim evita aristas recíprocas y host duplicado. La reserva de consumidores antes de leer Body es correcta como obligación; no está comprobada en código. |
| B5: guard/cuántificación/contexto | La admisión propuesta conserva operadores y limita positivos a fact/occurred/description bajo documento. Excluir guard/binder no equivale a borrarlos. Deben mantenerse sus estados y denominadores. |
| B6: bloqueo como única utilidad | Hay una operación positiva concreta: selección parcial del compuesto por predicado y restricciones conjuntas, sin valores de respuesta y sin elegir sentido/adjunción interna. La falta de wire preciso para ancla/revisión impide todavía demostrar que esa selección sea aplicable de forma inequívoca. |

Reutilizar una unión cerrada y el dueño existente es menor coste conceptual que QualifiedBody. No se necesita otra familia de entidades de ocurrencia, otro envelope, un parser ni una ontología preposicional. Sigue existiendo coste transversal real: validación, cierre, firma y reserva de rutas que podrían extraer Body desnudo. El tamaño pequeño del DTO no demuestra que ese coste esté implementado.

## 3. Cuatro precisiones imprescindibles

### 3.1. Fijar el universo del ancla de selección

`EvidenceSelectionIntent` contiene Term con refs por ID, pero no documento de origen, namespace de esos términos ni prueba de identidad recibida. Exigir «identidad existente, demostrada y localmente autorizada» no define cómo ejecutarla con ese wire. Si `ref(zinc)` viene de otro documento, el ejecutor no sabe si es el mismo Zinc, un homónimo o una referencia inexistente. Si viene del documento fuente, resolver ese ID es seleccionar una ocurrencia local, no probar Same entre documentos.

Contramodelo: dos documentos usan el mismo ID y nombre para reproductores distintos. Aceptar el intent por el ID hace falsa correspondencia; rechazar todas las refs porque falta el otro documento deja inoperante la selección positiva.

Reparación mínima: declarar expresamente que el intent de este primer alcance se resuelve únicamente contra el documento derivado seleccionado y que sus refs son localizadores de referencias de ese documento. Debe venir ligado a su hash derivado, y todos los IDs/evidencias/roles se validan allí. Esto permite seleccionar parcialmente una ocurrencia fuente ya presentada sin afirmar identidad externa. Una petición de otro universo queda fuera de ese contrato y no recibe correspondencia por ID o nombre. Si se desea selección entre documentos, hace falta transportar el lado, ámbito y prueba de resolución ya existentes; no llamarla implementada mediante una frase genérica. No ampliar ambos alcances a la vez.

La validación de Term debe precisar que «cerrado» excluye variables, anáforas no resueltas y referencias fuera de ese universo; no basta que el JSON tenga forma Term. Los términos descritos tampoco pueden abrir una ruta de razonamiento que el perfil no admita.

### 3.2. Cerrar la aceptación y su relación con las deudas conservadas

El registro de revisión usa estados accepted/pending pero deja `unresolved_debts: ID[]` sin universo de resolución, incidencia afectada ni regla para registros contradictorios. Un registro puede declarar cuatro accepted y conservar una deuda del mismo host; el raw actual conserva además coverage unresolved para «en Nacar». No está fijado si esa fila bloquea el positivo, si fue resuelta por review o si se ignora. El documento íntegro evita pérdida de bytes, pero no resuelve esa decisión.

Contramodelo: host_fidelity accepted junto con una deuda de adjunción externa del mismo contexto. Elegir el primer accepted y publicar produce soporte no acreditado; considerar cualquier coverage unresolved de todo el documento como veto impide conservar evidencia independiente.

Reparación mínima: definir los IDs de deuda como referencias únicas al registro de incidencias vigente, con owner/contexto o dependencia afectada, y exigir que toda deuda efectiva de ese cierre mantenga pendiente la selección. La falta de adscripción especializada o el origen temporal no dicho no son deudas artificiales si el alcance aceptado los conserva como subespecificación. Cuando una revisión resuelva una deuda de coverage de ese contexto, debe quedar un delta explícito ligado a la versión o una resolución estructurada inequívoca de esa incidencia; no ignorarla por un accepted global. El diseño ya permite correcciones declaradas: concretar esa regla antes de cualquier candidato positivo.

La revisión debe obtenerse de la etapa autorizada y comprobarse contra el manifiesto/contexto confiable. El mismo emisor del candidato no puede aportar un objeto accepted arbitrario y descargar su propia fidelidad. Hashes ligan versión, no autoridad. El campo `internal_attachment` de review debe comprobarse contra el modo exacto revisado; accepted no acredita implícitamente cualquier variante.

### 3.3. Preservar context_bindings anteriores en la adaptación inversa

La frase «retira únicamente el nuevo context_bindings» es ambigua: puede eliminar el campo entero y con él un contexto anterior, o retirar solo el miembro nuevo. El contrato propuesto admite exactamente un contexto en el owner y reconoce que existen otras variantes. Un host base con un contexto previo no admite sumar el nuevo sin violar ese alcance; tampoco se permite reemplazar el anterior silenciosamente.

Contramodelo: reemplazar un contexto nominal anterior por el léxico, retirar el campo completo al invertir y comparar solo Body. El host semántico habrá perdido su condición anterior aunque el predicado y roles sean idénticos.

Reparación mínima para el primer alcance: exigir owner base sin context_bindings previos, tratando ausencia y array vacío según su representación original exacta, y conservar sin cambios los contextos de otros owners. La proyección inversa retira solo la adición autorizada y reconstruye exactamente la presencia/ausencia anterior del campo, no una normalización favorable. Una futura combinación de variantes exige contrato separado; no borrar la existente para encajar el límite uno. Esta cota limita soporte, nunca justifica sacar el caso de111/116.

### 3.4. Cerrar salidas no positivas y contabilizar toda la operación

El witness positivo está detallado; «distinguir no coincidencia, aceptación pendiente, procedencia pendiente, forma no soportada y error» no especifica todavía una unión cerrada ni la precedencia cuando concurren. La promesa inicial de wire cerrado debe incluir también esos resultados. Un error de forma/límite no debe convertirse en no coincidencia o pending útil.

Reparación mínima: declarar una unión cerrada de resultados con tags y campos exactos. Solo la variante seleccionada contiene el witness positivo; las pendientes retienen propietario/contexto e incidencias efectivas sin respuestas, y los errores conservan su categoría estructural/operación. La no coincidencia solo procede después de validar los operandos necesarios, no cuando falta prueba de identidad. No hace falta otro sistema de reportes: puede utilizarse el contrato actual si se nombra y fija su mapeo exacto.

El contador único debe cubrir además intent y revisión, resolución de todas sus refs/deudas, verificación de ancla externa, hashes y copias en witness. Los límites32/4096 de evidencia deben aplicarse también a boundary_evidence y evidencias dentro de Terms del intent. Validar IDs únicos/no vacíos y tamaños de la revisión, ordenar/duplicados de matched_roles y cierre, y cobrar el recorrido para calcular el tamaño antes de copiar son parte de la operación; no preflight gratuito adicional. Los límites de infraestructura reutilizados no justifican otro cap favorable ni conteo separado de resultados excluidos.

## 4. Condiciones que ya están bien y no deben ampliarse

No se necesita especializar «en», ofrecer variables de subhost, buscar sinónimos de predicado, proyectar action_undergoer ni publicar el intervalo como respuesta. La selección por el predicado exacto y un ancla local puede devolver el único compuesto íntegro, con intervalo, lexema y complemento, de manera útil y honesta. Omitir roles del intent no significa que sus valores se hayan demostrado frente a una pregunta; significa que se devuelve la evidencia completa bajo condiciones conservadas.

El testigo puede incluir el documento íntegro para verificar cierre y citas, pero debe señalar que su único owner fue aceptado. Los demás claims y coverage retenidos no adquieren ese estado. Los endpoints del cierre se resuelven contra ese documento completo; no se aceptan como una lista de restricciones descartables. Aliases del host siguen participando en su cierre y correspondencia; la prohibición inicial de aliases del complemento es un alcance explícito, no una prueba de que sean distintos.

Las formas fuera de alcance, guardas y límites conservan su existencia y denominador. Un positivo sobre el alcance local especificado es necesario para evidenciar utilidad del mecanismo futuro; una batería solo de rechazos no lo prueba. Esta observación no exige ahora ejecutar matching ni construir preguntas.

## 5. Decisión

Devolver para los cuatro ajustes precisos; mantener la alternativa sin Body wrapper. Son cierres de contrato, no razones para añadir maquinaria general o bloquear perpetuamente toda evidencia. Una vez escritos y revisados esos ajustes podría considerarse listo el diseño del alcance local, con autorización de implementación separada y verificación pendiente. En su estado actual el contrato sigue bloqueado, especialmente por universo de ancla y resolución de deudas, aunque la dirección semántica y el dueño son adecuados.

No se acepta la fuente Zinc completa por su representación temporal, el registro de revisión o el nuevo carrier. No se revisan Cobre, reserva o Faro. No se declara aceptación111/116, extracción autónoma, utilidad medida, comportamiento del evaluador ni compatibilidad histórica ejecutada. Solo se escribe esta contracrítica.

## 6. Dictamen del delta congelado

Revisión acotada del diseño con SHA256 `7a9778f7ec17c8abbd12d251cce4ae04ad10f0b1c52a167f4fd0d03b5ace521f`, comprobado mediante hash local del archivo. Se revisan únicamente los cuatro ajustes solicitados y la costura CLI de sección9. Este addendum sustituye el bloqueo contractual anterior en esos cuatro puntos; no reabre la teoría ni evalúa implementación, pruebas o resultados.

| Ajuste | Dictamen y cierre concreto |
| --- | --- |
| Universo del ancla | Reparado. El intent incorpora `derived_input_sha256`; refs, evidencias y roles son exclusivamente locales al derivado. La selección entre documentos queda excluida, no inferida por ID/nombre. La coincidencia de aliases conserva sus restricciones locales. |
| Aceptación y deudas | Reparado. ReviewIncidence identifica origen, atribución y disposición; toda coverage unresolved/unsupported tiene una incidencia exacta y las deudas efectivas impiden el positivo. Un accepted global no borra «en Nacar». Registro, universo, modo interno y hashes se contrastan con el ancla autorizada externa. Cuatro accepted con deuda efectiva unresolved es invalid_review. |
| Preservación de contextos | Reparado. El owner base no puede tener contextos anteriores en este alcance; ausencia/array vacío se reconstruyen exactamente desde la base, sin revalidación retrospectiva favorable. Otros owners y todo el raw se conservan, y la inversión no compara únicamente Body. |
| Resultados y presupuesto | Reparado. SelectionResult es unión cerrada con precedencia: límite de operación, invalidez estructural, alcance no soportado, puertas pendientes y finalmente coincidencia. Intent, revisión, manifiesto, anclas, hashes, inversión, copias y salida pagan el contador compartido. No hay no_match para IDs inválidos ni falta de autoridad. |

La precisión `origin.coverage.owner=null` queda cerrada correctamente: localiza exclusivamente una fila con owner exactamente null y span idéntico; no es un wildcard ni acredita pertenencia a cualquier host. `affected_owner` sigue resolviendo un Endpoint real y su atribución está ligada a la revisión autorizada. No encuentro en ese delta el contramodelo de atribución indiscriminada que produciría un null tratado como comodín.

La sección9 concreta INPUT, base/derived, intent/review, TRUST_MANIFEST y OUTPUT separados. El manifiesto se recibe del runner confiable; su nombre no autentica el contenido ni el candidato elige su autoridad. Los hashes fuente son explícitamente de la triple canónica y los de revisión de su registro completo, sin autorreferencia. La inversión exige igualdad estructural exacta, y ancla ausente/contradictoria tiene tratamiento definido. El error técnico de I/O queda fuera de la promesa de OUTPUT válido, sin disfrazarlo de abstención.

Dictamen actualizado: **los cuatro ajustes quedan reparados en el contrato congelado para el alcance local autorizado**. No queda un bloqueo de diseño concreto de esta revisión que justifique demorar la implementación ya autorizada. Esto no verifica que el comando exista, que los consumidores preserven el contexto, que la revisión de Zinc sea aceptable o que111/116 cumpla fidelidad/utilidad. No se pide otro permiso ni se añaden nuevas condiciones teóricas.
