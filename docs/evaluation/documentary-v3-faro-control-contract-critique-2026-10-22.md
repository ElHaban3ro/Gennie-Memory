# Crítica del control contextual de la pregunta Faro

Fecha: 2026-10-22. Dictamen documental sobre una pregunta aislada y su candidata parcial; sin implementación, ejecución del validador ni comparación de pares.

## Dictamen

`actor:{kind:unresolved,reason:ambiguous_control}` **sí conserva una posición semántica positiva**: el campo es el agente de la acción subordinada `enroll`, vinculada por `purpose` al request de acompañamiento y con destino Faro. No equivale a perder toda la cláusula ni a `extraction_unresolved`. Conserva qué papel no se ha resuelto; no conserva una identidad o una relación de correferencia demostrada. El diagnóstico actual impide tratar la candidata como representación completa aprobada, pero ese bloqueo no demuestra por sí mismo extracción fallida.

El destino de una matrícula no demuestra autoridad normativa. La pregunta «¿Qué tutor debe acompañar a Ina para matricularse en el curso Faro?» permite conservar `course` como destino del objetivo. No basta para afirmar que Faro emitió la obligación ni para identificar automáticamente destino y ámbito normativo. La retirada de `normative_scope_filter:course` en la candidata está justificada frente a esa inferencia no acreditada. `null` conserva un alcance no explicitado: no prueba ausencia de autoridad y no autoriza aceptar cualquier ámbito como respuesta.

Un contexto de finalidad con ese alcance no explicitado debería poder **representarse** sin obtener elegibilidad de respuesta. El carrier actual tiene estructura para la finalidad, pero su restricción de soporte exige filtro normativo string y no admite esa combinación. Es una brecha del contrato operativo para representación conservadora, separada del control del infinitivo. No debe repararse inventando scope, usando `unspecified` para eludir un issue o suprimiendo diagnósticos.

## Material y aislamiento

Se revisaron únicamente los cuatro artefactos autorizados de `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/faro-question-purpose-design-v1/`: `design.md`, `candidate-input.json`, `rationale.md` y `receipt-candidate.json`; el contrato local V3; las instrucciones AGENTS y reglas backend/testing; y `tools/evaluation/src/documentary_v3/validation/request_context.rs`, truncado antes del primer `#[cfg(test)]` antes de mostrar su contenido. No se siguieron enlaces a otros informes.

No se leyeron `validation-current.json`, fuentes/contrapartes, gold, guía V6, matching, tests ni otras reviews. No se ejecutaron Cargo, APIs, validación o campañas. Los estados descritos del código son lectura estática. El recibo identifica la candidata como `partial_unvalidated_candidate`; esta crítica no convierte su hash declarado en auditoría de integridad ni confirma exposiciones de trabajos previos.

## Qué conserva la candidata

El request solicita el actor de `accompany(object=ina)`, con clase local tutor, modalidad requerida, fundamento solicitado normativo y extensión `one_value`. No afirma que exista un tutor determinado, que haya acompañamiento o matrícula realizados ni que esté probada una obligación factual. `claims:[]` resulta coherente con esa lectura interrogativa.

`context_bindings[0]` conserva la finalidad, el verbo de matrícula y Faro como destino. El agente subordinado tiene una posición explícita distinta del slot de respuesta del acompañamiento. Dejar esa posición sin referente no elimina el propósito ni convierte la matrícula en un hecho. Tampoco permite concluir condición necesaria, suficiencia, permiso o garantía de admisión.

La pregunta contiene dos participantes salientes, tutor e Ina, pero no explicita inequívocamente cuál controla el infinitivo. La lectura de Ina como quien se matricula es plausible; este material aislado no autoriza imponerla. Esa cautela tampoco demuestra que todas las lecturas posibles sean igualmente probables, ni que tutor/Ina formen una disyunción cerrada. La crítica no pretende resolver una controversia lingüística con el diagnóstico del validador.

El campo `action.actor` preserva la incidencia de agente y la razón tipada preserva la interpretación de incertidumbre propuesta. Por tanto, no hace falta inventar un referente existencial ni una variable global para registrar esta única incidencia. Una variable compartida sería necesaria si se debiera conservar una igualdad entre varias incidencias; no hay evidencia suficiente en esta pregunta para imponer esa igualdad.

## Tres ejes que no deben confundirse

