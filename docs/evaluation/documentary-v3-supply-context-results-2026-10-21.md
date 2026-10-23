# Contexto de abastecimiento y restricciones cerradas en V3

Se declararon dos enlaces dirigidos de contexto parcial: `receives_from→upstream_origin`, ligado al destino y transportando source; y `supplied_by→receives_from`, ligado al destino y transportando supplier. Son hipótesis semánticas explícitas de este dominio, de un salto, sin inversión ni composición transitiva. No declaran equivalencia ni prueban que un depósito sea una tubería o que cualquier bien recibido sea agua. Requieren revisión semántica adicional y contraejemplos de otros bienes antes de adopción.

El código conserva ahora una restricción positiva cerrada con existencia afirmada y miembros no vacíos como `source_constraint` íntegra. No publica una proposición por miembro ni un término de respuesta. La variable answer_role no puede actuar como binding fijo del enlace. Negación, existencia no afirmada, conjunto vacío, rama, fundamento o enlace ausente no habilitan esta vía. Se mantienen los caminos reservados de admisión y estado documental por separado.

Se corrigió además la comprobación de contexto parcial para respetar los bindings declarados, incluidos sus patrones/anclajes, en lugar de exigir todos los argumentos de la pregunta bajo nombres de rol idénticos en predicados distintos. Los roles de destino no ligados quedan en `unbound_target_roles` y `unbound_target_context`; no se completan. En Nardo y Azucena se conserva solamente la relación de alimentación original: sustancia y origen permanecen sin demostrar. Los controles restantes de contexto, ámbito, dependencia, rama, fundamento y premisas siguen operativos.

## Comparación emparejada

Los mismos 100 pares manuales y referencias heredadas se ejecutaron en cuatro variantes. Los frames y textos se conservan; la ampliación solo cambia el vocabulario y su hash.

| Variante | Útiles no ambiguas /39 | None seleccionadas /60 | Ambigua seleccionada /1 | Completas | Concordancia heredada /100 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Código anterior, vocabulario anterior | 29 | 0 | 0 | 3 | 89 |
| Código nuevo, vocabulario anterior | 29 | 0 | 1 | 3 | 90 |
| Código anterior, vocabulario ampliado | 30 | 0 | 0 | 3 | 90 |
| Código nuevo, vocabulario ampliado | 33 | 0 | 1 | 3 | 94 |

Se conservan las 29 selecciones útiles previas y las tres respuestas completas, con cero pérdidas o nuevas selecciones none. Una ganancia procede solo de la declaración bajo código anterior; las otras tres requieren código nuevo y vocabulario ampliado. Las cuatro ganancias no ambiguas son Nardo (`cv3q_05/cv3_05_00`), Azucena (`cv3q_07/cv3_07_00`) y dos recuerdos de Olmo (`pcv1_08_02/claim_00` y `/claim_01`). Son contexto parcial y no nuevas respuestas completas. La fuente de Azucena contiene T6, pero esta nueva vía solo conserva la relación de alimentación; no se afirma haber resuelto su origen.

La restricción de Olmo ante la pregunta por la tubería (`pcv1_08_01/claim_00`) es la referencia ya ambigua A/B. Ahora se conserva completa como contexto parcial, sin afirmar exclusión de red ni convertir el depósito en tubería. Se audita separadamente y no cuenta como quinta ganancia no ambigua. Su cambio añade un punto a la concordancia heredada; 94/100 no es exactitud del producto. Sobre los 99 pares no ambiguos la concordancia es 93/99; aún sin extracción automática.

Quedan **seis evidencias útiles no ambiguas omitidas** (3A,2B,1C): exposición de P2 en otra configuración; autonomía de Cobre; ausencia de red en Malva; formato no documentado de Delta; admisión de Duna con calificadores; y medición no realizada en Lino. La puerta global sigue fallando. Las entradas y etiquetas originales permanecen intactas.

## Controles y verificación

Once controles finales exigen documentos completos y cubren proposiciones, lista cerrada de uno/varios miembros, negativo, vacío, existencia no afirmada, rama, fundamento, enlace ausente y binding indebido del answer_role. Un fixture inicial intentaba una restricción abierta que el esquema V3 no admite; otro rellenaba el slot omitido de la pregunta. Sus rechazos se conservan como fallos de fixtures y no cuentan como rechazo semántico. El control de binding se corrigió variando answer_role sin rellenar el slot. El fallo válido de Nardo por exigir argumentos no ligados motivó la corrección del mecanismo; las expectativas históricas no cambian.

Pasan **605 pruebas, 24 omitidas**; 83 del motor/validador V3 y dos del adaptador CLI. Compilación y formato pasan. Se reproducen **335 informes idénticos**: 200 pares candidatos, 11 controles propios, ocho de alcance anteriores y 116 validaciones congeladas. Los nueve controles de contexto calificado anteriores conservan comportamiento; los positivos incorporan únicamente el diagnóstico explícito `unbound_target_roles:[]`, por lo que no se afirma igualdad de bytes de esos nueve. Los informes nuevos y sus diferencias permitidas se conservan.

La auditoría de las cuatro ganancias y la selección ambigua confirma cero términos de respuesta, proposición o restricción original íntegra, sin miembros afirmados por separado. Nardo/Azucena conservan sustancia no ligada; no se fabrica una relación de origen. Esta comprobación mecánica no sustituye revisión semántica independiente ni prueba general de relevancia de los enlaces en otros dominios.

No hay extracción automática nueva: la última sigue en **65/116 fieles para ambos críticos**, puerta fallida. No se midieron baseline, recursos, latencia, estabilidad o V6 y no se reactivaron subagentes. Antes de otra extracción: revisar las seis omisiones y los límites de los enlaces con contraejemplos de contexto; después medir extracción, utilidad y recorrido completo, recursos/estabilidad y cohorte ciega.

Coste nuevo: **0 peticiones API, USD 0 de proveedor**. Coste/tokens Codex desconocidos. Acumulado: 6129 operaciones, 23 campañas, USD 4,565416508 conocidos más 20 importes desconocidos; excluye Codex. Objetivo activo en paso 3, sin adopción ni cambios del producto.

[Propuesta final](../../outputs/iromem/evaluation/documentary-v3-supply-context-2026-10-21/proposal-final.json), [protocolo](../../outputs/iromem/evaluation/documentary-v3-supply-context-2026-10-21/protocol.json), [comparación](../../outputs/iromem/evaluation/documentary-v3-supply-context-2026-10-21/report.json), [ejecución](../../outputs/iromem/evaluation/documentary-v3-supply-context-2026-10-21/execution-provenance.json), [omisiones](../../outputs/iromem/evaluation/documentary-v3-supply-context-2026-10-21/remaining-after.json) y [controles](../../tests/fixtures/documentary-v3-supply-context-controls-v1.json).
