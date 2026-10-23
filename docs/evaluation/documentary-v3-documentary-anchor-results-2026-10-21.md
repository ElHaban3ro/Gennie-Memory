# Contexto documental con referente pendiente en V3

El registro «No consta el formato de las imágenes ya entregadas de Delta» no se conservaba ante la pregunta por formato de entrega y páginas digitalizadas. La comparación estricta encontraba diferencias entre estado documental y evento solicitado, entre owned_by/part_of y en la clase del referente Delta.

Se añadió `documentary_anchor_context`, optativo y versionado, para transportar únicamente la afirmación documental original bajo condiciones explícitas. No se transforma estado documental en formato ni en prueba de entrega. Tampoco se equiparan owned_by y part_of ni se declara identidad global del grupo.

El perfil identifica un enlace de relevancia documental existente, atributo, estado objetivo occurred, las dos clases de vínculo estructural y una configuración fuente obligatoria. La operación requiere dos grupos anónimos de la misma clase declarada y tipo compatible, con exactamente un vínculo directo cada uno y un referente padre con anclaje nominal compatible. Una clase explícita incompatible, otro nombre, raíz sin nombre o relación adicional bloquean esta vía. Los grupos no pueden tener anáfora o binding pendiente.

La configuración fuente se verifica como conocida, se conserva íntegra y solo se retira de la comparación restante cuando no es una condición explícita adicional de la pregunta. Permanecen los controles de contexto, ámbito, fundamento, premisas, dependencias, ramas y binders. El perfil no inventa la clase ausente del padre: identidad y ocurrencia del evento siguen pendientes.

La salida `partial_documentary_anchor_context` conserva answer_propositions con la proposición fuente y source_qualifiers originales, con answer_terms vacío. Declara link_kinds_equated, identity_proven, requested_event_occurrence_inferred, requested_value_inferred y unknown_value_inferred en false. No afirma que el registro corresponda con certeza al mismo grupo preguntado ni que «no consta» signifique desconocimiento del valor en el mundo. Es contexto documental calificado, no una respuesta completa.

## Comparación conocida

| Medida sobre los mismos 100 pares manuales | Anterior | Nuevo sin perfil | Nuevo con perfil |
| --- | ---: | ---: | ---: |
| Útiles no ambiguas /39 | 35 | 35 | 36 |
| None seleccionadas /60 | 0 | 0 | 0 |
| Respuestas completas | 3 | 3 | 3 |
| Referencia ambigua seleccionada /1 | 1 | 1 | 1 |
| Concordancia heredada /100 | 96 | 96 | 97 |

La ganancia es `cv3q_08/cv3_08_02`. Solo la primera parte de la pregunta obtiene contexto parcial; no se responde formato ni páginas. Se conservan las 35 selecciones útiles anteriores y las tres respuestas completas, sin nuevas selecciones none. La referencia ambigua anterior continúa separada y no cuenta como ganancia.97/100 es concordancia manual heredada, no exactitud automática; sobre99 no ambiguos es96/99.

Textos, frames y referencias de los100pares permanecen intactos; solo cambia el vocabulario y su hash. Ambos binarios sin el perfil reproducen exactamente los100informes anteriores. No se corrigieron owned_by/part_of en los frames ni se insertaron aliases para aprobar. El perfil declara una relación de contexto, todavía pendiente de revisión semántica independiente.

Quedan **tres evidencias útiles omitidas** (2B,1C): configuración de P2, ausencia de red en Malva y cautela de admisión de Duna con calificadores. La puerta global sigue fallando; no se eliminan esas filas ni se cambia su referencia.

## Controles y verificación

Catorce controles fijados antes de implementar incluyen dos positivos (Delta y un texto nuevo equivalente con Brisa), otro padre, padre con clase persona frente a expediente, otra clase del grupo, raíz sin nombre, estado objetivo distinto, entrega desconocida o pendiente, relación adicional, rama, fundamento incompatible, perfil ausente y otro atributo. Son desarrollo manual, mutaciones mecánicas y un texto del mismo autor; no revisión ciega. Trece fuentes y todas las preguntas son completas; una fuente es parcial deliberadamente por configuración desconocida. Ninguna es inválida. La versión anterior omite ambos positivos; la nueva conserva los dos y rechaza los doce negativos.

Ocho controles adicionales rechazan perfil con versión/enlace/atributo/estado/clase de vínculo no soportados, configuración vacía, fila duplicada y campo extra, con hashes recalculados. La comprobación compartida de configuración conocida se reutiliza también para mediciones, sin alterar su comportamiento. Ambos controles focales y la suite completa pasan sin modificar expectativas tras ejecutar la prueba.

Pasan **611 pruebas,24 omitidas**;89del motor/validador V3 y dos del adaptador CLI. Compilación y formato pasan. El hash final reproduce **387informes idénticos**:200pares candidatos,14controles propios,57controles anteriores y116validaciones congeladas. La auditoría del witness confirma proposición/calificadores íntegros, bindings de identidad vacíos, vínculos originales distintos y ningún formato, página o entrega inferidos.

Esta mejora no mide extracción automática nueva: la última permanece en **65/116fieles para ambos críticos**, con puerta fallida. La omisión de clases o relaciones por un extractor puede seguir causando selecciones indebidas; validar la forma o declarar un contexto no repara esa pérdida. Revisión independiente, extracción/utilidad completas, baseline, recursos, latencia, estabilidad y V6 siguen pendientes. No se reactivaron subagentes ni se cambió la política del producto.

Coste nuevo: **0peticiones API,USD0de proveedor**; coste/tokens Codex desconocidos. Acumulado intacto:6129operaciones,23campañas,USD4,565416508conocidos más20importes desconocidos,excluyendo Codex. Objetivo activo en paso3;sin adopción.

[Propuesta](../../outputs/iromem/evaluation/documentary-v3-documentary-anchor-2026-10-21/proposal.json), [protocolo](../../outputs/iromem/evaluation/documentary-v3-documentary-anchor-2026-10-21/protocol.json), [comparación](../../outputs/iromem/evaluation/documentary-v3-documentary-anchor-2026-10-21/report.json), [ejecución](../../outputs/iromem/evaluation/documentary-v3-documentary-anchor-2026-10-21/execution-provenance.json), [vocabulario](../../outputs/iromem/evaluation/documentary-v3-documentary-anchor-2026-10-21/vocabulary.json) y [controles](../../tests/fixtures/documentary-v3-documentary-anchor-controls-v1.json).
