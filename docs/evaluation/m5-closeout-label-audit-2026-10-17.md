# M5: auditoría ciega retrospectiva de etiquetas

Fecha: 2026-10-17. Estado: auditoría de contrato y semántica de los fixtures congelados; no es una campaña nueva, revisión humana ni validación independiente de rendimiento real.

## Alcance y procedimiento

Un agente distinto del autor de los fixtures leyó los recuerdos y preguntas completos de `conversation-m5-v2-development` y `conversation-m5-v3-development`, junto con sus README, auditorías y el protocolo v2. Se inspeccionaron los 99 positivos y los 37 negativos. No se abrieron diarios, respuestas de clasificadores ni informes de resultados. Tampoco se hicieron llamadas a proveedores. La ceguera operativa consiste en no abrir esos artefactos ni consultar el ID producido para elegir la relación semántica. La conversación heredada incluía resúmenes históricos de resultados y algunos ejemplos, por lo que no se afirma ceguera completa respecto de toda información anterior. La revisión es distinta de la autoría y sigue siendo automática, sobre datos sintéticos y sin adjudicación humana.

No se modificaron fixtures, etiquetas originales, manifiestos ni hashes. La [anotación retrospectiva](../../outputs/iromem/evaluation/m5-closeout-2026-10-17/negative-evidence-overlay.jsonl) contiene solo los 37 negativos, con `historical_expected_ids: []`, `provenance: agent_blind_fixture_review` y `human_review: false`. Las memorias indicadas se identificaron exclusivamente desde los fixtures. El archivo no constituye un reemplazo de `expected_ids` ni se usará para publicar un porcentaje superior.

## Hallazgo principal

La igualdad exacta con `expected_ids` mide concordancia con una etiqueta de selección. Por sí sola no distingue pertinencia, apoyo afirmativo a la presuposición de la pregunta, refutación de esa presuposición y fidelidad de una respuesta posterior. Una memoria que dice «no se firmó licencia» puede ser el contexto más útil para contestar una pregunta que presupone una licencia firmada.

Si el contrato del selector exige únicamente completar afirmativamente el dato preguntado, una etiqueta vacía puede describir la ausencia de ese dato. Si el objetivo es recuperar recuerdos útiles para una respuesta fiel, la misma etiqueta puede penalizar una refutación válida. El [protocolo v2](conversation-m5-v2-development-protocol.md) exige que un negativo no tenga ningún recuerdo aplicable y distingue candidatos de respuesta falsa. Los casos con evidencia negativa explícita exigen precisar qué significa aplicable antes de optimizar otra vez la abstención.

En v3, los doce negativos contienen una memoria que niega explícitamente el resultado presupuesto o documenta que todavía no existe. En v2, diez negativos contienen estado cercano: nueve tienen contexto parcial o provisional, y uno niega una selección en la reunión descrita. Los otros quince negativos v2 no contienen evidencia que responda el dato pedido. Estas cantidades describen la anotación del corpus; no son aciertos nuevos de ningún sistema.

## Ejemplos verificables

| Banco y caso | Evidencia del fixture | Interpretación |
| --- | --- | --- |
| v3, `cm5v3q_e9ad04e1c12b616e4183` | `cm5v3_a041e286152b2f96c320`: no se firmó licencia de emisión | Es pertinente para corregir la premisa; recuperarlo íntegro no demuestra una afirmación de firma. |
| v3, `cm5v3q_044d59c4471939147b1b` | `cm5v3_dce11f37068ffcd2da8c`: no se adjudicó el contrato a ninguno de los dos talleres | Permite responder que no hubo adjudicación en el estado documentado. |
| v2, `cm5v2q_c8df100909c8daf7fbc0` | `cm5v2_2155cb370088eaa59682`: no se seleccionó un proveedor en esa reunión | La negación está limitada a esa reunión. No se convierte en «nunca hubo ganador» ni en ausencia de una compra posterior. |
| v2, `cm5v2q_83eac9f1cc94898beab7` | `cm5v2_3ca079a84f414caa9ba8`: se solicitó información sobre una licencia comercial | La solicitud contextualiza el trámite, pero no acredita concesión. |
| v2, `cm5v2q_ee0d52649ceeffc138e1` | `cm5v2_da7230b8c12599e52561`: dos aves en un conteo preliminar | No respalda usar dos como cifra del informe final. |
| v2, `cm5v2q_1338c7f7bfaebae65327` | Ninguna memoria da la contraseña del catálogo digital | La abstención es defendible: no hay evidencia del dato solicitado. |

