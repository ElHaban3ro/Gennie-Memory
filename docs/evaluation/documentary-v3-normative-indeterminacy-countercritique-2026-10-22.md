# Contracrítica de indeterminación normativa

Fecha: 2026-10-22. Revisión previa a ejecución del contrato `documentary-v3-normative-indeterminacy-contract-2026-10-22.md` y de los puntos de entrada del matching existente. No certifica implementación, fidelidad, respuesta ni aceptación del banco116.

## Dictamen

Separar completitud representacional y resolución es válido bajo el opt-in propuesto. El contrato conserva el constructor positivo y permite registrar su actor ambiguo o un alcance no explicitado sin fabricar referentes. La deuda operativa continúa bloqueando. Esto no es una relajación indebida si ambos ejes tienen condiciones independientes y el consumidor recalcula la deuda desde contenido antes de cualquier ruta de salida.

La ejecución debe esperar hasta congelar forma cerrada del reporte y comprobar estáticamente tres condiciones: reconstrucción exacta del constructor, cobertura no vacía íntegramente representada y veto previo a todos los canales/proyecciones. No basta reconocer dos códigos históricos. La implementación de este feature todavía no se ha revisado en esta contracrítica; las ubicaciones siguientes corresponden a código existente leído antes de tests.

## Condiciones y contramodelos prioritarios

### 1. `request_context_unsupported` no identifica una sola deuda

Ubicación: `tools/evaluation/src/documentary_v3/validation/request_context.rs`, rama `document` → `normative_action_relation`. El mismo código se emite por incumplimientos distintos: modalidad, roles, clase, fundamento, tipo de documento o scope. Solo después de reconstruir la conjunción exacta del constructor puede atribuirse el issue concreto al scope null.

Contramodelo: conservar actor ambiguo y scope null, pero cambiar target a `judgment:fact` o fundamento solicitado distinto de normative. También produce unsupported; una whitelist de códigos podría convertirlo erróneamente en representación completa. Debe permanecer parcial/invalid según la validación aplicable. Lo mismo ocurre al introducir un segundo contexto, relación unresolved, rol extra o filtro de clase inválido.

La excepción debe comprobar todos los campos y restricciones del contrato, perfiles requeridos, propietario de pregunta, action positiva required/state, answer/actor, roles exactos action_kind/object, filtro de clase válido, fundamento exacto normative, contexto único, relación resuelta, símbolo registrado, destino válido y posición/evidencia del agente. Se excluyen binder/rama/guard/premisas y otras formas fuera del alcance. La disposición no puede reparar un input invalid.

Los issues eximibles se reconocen por código **y path de su causa reconstruida**, no por código global. Deben permanecer idénticos en el informe original, incluidos ID, path y procedencia. Un issue igual de otro request no hereda la excepción. Las deudas admisibles forman un subconjunto no vacío de actor ambiguo y scope null: actor explícito con scope null debe quedar bloqueado igualmente, sin exigir que siempre existan ambas deudas ni ampliar la excepción a actor `extraction_unresolved`.

### 2. Cobertura representada no puede aprobar un constructor incompleto

Ubicación contractual: secciones «Forma y semántica» y «Salida». Ambas condiciones son necesarias: constructor positivo completo **y** cobertura no vacía cuyas filas sean todas represented, sin issue de cobertura o de otra unidad. La mera operación `all` sobre una lista vacía no acredita nada. El alcance de la disposición debe ser explícito por request/documento; un segundo request defectuoso no permite declarar completo el documento entero.

Contramodelo: borrar relation o destination y dejar todas las etiquetas de coverage en represented. Debe fallar aunque los nombres de deuda coincidan. Contramodelo inverso: conservar todo el constructor y dejar la cobertura de matricularse unresolved. Debe permanecer parcial representacional, conservando la deuda; no se cambia cobertura automáticamente desde el nuevo reporte.

La cobertura exhaustiva y fiel frente al texto no queda demostrada por forma o etiquetas: la doble crítica fuente sigue siendo requisito independiente para contabilizar la corrección. `extraction_unresolved` y relación sin representar no se reclasifican como ambigüedad fuente. Tampoco se fuerza Ina, owner_slot o una lista cerrada de agentes.

### 3. El corte debe preceder al dispatcher completo

Ubicaciones existentes: `tools/evaluation/src/documentary_v3/pair.rs:5267`, `Engine::request`, despacha primero nominal possessive y después relative bound. `request_without_relative_bound` llega luego a request_context y finalmente legacy. Un veto situado solo dentro de `request_context_request` no acredita bloqueo de los canales anteriores. El nuevo corte por contenido debe precederlos o demostrarse común a todos ellos.

