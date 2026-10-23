# 0039 — Derivar localmente las citas de slot de la pregunta

## Contexto

Dos smokes ciegos V3 consecutivos llegaron a HTTP 200 y fallaron en referencias literales distintas. En V1 el modelo copió la pregunta en la cita de proposición. El prompt aclaró la procedencia de cada cita. En V2 la cita de proposición ya procedía del recuerdo, pero el modelo usó `from Seagrass Research Pier` —texto de memoria— como cita del slot y emitió el ID estable del sujeto (`M-63`) en `target_id` en vez del ID del slot. Relajar esas validaciones ocultaría la confusión.

## Decisión

En la ruta V3 compacta, `slots[]` transporta solo IDs. Rust deriva cada cita desde las palabras interrogativas literales de la pregunta y exige que el número de frases reconocidas coincida exactamente con el número de slots. Reconoce formas acotadas inglesas y españolas; deriva locuciones como `Which city`, `Where does` y `how many people`. Si una frase se repite, el conteo no coincide o no hay una forma reconocible, la traza se rechaza. Las citas de premisa siguen siendo citas literales emitidas por el modelo y validadas contra la pregunta.

Para conservar la comprobación de atribución, cada cita de slot incluye el ancla de consulta local más próxima después de la frase interrogativa y antes del siguiente slot; si no existe una posterior, incluye la ancla anterior más próxima. La cita sigue siendo un slice textual exacto de la pregunta. En un solo slot de respuesta (o una sola premisa de apoyo/refutación), Rust reemplaza `target_id` por el único ID compatible; con varios objetivos conserva el ID propuesto y la validación exige que pertenezca al conjunto correcto. El JSON Schema V3 ya no permite que el modelo suministre citas de slot ni desplazamientos.

## Evidencia local y límites

Una prueba nueva con el formato del fallo V2 deriva la cita `Which city does the marine biologist Inez Vale M-63`, el ID único `city`, conserva la cita de la oración fuente y selecciona el recuerdo correcto sin seleccionar la oración sobre Tomas. La batería cubre preguntas de uno y dos slots, orden, repetición, conteo incompatible, anclas múltiples, premisa separada y citas extra rechazadas. La suite completa del workspace pasa: 232 + 4 + 700 + 6 + 95 pruebas aprobadas; 25 ignoradas; cero fallidas. `cargo fmt --all --check` pasa.

Es evidencia de parser local, no obediencia autónoma ni calidad poblacional. La asociación del slot con un ancla usa proximidad léxica y puede abstenerse o enlazar mal en preguntas con subordinadas complejas; se conservan los límites del filtro de atribución y hace falta una revisión ciega nueva. La ruta V2 y el baseline no cambian. V3 requiere un fixture/manifiesto nuevo, la puerta de tres llamadas idénticas y después una cohorte ciega emparejada. No hay adopción por esta decisión.
