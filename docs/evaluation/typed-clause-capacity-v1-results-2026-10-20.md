# Contraste de capacidad V1: mejor formato, calidad insuficiente

UTC 2026-10-20. Se implementó y ejecutó el contraste de extractor con el mismo contrato. **7/8 DTO pasan; solo 2/8 contenidos son fieles según ambos críticos. No se supera la puerta ni se amplía.** El objetivo continúa activo en el paso 3. [Protocolo](typed-clause-capacity-v1-protocol-2026-10-20.md), [verificación y recibos](../../outputs/iromem/evaluation/typed-clause-capacity-v1-2026-10-20/verification.json).

## Contraste realizado

La comprobación contra los ocho payloads archivados prueba que **solo cambia `model`**, de `openai/gpt-4.1-mini` a `openai/gpt-4.1`. Se mantienen textos posicionales, fuente/pregunta aisladas, ontología, instrucciones, esquema, temperatura, límite de salida, proveedor exclusivo, traductor y parser V5. No se añadieron pistas, ejemplos, aliases de gold ni reparación de respuestas. [Comparación de peticiones](../../outputs/iromem/evaluation/typed-clause-capacity-v1-2026-10-20/payload-contrast.json).

La documentación oficial consultada describe el seguimiento de instrucciones de GPT-4.1 y lo marca deprecated; se usa aquí para un diagnóstico comparable de familia, no como selección definitiva ni recomendación de disponibilidad futura. [OpenAI Docs](https://developers.openai.com/api/docs/models/gpt-4.1). OpenRouter observó el endpoint activo; los ocho recibos identifican OpenAI y el alias `openai/gpt-4.1`, sin verificar por separado una revisión fechada. La comparación contra mini es histórica, no A/B contemporánea ni causalidad demostrada.

| Indicador | Cláusulas mini V1 anterior | Contraste full V1 |
| --- | ---: | ---: |
| HTTP 200 terminados | 8/8 | 8/8 |
| DTO completos válidos | 1/8 | **7/8** |
| Fuentes válidas | 1/4 | 3/4 |
| Preguntas válidas | 0/4 | 4/4 |
| Raw fiel según ambos críticos | 0/8 | **2/8** |
| DTO válido y fiel según ambos | 0/8 | **2/8** |
| Coste por recibos USD | 0,0091212 | **0,055206** |

El coste observado es 6,0525× el del piloto mini, con igual entrada y caché reportadas pero más salida y otra tarifa. No se atribuye ahorro, eficiencia ni mejora poblacional; la validez estructural mejora en esta muestra y la suficiencia semántica sigue fallando.

## Críticas de todos los contenidos

Dos agentes nuevos sin historial recibieron exclusivamente textos, contrato, raw y normalizados disponibles, sin gold, roles esperados ni otros resultados. Son revisiones de la misma familia de agentes, no humanas ni independencia poblacional. Ambos revisan los ocho ítems: 16 juicios conservados.

El primer revisor informa tres fieles y cinco defectuosos; el adversarial, dos fieles, cuatro defectuosos y dos inciertos. Acuerdan cuatro defectos (`op_000`, `op_054`, `op_043`, `op_115`), dos fieles (`op_021`, `op_065`) y discrepan en dos (`op_001`, `op_088`). No se compensan defectos con aciertos. Los siete normalizados disponibles conservan su raw, incluidos sus problemas; uno está ausente. Que la traducción conserve el raw no lo convierte en fiel al texto. [Críticas y discrepancias](../../outputs/iromem/evaluation/typed-clause-capacity-v1-2026-10-20/blind-review-summary.json).

- El requisito mantiene negación y teléfono, pero el sujeto pierde el formulario del pedido; `kind=single` confunde cuantificación con clase de objeto.
- La red por T2 conserva palabras en el valor, pero la referencia usa `network_provenance` como tipo del sujeto y una descripción como nombre del propietario. Falla además literalidad (`la` frente a `La`). Relajar esa validación no resolvería la referencia incorrecta.
- La pregunta de autonomía conserva el evento de medición, pero trata «faros de Bruma» como nombre singular sin conservar la estructura del grupo/propietario.
- Atlas conserva cero pendientes, pero la fuente lo clasifica `text` mientras la pregunta usa `expediente`. Es un problema operativo de identidad/tipo, no un número incorrecto. No se igualaron las claves con aliases del banco.
- Se recupera la frecuencia mensual de Cima en `review_schedule`, frente al requisito sin frecuencia de mini. El adversarial deja incierta su referencia `text`; el otro la considera fiel. La discrepancia permanece.
- La pregunta de O7 conserva su dato pedido, pero clasifica Darsa como empresa sin que el texto declare ese tipo. Uno lo considera defecto y otro incertidumbre.

La causa observada ya no es principalmente la envoltura JSON: referencias libres mezclan metadatos (`single`, `text`), nombres, clases y relaciones de propiedad; también desaparecen portadores explícitos. Hace falta representar lo declarado y lo no declarado sobre el referente sin inventar tipos.

## Cobertura, recursos y límites

El control manual sigue en 100/100 comportamientos previstos y 40 seleccionados; no mide extracción automática ni autonomía. Los cuatro brazos mantienen denominador 100, aunque el piloto solo ejecuta ocho operaciones del banco:

| Brazo | Pares calculados | Pares degradados | Comportamientos previstos coincidentes |
| --- | ---: | ---: | ---: |
| Control manual | 100 | 0 | 100 |
| Fuente automática | 3 | 97 | 2 |
| Pregunta automática | 12 | 88 | 7 |
| Ambos automáticos | 2 | 98 | 1 |

En ambos automáticos se pierde una evidencia útil entre los dos pares calculados; las otras 98 degradaciones no son abstenciones fiables. Las mezclas con anotaciones manuales pueden fallar por namespaces incompatibles y no prueban por sí solas falsedad textual. Tampoco se presenta 1/100 como exactitud de 100 extracciones completas. No hay mejora de recuperación completa demostrada.

Ocho HTTP/2 200, cero reintentos, cero costes nuevos desconocidos. Estimación anunciada USD 0,1970153; cota conservadora por bytes/salida máxima USD 0,68039. **Coste real USD 0,055206**, incluidos los DTO/juicios fallidos. 41.887 tokens de entrada, 2117 de salida y 30.336 de entrada en caché según recibos. Son tokens del proveedor experimental, no uso de Codex.

| Peticiones | p50 ms | p95 ms |
| --- | ---: | ---: |
| Fuentes, 4 intentos | 2234,870 | 4122,444 |
| Preguntas, 4 intentos | 1530,102 | 2029,022 |

Runner: 6408,247 ms con cuatro concurrentes; test Cargo: 7,11 s. No son recall, hook ni respuesta final. No se ejecutó campaña emparejada contemporánea de recursos ni estabilidad; no pasan esas puertas.

Acumulado del objetivo: **5985 operaciones, 23 campañas emparejadas completas, USD 0,694444508 conocidos más 16 operaciones de coste desconocido**, sin gasto de Codex. Este diagnóstico añade ocho operaciones, no una campaña completa.

## Verificación y decisión siguiente

415 pruebas pasan con y sin HTTP/2; una prueba nueva de igualdad/rechazo de modelos se ejecuta en las dos inclusiones del contrato. Formato y `git diff --check` pasan. Crítico estático sin bloqueantes nuevos; payloads reales iguales salvo modelo. Los cuatro bancos manuales, siete informes anteriores de extracción y el nuevo se reproducen byte por byte. Snapshots 001–054 y hash de manifest V6 conservados; V6 sin inspección semántica ni llamadas. No cambian política, README ni gráficas de rendimiento vigentes. No se crean secretos, dependencias, SQL, puertos o servicios.

La siguiente hipótesis debe hacer explícito el tipo de objeto que el texto declara y su ausencia, separándolo de `scope` y de las categorías JSON. Antes de implementarla, se conserva una crítica del diseño de referencias apoyadas en menciones literales: deberá comprobar homónimos, propietarios, componentes, clase/grupo, tipo ausente, sinónimos y plurales. Una mención literal no demuestra por sí sola que se eligió el núcleo nominal correcto, y no se deben borrar tipos para obtener matches. No añadir aliases desde gold ni aceptar un tipo omitido como comodín. Esa revisión no equivale a código implementado o calidad probada.

La [crítica de diseño conservada](../../outputs/iromem/evaluation/typed-clause-capacity-v1-2026-10-20/reference-design-critic.json) añade tres condiciones previas: evidencia de que el sustantivo clasifica ese referente, separación entre identidad léxica y `value_type` ontológico para rutas intermedias, y tratamiento de tipo ausente sin unir clases anónimas ni presentar igualdad de nombres como identidad demostrada. También exige codificación reversible sin truncado y sin normalización de sinónimos/plurales obtenida del banco. Sigue siendo una propuesta, no una implementación nueva.

Permanecen pendientes extracción fiel completa, alineación automática fuente/pregunta, banco conocido y difícil, contraste contra baseline, recursos/estabilidad y cohorte nueva ciega. El goal sigue activo; no se rebaja la puerta de ocho ni se promueve el mejor subconjunto.