| Eje | Pregunta que responde | Situación de Faro |
| --- | --- | --- |
| Completitud representacional | ¿Se conservan los operadores y papeles documentales exigibles, incluida la incertidumbre? | La candidata conserva el núcleo y finalidad; el contrato actual no admite scope null para este carrier y mantiene issue de control. No se acredita completitud aprobada. |
| Resolución referencial | ¿Se identifica quién se matricula y con qué evidencia? | No; se conserva una posición de agente no resuelta sin asignarla a Ina o al tutor. |
| Elegibilidad de respuesta | ¿Existe soporte independiente aplicable para publicar el tutor solicitado? | No se ha examinado soporte. Esta crítica de pregunta no puede otorgarla, aun si una futura representación fuese completa. |

Una representación fiel de una ambigüedad documental puede ser completa como representación y continuar sin resolución referencial ni respuesta. Esto es una distinción conceptual prospectiva: el contrato operativo actual sigue emitiendo su issue. En sentido inverso, un JSON que pase forma no prueba que la ambigüedad propuesta sea fiel al español ni que haya recogido todo el texto.

La cobertura de la candidata marca `matricularse` como `unresolved`. Es una forma conservadora de no esconder la deuda actual, pero no prueba que el verbo/acción se haya perdido: `enroll` sí está representado. Un futuro informe debería distinguir cobertura del operador de resolución de su participante; cambiar solo esa etiqueta no demostraría completitud.

## Evidencia estática del contrato operativo

En `ContextParticipant`, el código admite `reference`, `owner_slot` y `unresolved`. Este último distingue `unspecified`, `ambiguous_control` y `extraction_unresolved`. Emite respectivamente ningún issue de esa rama, `request_context_ambiguous` y `request_context_extraction_unresolved`. El comentario de `unspecified` lo interpreta como ausencia documental fiel, sin binding; no es un reemplazo apropiado para una ambigüedad reconocida con el único fin de pasar.

`ContextRelation` separa `purpose` de `necessary_condition`; la relación no resuelta conserva alternativas y genera su propio issue. La candidata elige `purpose`, sin ambigüedad de relación declarada. No procede transferir automáticamente a esa relación la deuda de actor.

En `document`, el request con `normative_action_relation` exige target normativo positivo/state, roles exactos action_kind/object, consulta answer/actor, filtro de clase, fundamento exacto normative y `normative_scope_filter.is_string()`. Por ello la candidata con null no cumple esa condición de soporte. Esta conclusión no procede de ejecutar el validador ni de leer su informe. El archivo también recoge referencias implícitas del carrier; conservar Faro como operando no exige convertirlo en autoridad normativa.

El contrato local mantiene separados obligación, permiso, prohibición y ejecución con alcance normativo explícito. También afirma que la deuda efectiva de un request impide publicación/selección fiable. Esa regla sigue vigente: registrar incertidumbre con mayor fidelidad no debe descargarla como evidencia aplicable.

## Cambio prospectivo mínimo y sus límites

No se justifica aquí un nuevo operador de control contextual. La forma existente ya distingue posición de agente, referencia explícita, control por el actor del owner y falta de resolución con razón. Falta acordar cómo valorar esa forma en los tres ejes anteriores y cómo transportar scope no explicitado. Antes de implementar habría que congelar una revisión versionada con estas reglas:

1. `ambiguous_control` significa que la representación conserva el papel de agente subordinado pero no establece enlace referencial. No afirma existencia, identidad, lista cerrada de controladores ni igualdad con el actor pedido. `extraction_unresolved` continúa señalando incapacidad de extracción y no se promueve a ambigüedad textual por conveniencia.
2. La representación de `purpose` no necesita una autoridad normativa inventada. El scope null significa únicamente no explicitado en esta pregunta. Destino y scope continúan en campos independientes. Admitir esa representación debe ser una combinación optativa y versionada; no una relajación global de null.
3. La disposición de representación y la de elegibilidad se informan por separado. Puede conservarse completa la estructura documental con resolución pendiente, manteniendo un bloqueo explícito de respuesta. Los issues originales y resultados históricos se conservan; no basta quitar `request_context_ambiguous` o `request_context_unsupported` para declarar aprobación.
4. El consumidor no puede satisfacer un requisito de scope mediante null, convertirlo en ausencia de restricción o completar el agente subordinado desde una respuesta atractiva. Un eventual criterio para resolverlo desde evidencia adicional requiere su propio contrato y pruebas; esta crítica no lo define ni lo autoriza.

