# Revisión retrospectiva del banco r19

Fecha: 2026-10-23

## Hallazgo

La adjudicación ciega inicial contó como irrelevantes seis recuerdos que contradicen explícitamente la premisa de que Ana habló con Bob: `explicitly-excluded-recipient-does-not-rescue`, `explicit-spanish-recipient-exclusion-does-not-rescue`, `excluded-recipient-with-time-aside-does-not-rescue`, `other-than-recipient-does-not-rescue`, `parenthetical-recipient-exclusion-does-not-rescue` y `dash-recipient-exclusion-does-not-rescue`. Las exclusiones aparecen en la fuente como `not`, `salvo`, `other than` o `except`. Son evidencia para corregir el destinatario, pero no contienen la respuesta a qué dijo Ana a Bob.

La revisión crítica separada distinguió estos seis casos de las menciones de otro hablante o la inversión de dirección: una persona diferente que habló, por sí sola, no refuta lo que dijo Ana; tampoco lo hace invertir quién habló a quién. Esos casos siguen clasificados como irrelevantes. La primera auditoría tuvo inconsistencias en sus agregados; por eso este addendum se apoya en la revisión fila por fila y no reutiliza sus porcentajes como una nueva evaluación de rendimiento.

## Corrección preservando el histórico

`tests/fixtures/slot-evidence-v2/local-bank-r19.json` permanece byte por byte sin cambios: los seis `expected_selected` originales continúan vacíos. La capa separada `local-bank-r19-retrospective-overlay.json` los etiqueta como `refutes_premise`, con dimensión `recipient`. No se recalcula el resultado de r19 como si el overlay hubiera estado disponible entonces.

El overlay está ligado por SHA-256 al banco congelado (`4d3745403599924eb4a759879ed690ab78398b288dbfb80d34c06f37a5539e46`).

Esta corrección cambia la lectura semántica del resumen previo 22/22 y 28/28: de los 28 casos originalmente negativos, seis aportan corrección explícita de premisa y 22 siguen siendo irrelevantes. Los 22 casos seleccionados originalmente siguen siendo evidencia útil. El reporte anterior 50/50 queda conservado como histórico, pero ya no representa una concordancia semántica completa del banco.

| Métrica retrospectiva sobre las mismas 50 trazas congeladas | Resultado |
| --- | ---: |
| Evidencia útil para responder o corregir premisa | 22/28 recuperadas (78,6 %) |
| Casos irrelevantes seleccionados | 0/22 (0 %) |
| Precisión de las 22 selecciones | 22/22 (100 %) |
| Exactitud bajo el overlay corregido | 44/50 (88 %) |
| Refutaciones de destinatario omitidas por la traza congelada | 6/6 |

Estos valores son un análisis retrospectivo de un banco sintético ya observado; no son una cohorte independiente ni prueban una mejora respecto a la configuración vigente. La prueba del mecanismo con seis trazas retipadas mide solo el camino cuando `refutes` llega correctamente clasificado.

## Cambio local de mecanismo

El prompt de extracción V1/V2 ahora pide separar el contenido solicitado de la relación presupuesta entre hablante y destinatario (por ejemplo, «¿qué le dijo Ana a Bob?») y permite refutar la segunda solo con evidencia explícita. La normalización local ya no degrada una proposición `refutes` cuando está dirigida a una premisa marcada `recipient`, la premisa contiene un verbo de habla y una identidad distinta de la persona ligada. La misma cita marcada como respuesta del slot sigue degradándose ante la exclusión explícita. Una regresión usa los seis casos del overlay y comprueba ambos caminos.

La prueba demuestra la preservación estructural de una corrección bien tipada; no demuestra que un clasificador remoto detecte de forma fiable la premisa o elija `refutes`. La suite local pequeña y dirigida tampoco habilita TRAIN, evaluación ciega nueva, comparación con la configuración vigente ni adopción. Los gates remotos siguen cerrados mientras persista el HTTP 429 sin causa confirmada.

## Verificación de esta iteración

`cargo test --offline -p iromem-retrieval` pasó con 67 pruebas y cero fallos. La prueba focal del prompt V1/V2 pasó. `cargo test --offline --workspace --quiet` pasó con 984 pruebas y cero fallos; 24 pruebas quedaron ignoradas. También pasaron `cargo fmt --all --check`, `git diff --check` y la comprobación del overlay contra los 50 casos, etiquetas congeladas, seis citas literales y SHA-256. No se realizaron llamadas a proveedor: coste remoto US$0, latencia de clasificador no medida. La ejecución local del crate tardó 0,07 s; el comando de workspace terminó en aproximadamente 14 s, incluida su salida por crates.

## Contraejemplo crítico y corrección local

Una revisión adversarial encontró una falsa abstención en la iteración: con la consulta «What did Ana E-17 tell Bob B-29?» y la cita «Ana told Bob B-29, not Alice A-30, that rain was expected», el detector podía interpretar la exclusión de Alice como si invalidara a Bob. La puerta de ambigüedad también rechazaba el inciso aunque Bob fuera el destinatario preguntado.

La detección de exclusión ahora se limita al ID que una pregunta directa identifica como destinatario del acto de habla. Además exige que la persona ligada sea el sujeto inicial de la pregunta (con el tipo declarado como único prefijo permitido), así evita confundir «Who asked Ana to tell Bob?» con «What did Ana tell Bob?». La comprobación de complemento admite el inciso contrastivo posterior al destinatario solo cuando identifica un tercer ID estable; así mantiene el contenido sobre Bob sin rescatar la cita cuando la pregunta sí excluye a Bob. La preservación de `refutes` exige que la premisa referencie al destinatario pedido, no a cualquier persona mencionada.

Se añadieron regresiones basadas en el positivo congelado `useful-complement-about-foreign-person` del banco r21: se añade `not Alice A-30` a cita y resumen y se comprueba validez/selección; después se invierte la exclusión para Bob y se verifica la abstención. Otro control comprueba que una supuesta refutación sobre Alice no se conserva como corrección de una pregunta sobre Bob. La forma subordinada «Who asked Ana to tell Bob?» queda como control negativo del reconocimiento de destinatario. Los fixtures r19/r21 y sus etiquetas siguen intactos; estos resultados no cambian retrospectivamente los porcentajes ni prueban una mejora poblacional.

Tras el arreglo, `cargo test --offline -p iromem-retrieval` pasó con 71 pruebas y `cargo test --offline --workspace --quiet` pasó con 989 pruebas y 24 ignoradas. También pasaron `cargo fmt --all --check` y `git diff --check`. La suite del crate tardó 0,07 s y la ejecución del workspace, 11,1 s. Un test adicional recorre el JSON `CompactTraceV2` del adaptador hasta la normalización, validación y selección, incluyendo exclusión de terceros y refutación correctamente ligada. La revisión crítica independiente no encontró un hueco reproducible en ese cableado. No hubo llamadas remotas: coste US$0; latencia remota no medida. El test usa salidas sintéticas construidas localmente y no demuestra qué etiquetas produciría un proveedor. El HTTP 429 previo sigue impidiendo medir la producción remota de `refutes`, la evaluación ciega prospectiva y la comparación con la configuración vigente; no se promueve ni adopta este mecanismo.
