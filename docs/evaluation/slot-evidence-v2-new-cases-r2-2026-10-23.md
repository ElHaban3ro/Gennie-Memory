# Cohorte sintética nueva V2 — revisión y validación local r2

Fecha: 2026-10-23. Estado: **fixture estructural local validada; sin ejecución de clasificador**.

## Alcance

Se congelaron once pares sintéticos nuevos en [`local-bank-r22.json`](../../tests/fixtures/slot-evidence-v2/local-bank-r22.json): cuatro respuestas, dos refutaciones de una premisa explícita, una respuesta parcial, tres ausencias explícitas y un caso de contexto no accionable. Diez trazas esperan selección y una espera rechazo. El caso 11 del preflight conceptual anterior sigue excluido: el binding V2 actual no representa la respuesta abierta «quién» como fue redactada.

Cada traza incluye ancla con tipo, sujeto e ID estable explícitos; targets copiados literalmente de la pregunta; citas de evidencia literales; y una revisión/referencia de candidato. Las citas y los targets se validan con el mismo código local del selector. La fixture se mantiene aparte del banco r21 existente.

SHA-256 de la fixture: `04cf0da78f00d9c5f6486012416b645d23c04b5278484739dac885e031ba4239`.

## Revisión ciega y adjudicación

Dos revisiones independientes recibieron las preguntas y resúmenes sin etiquetas. Ambas mantuvieron las diez respuestas previstas: la refutación del servicio de ferry, la corrección Cali→Palmira, la parcialidad del dato de sala y las ausencias explícitas se juzgaron de forma coherente con las citas. Ambas recomendaron no usar la respuesta de Teo para contestar por Alba. Una adjudicación fresca clasificó ese caso como `context_only`, no como `unrelated`, y confirmó que no debe seleccionarse para el slot de Alba. Se conserva esa corrección de etiqueta; la primera anotación `unrelated` permanece en el artefacto histórico r1.

En el caso del observatorio, ambas revisiones eligieron La Serena y reconocieron Valparaíso como propuesta de un borrador anterior. Esto valida la lectura humana de estos textos, no la capacidad del clasificador de distinguir versiones.

## Resultado local

`v2_new_holdout_bank_binds_useful_and_non_actionable_evidence` reproduce las once trazas contra el validador y el selector locales: **11/11 trazas válidas; 10/10 candidatos esperados seleccionados; 1/1 candidato `context_only` rechazado**. La prueba negativa `v2_literal_binding_known_false_acceptance_for_superseded_draft` construye una traza con la cita literal del borrador de Valparaíso: pasa la validación y el selector la selecciona. La suite completa `cargo test --offline -p iromem-retrieval` terminó después con **74 correctas, 0 fallidas, 0 ignoradas**; el harness reportó 0,07 s para las pruebas.

Estas cifras son una comprobación de contratos y selección con trazas escritas a mano. El falso positivo esperado en la prueba negativa confirma un límite actual; no es una aprobación de calidad. Los conteos no miden la extracción de un modelo y no deben presentarse como accuracy, recall ni mejora frente al baseline.

## Límite adversarial conservado

La cita literal del borrador que propone Valparaíso contiene la misma identidad estable de Sol que la nota final. Un clasificador podría citar ese fragmento y asignar `answers`; el validador estructural por sí solo no detecta que la propuesta está supersedida. La respuesta esperada sigue siendo La Serena. El caso se conserva como control semántico para cualquier corrida futura; no se considera resuelto por pasar esta prueba local.

## Métricas remotas y baseline

No se llamó a ningún proveedor para esta fixture: coste remoto **USD 0**, latencia de clasificador **no medida**, tokens **0 enviados** y salidas de clasificador **ninguna**. Por ello no hay puntuación comparativa G0/G2/D0 ni evidencia para cambiar la configuración vigente. El gate de tres smokes válidos consecutivos y la revisión ciega de las salidas reales siguen pendientes.
