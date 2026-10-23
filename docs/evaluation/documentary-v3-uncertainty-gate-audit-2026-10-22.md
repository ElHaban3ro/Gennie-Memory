# Auditoría crítica: incertidumbre fiel frente a pérdida semántica

Fecha: 2026-10-22. Estado: auditoría documental retrospectiva; sin nueva extracción, ejecución de validación ni puntuación.

## Conclusión

**La puerta vigente `116 complete + faithful` permanece fallida.** No debe aceptarse un documento por ser honesto acerca de su pérdida, por llevar una cita íntegra o por declarar `unresolved`. Esas propiedades pueden evitar una afirmación falsa, pero no reconstruyen la relación, condición o atribución que falta. A la vez, forzar una interpretación determinada cuando el texto no la determina introduce información y también falla fidelidad.

La salida defendible es conservar la incertidumbre real y la deuda del contrato con estados separados. Una eventual representación capaz de expresar una ambigüedad completa necesita un contrato nuevo, cerrado y criticado prospectivamente. Este informe no propone cambiar la puerta histórica, sus etiquetas, sus dictámenes ni su denominador; tampoco autoriza un nuevo pase o una puntuación más fácil.

## Alcance y evidencia

Se inspeccionaron `AGENTS.md`, `docs/rules/testing.md`, `tools/evaluation/src/documentary_v3_io/readings.rs`, la fuente vigente `tools/evaluation/src/documentary_v3/validation.rs`, `consolidated-extraction-contract-v1/root-dispatch-protocol.json` y `consolidated-failure-diagnosis-v1/{source-diagnosis,question-diagnosis}.json`, estos últimos bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`.

La ruta solicitada `tools/evaluation/src/documentary_v3_io/validation.rs` no existe; se localizó la anterior ruta vigente sin modificar código. Los diagnósticos son evidencia retrospectiva de problemas propuestos, no revisión independiente nueva ni demostración de que sus correcciones funcionen. Sus clasificaciones se citan como diagnósticos conservados: no se ratifican ni sustituyen los dictámenes críticos originales.

El protocolo congela 83 fuentes y 33 preguntas, exige las 116 estructuralmente completas y dos revisores por ítem fieles en `text_fidelity`, `completeness`, `attribution` y `operative_preservation`, sin pendientes ni faltantes. La utilidad está subordinada a fidelidad. El estado `ready_not_dispatched` del protocolo describe su registro de preparación; este informe no lo toma como estado vivo de la campaña ni reconstruye su ejecución. La condición fallida actual se conserva, sin recalcular porcentajes a partir del subconjunto diagnosticado.

No se consultaron particiones reservadas, gold, selecciones, APIs ni DB; no se ejecutó Cargo, clasificador ni nueva revisión. Solo se escribe este documento. La verificación es estática y documental.

## Qué significa conservar una incertidumbre

Conviene separar cuatro preguntas que no admiten un único flag:

| Pregunta | Evidencia necesaria | Lo que no basta |
| --- | --- | --- |
| ¿Qué dice y qué deja abierto el texto? | Span propio, alcance y alternativas justificadas sin información de contraparte | Una etiqueta `genuine_ambiguity` o la conveniencia para la respuesta |
| ¿Puede el contrato transportar ese significado? | Carrier tipado con propietario, relaciones, modalidad, condiciones y dependencias | La existencia de una key de nombre parecido |
| ¿Lo transportó esta entrega? | Caminos operativos efectivos en el raw y contraste crítico con el texto | Una mención nominal, cita o explicación externa |
| ¿Puede publicarse una respuesta? | Reglas aplicables y evidencia completa en todas las lecturas admisibles | Fidelidad documental o esquema válido por sí solos |

Una incertidumbre puede ser dato fiel cuando el propio texto no elige entre interpretaciones y la representación conserva exactamente esa indeterminación, lo afirmado en común y las restricciones de cada interpretación. No hace falta conocer la respuesta de una pregunta para representar fielmente su slot abierto. Tampoco hace falta identificar globalmente un referente anónimo para conservar su descripción local. Son ausencias del dato, no necesariamente pérdidas del extractor.

En cambio, es pérdida si desaparece una condición expresada, si queda solo en `mention`/`evidence`, si no se sabe a qué hecho modifica o si una hoja seleccionada elimina otra lectura relevante. `unsupported` puede describir correctamente esa pérdida; no la convierte en significado operativo preservado. Un fallo estructural que impide transportar el documento tampoco se convierte en incertidumbre lingüística.

Hay casos mixtos. La fuente puede ser ambigua y el contrato carecer del carrier de una de sus lecturas. Conservar el inventario abierto es más prudente que cerrarlo, pero sigue sin acreditar conservación operativa completa. Un contrato futuro debe impedir que la parte ambigua o desconocida absorba condiciones que sí son inequívocas.

## Lo que hoy comprueba el código y lo que no comprueba

En `readings.rs::validate`, un sobre exige campos exactos, hashes, de una a ocho lecturas, IDs válidos y citas por bytes. Más de una lectura exige una ambigüedad. `closure != closed`, dimensiones pendientes, ambigüedades abiertas o una hoja no completa producen `partial`; una hoja inválida hace inválido el conjunto. El reporte declara `semantic_fidelity_proven:false`.

Es importante mantener esa separación. Una ambigüedad cerrada con hojas completas puede obtener `complete` estructural. Esto no demuestra que las hojas sean interpretaciones permitidas por el texto ni que el inventario sea exhaustivo. Por el contrario, una única hoja con toda cobertura declarada `represented` puede validar aunque haya omitido una lectura: el diagnóstico de «bandeja de papel grande» ilustra ese riesgo semántico, sin que esta auditoría ejecute una reproducción.

`unresolved_dimensions` comprueba forma, cita y tipos básicos; la dimensión y el motivo son strings y los `affected_targets` son strings. Esa validación no prueba que el motivo sea lingüísticamente correcto ni que cada destino corresponda a una pérdida bien delimitada. No es un certificado de incertidumbre fiel.

En `documentary_v3/validation.rs`, cobertura distinta de `represented` añade una incidencia; ausencia de incidencias da `complete`, otras incidencias dan `partial` y determinados errores estructurales dan `invalid`. El código verifica literalidad de citas y estructura; no interpreta de nuevo el texto para demostrar exhaustividad semántica. Además, un candidato puede aparecer `eligible` mientras el documento global es `partial`, como advierten los diagnósticos. No puede usarse ese estado local como sustituto de la puerta global.

`readings.rs::classify_with_context` exige ambos lados completos para elegibilidad; recorre el producto de hojas, exige pedidos estables y solo permite respuesta completa cuando todas las celdas respaldan firmas coincidentes y completas. Conserva la matriz. Relajar la entrada por un flag de incertidumbre sin probar la propagación de sus dependencias rompería esta barrera. Tampoco basta el acuerdo entre hojas si todas repiten la misma omisión o si las firmas no pueden representar la condición ausente.

## Casos que separan pérdida, ambigüedad y error

Los siguientes son contraargumentos basados en los diagnósticos, no nuevas etiquetas:

| ID técnico | Frontera que hay que mantener |
| --- | --- |
| `d-b0eb2f1b48f42710c9cf6480` | «matrículas» no autoriza fijar la acepción de inscripción. Entrega, destino y asociación neutral pueden conservarse; el sentido abierto no permite inventar sentidos exhaustivos ni borrar entrega/destino. |
| `d-edc2cc8137740eb42150d567` | «sala baja» íntegra en una mención no liga operativamente `baja` al referente. Un descriptor neutral deberá conservar esa atribución sin fabricar planta, altura o dirección y sin decidir sin prueba si es posición o identificación. |
| `d-eeb265867d41ca3c7ebc7f3b` | «hasta apagarse» puede modificar la duración no medida o el intervalo de no medir. Declarar cobertura completa con una sola lectura oculta el problema; una cita literal no transporta el alcance temporal faltante ni demuestra apagado ocurrido. |
| `d-38fb24e622a073a293ead090`, `d-5d8e6d45d1ee52311cdfb8b4` | «en Nacar» exige conservar el complemento y su adjunción pendiente; el nombre no demuestra lugar, ajuste o modo. Un `context` genérico no conserva la misma incertidumbre si ya fija una de esas relaciones. |
| `d-79d8fc5f9965fe9d130446dd` | Cero páginas pendientes no especifica la tarea. Conservar cuenta y tarea indeterminada puede ser correcto; no permite completar con digitalizar/revisar. La deuda administrativa del `issue_id` no es la deuda semántica. |
| `d-f4152dc4613a39eccea79714` | «grande» en bandeja de papel puede atribuirse a referentes distintos. El tamaño de bandeja no sirve como tamaño de papel; dos hojas no bastan si una carece del descriptor completo. |
| `d-26600b62b77b674c11cdf612` | Confirmación pendiente no es estado documental, calendario futuro ni cantidad desconocida. La atribución especializada errónea y la falta de carrier neutral de estado de acción pueden coexistir. |
| `d-5b4860f3cf848e9dcbf9985a` | «su credencial de alumna» requiere posesivo y suficiencia referencial. Texto libre dentro de `class_suffices` no resuelve propietario ni autoriza transformar un individuo en clase. |
| `d-a54874431854cbe517a24364`, `d-1dc7dd73ded28e0da27a8682` | Necesidad normativa y finalidad de matricularse requieren ligadura entre actos, ámbito y participantes. El curso como ámbito no reemplaza esa ligadura; no se debe inventar alumno, matrícula ocurrida ni condición cumplida. |
| `d-f073f9c398966f95129a1e42` | «durante la ruta Nacar» limita la magnitud interrogada. `route` existe, pero su guard exige otro portador. El nombre de key y una asociación al reproductor no conservan la condición sobre autonomía máxima. |

Los targets incompletos de `d-1e1328d717a498cd87dd6cd9`, `d-458c56736909267cfa350f58` y `d-6422c131c9d10e8f50462111`, así como el binder incorrecto de `d-aca84f2d2cbee8a67701764b`, tienen una explicación distinta: los diagnósticos identifican mecanismos existentes para conservarlos. No hay justificación para un pase por incertidumbre que perdone roles omitidos o la variable incorrecta. Repararlos prospectivamente tampoco demostraría fidelidad de las restantes dimensiones.

## Condiciones para estudiar otro contrato

No se propone un catch-all ejecutable de texto libre ni un flag que dispense completitud. Una hipótesis futura defendible tendría dos mecanismos separados: carriers neutrales de relaciones realmente ausentes y una representación explícita del límite epistémico. El coste es ampliar esquema, validación, propagación de dependencias, reglas de comparación y revisión crítica; no es un ajuste de etiqueta.

La especificación futura deberá ser cerrada: campos y enums exactos, tipos de portador y alcance autorizados, relaciones y operadores definidos, reglas de composición y rechazo, y ausencia explícita donde corresponda. Debe distinguir al menos `source_ambiguity`, `source_unspecified`, `contract_unsupported` y `extraction_failure`; este vocabulario es propuesta, no perfil vigente. Toda dimensión debe tener span, propietario estructural, objetivos existentes y dependencias verificables. Un motivo textual puede explicar la deuda, pero no descargarla ni reemplazar un carrier.

Para una ambigüedad finita, exigir inventario completo de lecturas justificadas y cada hoja completa respecto de su interpretación, incluidas condiciones, modalidades, negaciones, cuantificadores, cardinalidades, desigualdades, posesivos y límites temporales. Para un inventario no demostrablemente exhaustivo, conservar `open`, sin respuesta derivada de la intersección de las hojas enumeradas. No presentar la intersección provisional como verdad común a posibilidades aún no representadas.

Una relación neutral puede preservar un complemento sin resolver su tipo solo si especifica qué permanece abierto y qué ya está fijado. No debe equivaler a `located_at`, `mode`, `owned_by`, identidad o procedencia por defecto. Una condición sobre una petición no afirma su cumplimiento ni crea un evento. Los slots sin dato siguen vacíos. Ningún mecanismo puede fabricar respuestas, miembros, identidad, origen transitivo, actor ni realización para cerrar un documento.

Antes de cualquier puntuación nueva: especificación y controles congelados por hash, crítica separada con posibilidad de rechazo, población y denominador fijados y procedimiento de revisión acordado. Mantener aislamiento fuente/pregunta y doble crítica sin contrapartes, etiquetas ni razones del autor; registrar que críticos de la misma familia no son validación humana ni independencia de familia. Toda evaluación posterior tendrá versión y resultado propios, conservando los 116 originales y cada fallo. Este informe no activa ese trabajo.

## Contraejemplos y pruebas mínimas prospectivas

Estas pruebas son requisitos propuestos, no pruebas ejecutadas ni fixtures incorporados:

1. **Cita intacta, vínculo ausente:** conservar toda la frase de ruta/finalidad/posesivo, quitar solo su arista operativa. Debe seguir fallando preservación; añadir `unresolved`, una razón o `semantic_fidelity` no debe dar pase.
2. **Deuda inventada sobre texto inequívoco:** tomar una condición explícita, marcarla abierta y omitirla del carrier. Debe distinguirse de indeterminación de fuente y bloquear completitud; de otro modo todo fallo se lavaría como cautela.
3. **Ambigüedad real conservada:** lecturas de adjunción con todas las condiciones y distintos propietarios correctos. Ninguna respuesta o identidad específica puede publicarse si difieren; borrar una hoja, abrir inventario o exceder límite debe bloquear cualquier cierre favorable, sin truncar.
4. **Hojas equivalentes por omisión:** dos lecturas sin la misma condición pueden coincidir en firma. Deben fallar fidelidad aun con matriz unánime y hashes/citas válidos. Se necesita revisión del texto, no solo comparación de hojas.
5. **Carrier parecido pero prohibido:** `route` sobre magnitud sin el portador admitido; `context` para un nombre sin categoría; `documentary_state` para confirmación; `paper_tray_size` sobre papel. La forma válida no debe legitimar el cambio de significado.
6. **Roles y bindings:** target sin `supplier`/`material`/`mode`, referencia local declarada variable del binder ajeno y eliminación de desigualdad o dominio. Deben fallar aunque el autor declare incertidumbre o mantenga la cita.
7. **Incertidumbre sin fabricación:** tarea no especificada, sentido de «matrículas», actor normativo impersonal y «en Nacar». Exigir preservación del núcleo afirmado y deuda exacta; rechazar completar tarea, actor, clase, lugar o modo por plausibilidad.
8. **Transferencia de condición y procedencia:** cambiar propietario, claim, rama, tiempo, referente homónimo, namespace o relación de suministro. Debe impedir respuesta/identidad/origen nuevos; recibir un objeto o estar asociado a él no demuestra posesión ni origen.
9. **Invariancia y cierre:** renombrar IDs conservando estructura no cambia el juicio; cambiar texto/hash/cita o usar destinos inexistentes se rechaza. Campos ajenos, enums desconocidos y motivos libres no deben abrir un fallback favorable. Positivos deben validar completos antes de interpretar negativos; fixture inválido no acredita rechazo semántico.
10. **Trazabilidad y no reescritura:** hash del contrato, guías, entradas, raws, validaciones y dictámenes; exactamente las filas esperadas, sin filtro por aceptadas ni reintento favorable. Prueba de que una evaluación nueva no reetiqueta la puerta histórica. Recursos y ejecución se informan aparte; desconocido no equivale a cero.

El mínimo operativo es mostrar, para cada dimensión disputada, la ruta desde el span al carrier, su propietario, dependencias y efecto observable al retirar o intercambiar solo esa condición. Esa prueba demuestra que la condición tiene función dentro de un mecanismo; la crítica semántica debe además comprobar que esa función corresponde al texto. Ninguna de las dos pruebas sustituye a la otra.

## Riesgos y decisión

El riesgo principal es optimizar la definición de éxito sobre los fallos conocidos: convertir pérdida en `source_unspecified`, aceptar frases enteras como carriers, llenar un esquema con relaciones neutrales sin semántica, o declarar exhaustivas lecturas elegidas por conveniencia. El riesgo inverso es castigar la incertidumbre honesta hasta inducir al autor a inventar una relación más fuerte. Ambos se evitan exigiendo por separado preservación de lo explícito, conservación de la indeterminación y ausencia de inferencias nuevas.

**Decisión de esta auditoría:** no conceder un pase más fácil ni recalificar los resultados existentes. Conservar `partial`/`invalid` y críticas originales donde corresponden al contrato vigente; investigar únicamente contratos futuros acotados si pueden superar los contraejemplos anteriores antes de otra puntuación. No hay evidencia nueva de mejora de calidad, extracción autónoma, utilidad automática o adopción productiva.
