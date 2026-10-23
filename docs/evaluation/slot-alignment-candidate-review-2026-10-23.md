# Revisión crítica de la siguiente hipótesis de alineación

Fecha: 2026-10-23. Paso 3 del objetivo íntegro. Esta revisión se realizó después del rechazo de [contextual-token-rescue-v1](contextual-token-rescue-v1-results-2026-10-23.md); no modifica datos, etiquetas, métricas ni producto.

## Evidencia que motiva investigar

El fallo contextual terminó en **356/432**, con **171/200 útiles**, **47 ajenos** y **cero rescates C**, frente al padre **374/432**, 171 útiles y 29 ajenos. En el componente excluido, el AUC útil/none mejoró de 0,379 a 0,669, pero el umbral fijado siguió seleccionando cero útiles y 18 ajenos. Diez de esos falsos positivos compartían un nombre superficial con la pregunta. El hallazgo motiva estudiar la alineación del atributo pedido, pero no identifica una causa ni prueba que una nueva estructura mejore la selección.

Las variantes anteriores limitan la novedad: la instrucción directa V4 perdió seis útiles y añadió nueve ajenos en su prefijo ampliado; la revisión de fuente encontró 16/24 fichas fieles. En V2 de span/link hubo 3/12 fichas fieles; V3 tuvo 8/24 y dos ajenos nuevos. El verificador de relaciones entre tokens rechazó todos los 232 candidatos abstendidos, incluidos 29 útiles. La ablación del adaptador mostró complementariedad C, pero el estado sin residual añadió 128 ajenos y los 13 útiles C exclusivos estaban dominados por ajenos en ambos márgenes. Esos resultados no permiten presentar otra capa de atención, cita literal o selección más amplia como mecanismo nuevo.

## Propuesta y crítica

La propuesta exploratoria separaba atributo solicitado, argumentos y relación textual, y emitía evidencia con offsets y estados por pieza. La revisión crítica independiente **no la aprobó para implementación ni inferencia**. Señaló que:

- alinear solo slots de respuesta podría eliminar una refutación válida que no contesta el slot;
- `partial` no especifica si la evidencia parcial se selecciona o si el sistema se abstiene;
- elegir una única mejor proposición puede perder varias piezas válidas del mismo recuerdo;
- la política de composición entre piezas, slots, identidad y premisas no estaba congelada;
- el supuesto parser label-blind carecía de representación/algoritmo concreto, controles de negación, modalidad, tiempo y condiciones, y cobertura incierta;
- la diferencia frente a la atención pregunta-documento ya probada no estaba operacionalizada.

La crítica propuso como mínimo una matriz estática adversarial, antes de modelos: identidad separada de premisas y uno o más slots; apoyo, refutación, parcial útil, contexto y ausencia como estados distintos; varios hechos por recuerdo; coincidencia de entidad con predicado/rol equivocado; negación, modalidad y condiciones; trazabilidad de todo target de entrenamiento a evidencia fuente; y una política congelada que derive selección/abstención sin mirar etiquetas de evaluación. La matriz debe comprobar cobertura y fuga, no contarse como mejora de recuperación.

## Decisión y siguiente acción

No existe una candidata implementable aprobada a partir de esta propuesta. No se ejecutaron modelos, no se puntuaron los 88 conocidos ni las sondas, no se abrieron reservas y no se modificó el producto. La siguiente acción es especificar esa matriz y política sin usar IDs/roles/decisiones para producir features; someter el contrato y flujo de datos a crítica independiente. Si la regla no representa todas las rutas de utilidad del contrato o necesita gold/resultados para resolver una pieza, se descarta antes de inferencia. Solo un contrato que pase esa preflight habilitará una única prueba TRAIN congelada con las puertas originales intactas.

El objetivo permanece activo en el paso 3 (2/5 etapas cerradas). No hay mejora demostrada, candidata, validación fresca, recibo de recursos de producto ni adopción. El coste API de esta revisión es USD 0; el coste de trabajo Codex no está cuantificado.

## Iteración de la preflight estática

Se redactaron tres versiones de una matriz sintética nueva y se sometieron en secuencia a crítica independiente, sin editar el banco de calidad ni ejecutar modelos. V1 fue rechazada por una premisa no presente en la pregunta, estados parciales ambiguos, temporalidad y vínculo de identidad. V2 corrigió esos casos, pero todavía llamaba “silencio” a la ausencia de anotación, asumía identidad por nombre y duplicaba el contrato existente. V3 acotó el conjunto de candidatos, añadió anclas explícitas y reconoció que extiende `evidence-roles-v1`; la crítica identificó una última laguna: `not_observed_in_this_memory` requiere constancia de revisión exhaustiva de cada fuente y salidas esperadas congeladas. Por tanto, ninguna de esas versiones se declara lista para ejecutar.

La V4 añadió por memoria `source_coverage=complete|incomplete|unknown`, separada de la completitud de respuesta; congeló doce salidas y definió grounding de cita, destino de cada hecho, compatibilidad de alcance y evidencia explícita para `explicit_unmeasured`. `not_observed_in_this_memory` requiere cobertura completa para ese ID y alcance. La crítica independiente la aprobó **solo** como preflight estática de consistencia/estructura. El checker `validate-slot-contract` ya existe en el CLI: pasó los 12 casos y cinco pruebas unitarias focalizadas, además de formato. La prueba admite etiquetas de identidad/alcance no resueltas como candidatos ambiguos, pero no los cuenta como respuestas aceptadas. El recibo excluye explícitamente verdad semántica, calidad de recuperación, extracción automática y runtime. Esto no aprueba una política automática; el siguiente trabajo es diseñar una candidata runtime sin acceso a las anotaciones y medir TRAIN con las puertas originales intactas.
