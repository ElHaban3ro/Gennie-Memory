# Correcciones parciales de premisa y cautelas de admisión en V3

El comparador descartaba una negación explícita de autoría porque la pregunta añadía una clase ausente en la fuente. También omitía una nota de admisión negativa relevante para un modo solicitado. Se corrigieron ambas pérdidas sin completar el dato pedido ni inferir uso efectivo.

La comparación de premisas ahora conserva como `partial_premise_refutation` una proposición opuesta con predicado, roles y términos no referenciales idénticos cuando cada referencia pendiente tiene nombre/alias documental compatible. Clase e identidad pendientes se conservan como tales; no se demuestra corrección completa por coincidencia de nombre. La proposición negativa original se transporta íntegra, sin fecha ni valor inventado. Se rechazan otro actor, objeto o polaridad no opuesta, y se respetan alias explícitos.

La cautela de admisión necesita una relación explícita `admits→quantity`, con binding sujeto→sujeto y transporte de object, además del perfil existente de etiquetas de configuración. Solo se aplica a peticiones positivas de asignación efectiva con slot omitido. Una admisión negativa del modo solicitado puede aportar `partial_relevance`; una lista cerrada de modos admitidos conserva su restricción original si el contexto se acredita y la pertenencia del modo no está establecida. No se presume desigualdad entre entidades ni se concluye exclusión global por nombres distintos. Una admisión positiva/abierta no prueba uso; una lista de modos **no** admitidos no se interpreta como lista de admitidos.

Los witnesses de esta vía tienen `answer_terms:[]` y `actual_use_inferred:false`. Conservan la negación original o la restricción completa con miembros, cierre, existencia y calificadores. No se asigna resolución ni se afirma que un modo no pueda usarse efectivamente. Contexto, ámbito, ramas, dependencias y otras premisas pendientes bloquean la selección de esta cautela.

Se congelaron100pares manuales con textos, frames y referencias históricas V5 intactos; en la ampliación solo cambian el vocabulario y su hash. Cuatro variantes distinguen código y declaración:

| Variante | Útiles seleccionadas | Referencias `none` seleccionadas | Respuestas completas | Concordancia heredada /100 |
| --- | ---: | ---: | ---: | ---: |
| Código anterior, vocabulario anterior | 26 | 1 | 3 | 85 |
| Código nuevo, vocabulario anterior | 27 | 1 | 3 | 86 |
| Código anterior, enlace de cautela | 26 | 1 | 3 | 85 |
| Código nuevo, enlace de cautela | 28 | 1 | 3 | 87 |

La candidata conserva las26selecciones útiles previas y las tres respuestas completas, sin selección nueva entre las referencias `none`. Recupera `cv3q_10/cv3_10_00` (negación de que Nerea inaugurara Vado) y `cv3q_15/cv3_15_00` (Risco no admite Fino). La primera mejora procede del tratamiento de premisa bajo el mismo vocabulario; la segunda requiere el enlace declarado y el código nuevo. No se atribuye el resultado a metadatos por sí solos.

La lista cerrada de Duna (`pcv1_07_01/claim_01`) sigue sin seleccionarse: sus calificadores de modo/cantidad no acreditan el contexto requerido. No se eliminan ni reinterpretan como hechos incondicionales para aprobar. La vía cerrada solo pasa controles propios cuando ese contexto sí está resuelto. Persisten11referencias útiles no ambiguas omitidas (8A,2B,1C), una ambigua separada y la discrepancia de alcance universal anterior. La puerta global continúa fallando.

Pasan **601 pruebas,24omitidas;81V3**, dos controles nuevos con variantes. La primera prueba de premisa tenía un fixture inválido por faltar una dependencia; se conserva ese fallo y la reproducción válida con el binario anterior sobre el caso real conocido, cuyos dos documentos eran completos y cuya selección era vacía. Se corrigió el helper de dependencias del test. Se conserva además el fallo que interpretaba una lista cerrada negativa como lista de admitidos, corregido mediante control de polaridad. Las mutaciones de grafos se identifican como controles mecánicos, no extracción fiel.

El hash final reproduce316informes idénticos:100de cada variante candidata y116validaciones de extracción congelada. La auditoría de los dos witnesses recuperados confirma negación original intacta y ausencia de fecha, cantidad o uso efectivo inferidos.

No hay revisión semántica independiente nueva, extracción automática nueva, baseline de producto, recursos, estabilidad o V6. La última extracción sigue en65/116fieles para ambos críticos. La concordancia manual87/100no es exactitud del producto ni aprobación para adopción. Cero API/USD0nuevos; coste/tokens Codex desconocidos. Sin reactivar subagentes; goal activo en paso3.

[Propuesta](../../outputs/iromem/evaluation/documentary-v3-correction-context-2026-10-21/proposal.json), [protocolo](../../outputs/iromem/evaluation/documentary-v3-correction-context-2026-10-21/protocol.json), [comparación](../../outputs/iromem/evaluation/documentary-v3-correction-context-2026-10-21/report.json) y [vocabulario](../../outputs/iromem/evaluation/documentary-v3-correction-context-2026-10-21/vocabulary.json). La ejecución del hash final y auditoría de witnesses se registran en `execution-provenance.json`.