Contramodelo: request con scope null y actor explícito, informe parcial sin el issue esperado, y una ruta que intente responder al átomo principal omitiendo finalidad. Debe seguir pendiente con support/refutation/partial y bindings de respuesta vacíos, request y documento original conservados. No se permite resolver el propósito emparejando únicamente accompany o la clase tutor. Scope null no puede igualarse a Faro ni tratarse como wildcard.

`request_context_matching.rs:124` ya devuelve Unknown para participantes no acreditados en `context_participant`; eso es compatible con la deuda, pero no sustituye el corte general. El veto debe derivarse del contenido incluso si una API interna aporta informe parcial o no tiene el issue que normalmente emitiría validate. No debe depender de representation_disposition.status ni resolution_disposition.

### 4. Consumidores efectivos y agregación no heredan completitud

Ubicaciones existentes: `request_context_matching.rs:32` recorre dependencias declaradas para detectar carriers; `pair.rs:5685` agrega IDs desde support/refutation/partial. No basta con sanear selected_claim_ids al final: cualquier witness o respuesta derivada anterior ya podría transportar el dato. El corte debe producir filas sin esos canales antes de agregar, conservando solo el candidato pendiente.

Contramodelo prospectivo: un request ordinario depende transitivamente de un request con indeterminación a través de un qualifier o referencia implícita. No debe eludir la deuda por carecer de context_bindings propio. Se requiere cierre efectivo conforme al contrato vigente, no solo aristas explícitas. Otro request independiente conserva comportamiento; bloquear todo el documento por comodidad no es una solución válida.

La proyección existencial y los agregadores de lecturas deben comprobar esa misma deuda antes de completar. No se inspeccionó aquí su implementación; por ello este documento exige la comprobación y no afirma que ya exista. No basta una respuesta de pair limpia si un consumidor posterior vuelve a publicar el término del actor o reconstruye un subconjunto sin propósito.

### 5. Presupuesto y compatibilidad son parte del contrato

Ubicaciones existentes: `pair.rs:5585` abre operación contextual cuando corresponde; `request_context_matching.rs:74` precobra candidatos y copias. El feature nuevo no debe iniciar su recorrido cuando está ausente. Con opt-in, las copias de documentos, listas de deuda y recorridos del cierre comparten el contador y un límite invalida globalmente la operación; no degrada a éxito representacional sin materializar deuda.

Contramodelos: límite agotado justo antes de copiar documento/deudas y perfil histórico sin el feature. El primero no produce complete; el segundo conserva bytes/disposición histórica, incluidos issues. No se declara compatibilidad por simple ausencia de panic.

## Condición de congelación previa a controles

El contrato deja pendiente la forma exacta de `representation_disposition` y `resolution_disposition`. Antes de ejecutar deben fijarse campos, enums, cardinalidad, localización por request/documento, representación de paths y procedencia, y la regla para conservar issues originales sin duplicarlos ni renumerarlos. El campo de resolución debe expresar bloqueo/elegibilidad no acreditada sin inferir identidad o autoridad. La variante sin feature no añade estos campos.

La condición mínima de status representacional complete es la conjunción de constructor admitido íntegro, cobertura no vacía y toda represented, validación no invalid, ausencia de otras incidencias y solo las deudas exactas atribuidas al request correspondiente. El status de resolución sigue bloqueado por contenido aunque el primer status sea complete. Esta condición debe estar reflejada en controles positivos y en los contramodelos anteriores antes de considerar cumplida la separación.

Se comunicaron estas condiciones al agente implementador `normative_indeterminacy_impl`, sin editar sus archivos. Informó que preparará `normative_indeterminacy.rs`, reconstruirá forma exacta y hará veto previo al dispatcher con cierre efectivo. Esa comunicación es un plan, no evidencia de código o pruebas terminadas.

## Alcance y exposición incidental

No se ejecutaron Cargo, APIs, pruebas ni matching. No se leyeron contrapartes, gold, V6, otros textos fuente ni outputs en esta segunda revisión. Se leyeron el contrato autorizado y fragmentos de producción de request_context_matching.rs y pair.rs truncados antes de `#[cfg(test)]`.

Una búsqueda inicial de símbolos con exclusión glob insuficiente mostró incidentalmente nombres y líneas de tests dentro de archivos de producción y archivos terminados en `_tests.rs`. Se notificó al coordinador inmediatamente; esas líneas no se emplean como evidencia ni como controles existentes. Por ello esta revisión no se presenta como completamente aislada de nombres de tests, ni como crítica ciega del banco. Las lecturas posteriores se truncaron antes del marcador de tests.

La separación propuesta permanece aceptable **condicionada a estas garantías**. No hay prueba aquí de implementación, fidelidad automática, corrección contabilizada, utilidad, gate116 ni adopción. Cualquier incumplimiento de constructor/cobertura/corte requiere corregirse antes de ejecutar los controles, conservando resultados y raws históricos.
