# Contexto de medición no realizada en V3

El comparador no conservaba «no se midió» ante una pregunta por autonomía máxima porque el evento y la cantidad pedida tienen formas y atributos distintos. Se añadió un perfil optativo `measurement_context`, versión `negative_measurement_context_v1`, que declara esa relevancia sin convertir el evento en un valor.

La vía solo admite eventos measurement/negated_occurrence sin resultado, con fundamento description/observation, ante peticiones positivas fact/state de quantity.amount. El perfil relaciona explícitamente aspecto de evento y atributo pedido, exige configuraciones fuente/objetivo conocidas y permite bindings de etiquetas de contexto. El nombre de sujeto debe acreditar identidad o un anclaje nominal calificado; no bastan clase o coocurrencia. Se conservan fundamento solicitado, premisas, dependencias, ámbitos, ramas y calificadores restantes. No se aplica a asignación efectiva con fundamento incompatible ni a una ocurrencia, plan u otro aspecto.

Las condiciones declaradas se comprueban antes de consumir sus IDs para comparar los calificadores restantes. Los IDs y objetos originales siguen íntegros en el witness; no se borran para aprobar. Una etiqueta contextual fuente representada como referencia y otra como texto pueden corresponder por nombre bajo un binding explícito del perfil. Se registra `documentary_label` con `identity_proven:false`; eso no establece identidad global entre ruta y entidad.

La salida es `partial_measurement_context`, con source_event y source_qualifiers originales, answer_terms/answer_propositions vacíos y requested_value_inferred/unknown_value_inferred en false. **No medir no implica que el valor sea desconocido**, cero, máximo ni asignación efectiva. La operación sigue incompleta.

## Comparación conocida

| Medida sobre los mismos100pares manuales | Anterior | Nuevo sin perfil | Nuevo con perfil |
| --- | ---: | ---: | ---: |
| Útiles no ambiguas /39 | 33 | 33 | 35 |
| None seleccionadas /60 | 0 | 0 | 0 |
| Respuestas completas | 3 | 3 | 3 |
| Referencia ambigua seleccionada /1 | 1 | 1 | 1 |
| Concordancia heredada /100 | 94 | 94 | 96 |

Las dos ganancias son Cobre (`cv3q_04/cv3_04_00`) y Lino (`pcv1_control_17/claim_01`). Se conservan las33selecciones útiles anteriores y las tres respuestas completas, sin nuevas selecciones none. Los100pares mantienen textos, frames y referencias; solo cambian vocabulario y hash. Perfil ausente reproduce exactamente los100informes anteriores con ambos binarios.

En Cobre se transporta únicamente el evento c2 y su contexto Nacar. Los80minutos de funcionamiento no se publican como autonomía máxima ni se convierten en medida hasta apagarse. En Lino se conserva la condición «hasta apagarse» del evento. Ambas salidas siguen sin cantidad. Las dos filas del perfil son declaraciones semánticas del experimento, pendientes de revisión independiente y de extracción real.

Quedan **cuatro evidencias útiles no ambiguas omitidas** (1A,2B,1C): configuración de P2, ausencia de red en Malva, formato no documentado de Delta y cautela de admisión de Duna con calificadores. La referencia ambigua anterior se mantiene separada; no se cuenta como mejora nueva.96/100es concordancia manual con etiquetas heredadas, no exactitud del producto; sobre99no ambiguos es95/99. La puerta global continúa fallando.

## Controles y verificación

Dieciséis controles fijados antes de implementar incluyen tres positivos (Lino, Cobre y un texto nuevo con Lirio), otro dispositivo/atributo/aspecto, plan/ocurrencia, condición terminal ausente/desconocida/distinta, rama, fundamento de asignación, perfil ausente y ruta diferente/desconocida. Son controles manuales y mutaciones mecánicas; el texto nuevo no se presenta como revisión ciega o generalización. Catorce fuentes y todas las preguntas son completas; dos fuentes son parciales deliberadamente por condición/ruta desconocida. Ninguna es inválida. La versión anterior omitía los tres positivos; la nueva conserva los tres y rechaza los trece negativos.

Ocho mutaciones adicionales verifican forma cerrada, versión, IDs únicos, atributo conocido, configuración objetivo no vacía, bindings sin claves duplicadas/solapadas y campos/tipos inválidos, con hashes recalculados. El primer intento bloqueaba un candidato `conditional` aun habiendo acreditado sus condiciones. Se conserva el fallo y se permite ese estado únicamente tras las comprobaciones de configuración, contexto y dependencias; un pendiente no se transforma por sí solo en elegible.

Pasan **609 pruebas,24omitidas**;87del motor/validador V3 y dos del adaptador CLI. Compilación y formato pasan. El hash final reproduce **373informes idénticos**:200pares candidatos,16controles nuevos,41controles anteriores y116validaciones congeladas. La auditoría de los dos witnesses confirma evento/calificadores íntegros y ninguna duración, ignorancia o máximo fabricados.

La última extracción automática permanece en **65/116fieles para ambos críticos**, sin nueva medición. No se midieron baseline, recursos, latencia, estabilidad o V6 ni hubo revisión semántica independiente nueva. No se reactivaron subagentes ni se cambió producto. Antes de la cohorte ciega siguen pendientes las cuatro omisiones y la validación completa de extracción/utilidad/recursos.

Coste nuevo: **0peticiones API,USD0de proveedor**; coste/tokens Codex desconocidos. Acumulado intacto:6129operaciones,23campañas,USD4,565416508conocidos más20importes desconocidos,excluyendo Codex. Objetivo activo en paso3;sin adopción.

[Propuesta](../../outputs/iromem/evaluation/documentary-v3-measurement-gap-2026-10-21/proposal.json), [protocolo](../../outputs/iromem/evaluation/documentary-v3-measurement-gap-2026-10-21/protocol.json), [comparación](../../outputs/iromem/evaluation/documentary-v3-measurement-gap-2026-10-21/report.json), [ejecución](../../outputs/iromem/evaluation/documentary-v3-measurement-gap-2026-10-21/execution-provenance.json), [vocabulario](../../outputs/iromem/evaluation/documentary-v3-measurement-gap-2026-10-21/vocabulary.json) y [controles](../../tests/fixtures/documentary-v3-measurement-gap-controls-v1.json).