Esa separación es una reparación válida de contrato si se limita mediante opt-in explícito a la ambigüedad **documental del actor ya tipado** y al alcance normativo no explicitado; no debe aprobar indistintamente todos los estados `unresolved`. Declarar cobertura representada exige conservar el constructor positivo completo —owner, relación, acción, papel, destino y evidencias— y una doble crítica del texto de pregunta que confirme esa lectura. El acuerdo de críticos no acredita respuesta ni independencia de familia. Una relación sin representar, una unidad semántica perdida o `extraction_unresolved` siguen siendo parciales; no se rescatan cambiando etiquetas de cobertura. Esta crítica individual no cumple por sí sola ese requisito doble.

La seguridad de la separación exige cortar la publicación de los consumidores con deuda antes de cualquier ruta histórica, conservando `Unknown` o deuda equivalente, sin bindings, respuestas directas/derivadas ni proyecciones que eludan el corte. Todos los canales deben compartir esa condición. Sin esta garantía la propuesta sí relajaría indebidamente la puerta. Es una exigencia prospectiva de diseño: no se inspeccionaron matching ni proyecciones para afirmar que ya ocurra. Feature ausente conserva exactamente el comportamiento histórico. La puerta global de 116 entradas y sus críticos permanece intacta y no se satisface aprobando esta pregunta aislada.

El coste principal sería introducir disposiciones separadas y propagación explícita de deuda, además de compatibilidad/versionado para el carrier con scope no explicitado. Es un cambio de contrato, no una corrección cosmética del JSON. Si esos estados no pueden distinguirse sin alterar consumidores, se conserva la candidata parcial con ambas deudas actuales.

## Pruebas críticas mínimas antes de cualquier implementación futura

Estas son obligaciones prospectivas, no pruebas ejecutadas ni observación de tests existentes:

1. Contraste positivo de representación: finalidad, enroll, destino y posición de actor sobreviven roundtrip con scope null y control ambiguo; resolución y elegibilidad permanecen pendientes. Un sistema que rechaza todo no satisface este control.
2. Distinción causal: la misma estructura con `extraction_unresolved` conserva diagnóstico de extracción, sin certificarse como incertidumbre documental completa; `unspecified` no sustituye silenciosamente ninguna de las dos razones.
3. Control explícito en ejemplos nuevos: referencia explícita y owner_slot permanecen diferentes; el caso ambiguo no se normaliza a ninguna de ellas ni recibe una lista cerrada tutor/Ina.
4. Separación destino/scope: destino Faro con scope null no produce scope Faro. Un scope explícito distinto permanece distinto; ausencia, igualdad y diferencia demostrada no se intercambian. null nunca abre un camino de respuesta que antes necesitaba prueba normativa.
5. Semántica modal: purpose no satisface necessary_condition, suficiencia, permiso o ejecución. No aparecen matrícula ocurrida, existencia de tutor, respuesta o binding al completar únicamente la representación.
6. Aislamiento y deuda: un request con control/scope pendientes conserva su bloqueo y dependencias antes de legacy, en todos los canales y proyecciones; no hay respuesta directa, derivada, bindings ni conclusión existencial por atajo. Un hecho independiente no se borra. Mantener perfil histórico sin el opt-in reproduce la disposición anterior, incluidos issues y diagnósticos.
7. Integridad del carrier: evidencia UTF-8, IDs/scope inexistentes, dependencia del destino, propietario incorrecto, duplicación del contexto y agotamiento de presupuesto fallan de forma atribuible sin salida parcial promovida a éxito.
8. No vacuidad de completitud: constructor positivo completo con ambigüedad documental y doble crítica puede acreditar solo representación bajo el opt-in; quitar relación, destino o evidencia, sustituir por extraction_unresolved, o discrepar sobre la lectura conserva parcialidad. La cobertura no se aprueba únicamente por ausencia de issues. La matriz de 116 y sus juicios obligatorios no cambia por estos controles locales.

Para justificar un futuro tipo nuevo habría que aportar un texto donde dos o más posiciones deban compartir un controlador aún no identificado y demostrar que la forma actual pierde esa correlación. En tal caso el tipo deberá definir binder local, alcance, incidencias, evidencia y prohibición de identidad global antes del código. La pregunta Faro aislada no demuestra esa necesidad.

## Decisión conservada

La candidata aporta una representación parcial razonada y conserva incertidumbre; no acredita extracción fallida por usar `ambiguous_control`, tampoco acredita completitud operativa o respuesta. Se mantiene la deuda actual de control y de combinación con scope null. La revisión prospectiva puede separar esas deudas sin inventar contenido ni convertir null en comodín, pero necesita contrato y verificación nuevos. No hay dictamen sobre fuente/contraparte, calidad automática, utilidad del par, aceptación del banco de 116 ni adopción en producto.