Los 75 positivos v2 y los 24 positivos v3 tienen una respuesta explícita y localizada. No se encontró una ambigüedad bloqueante comparable. Los positivos v3 de pintura descartada, prohibición de grabar y prohibición de control remoto muestran que una decisión negativa puede constituir un recuerdo útil. La revisión no demuestra que todos los usos reales tengan una sola respuesta admisible.

## Contrato propuesto para la siguiente evaluación

La anotación usa tres relaciones para los negativos: `refutes_premise` cuando existe una negación explícita del resultado o del estado presupuesto; `pending_or_partial` cuando hay evidencia cercana o no definitiva que no permite completar el dato final; `unrelated` cuando ningún recuerdo responde ese dato. La relación se aplica al contenido del recuerdo, no a una salida observada del sistema. En el caso v2 de proveedores, la refutación conserva expresamente su alcance a esa reunión.

Una evaluación futura debe anotar pares pregunta–memoria como apoyo, refutación, contexto parcial o irrelevancia y fijar antes de ejecutar si el contexto parcial se admite. Debe separar al menos:

1. Cobertura y precisión de recuperación útil, con apoyo y refutación medidos por separado.
2. Disponibilidad del dato: conocido, explícitamente ausente o pendiente, o desconocido en el corpus.
3. Fidelidad del texto emitido: conservación de negación, estado, temporalidad, condición y fuente.
4. Afirmaciones sin respaldo en la semilla o respuesta realmente producida, sin inferirlas de un ID discordante.
5. Abstención por falta de evidencia y pérdida de evidencia útil por abstención excesiva.

Recuperar un recuerdo parcial tampoco se considera automáticamente correcto: si luego se convierte su cifra provisional en final, sí existe una afirmación sin respaldo. Para establecerlo se necesita observar el texto producido. Esta auditoría ciega no lo ha leído y no atribuye ese fallo a ningún proveedor.

## Uso retrospectivo y límites para el cierre

Los porcentajes históricos y los fixtures originales se conservan bajo su contrato de igualdad exacta. Esta anotación explica por qué la expresión falso recuerdo no queda acreditada por una discrepancia de ID en todos los negativos. No borra errores históricos, no declara ganador a un proveedor y no acredita seguridad de una política nueva.

El cierre técnico de esta iteración puede incluir el hallazgo y el cambio de contrato propuesto. La validación del producto requiere medir fidelidad bajo un contrato fijado previamente y mantener visible la revisión humana o muestra real pendiente que exige el protocolo. Cerrar el informe no equivale a haber obtenido esa evidencia. Un banco pequeño, sintético, literal, con revisión 1 y sin condiciones estructuradas no estima rendimiento poblacional ni comprueba autorizaciones, revisiones sucesivas o integración real.

## Trazabilidad de archivos

SHA-256 de la anotación: `f2fdc549ada1debf449f63eb0e598d68174a8316edcac481fc8c7f79660b9ef2`.

| Banco | Archivo original | SHA-256 |
| --- | --- | --- |
| v2 | `development.jsonl` | `b755025aa45893438db9c6b4a833d9aee84f5d5e822a42c24121cd5aebf0355e` |
| v2 | `memories.json` | `3cf0394ff2013db264877a90fd483e77c01ae559b03a8270fd3e42376150f4e8` |
| v2 | `manifest.json` | `6d2ccbdf4a40ec1b05cbbaa9e0be406453bc1a3f599940efc3a1b9ae125e18b7` |
| v3 | `development.jsonl` | `a3965c56b3f15db657672653b9959569e15a481bd1be3c3b3498ead298f08ac4` |
| v3 | `memories.json` | `062270aa79a79c158157f5f864f06c381e7b9d719e3fe93cf01077c46db517a8` |
| v3 | `manifest.json` | `56ecb63f665b957cdffbeabca04a8dcef39e33ae80b53e553469be19523ecd5b` |
