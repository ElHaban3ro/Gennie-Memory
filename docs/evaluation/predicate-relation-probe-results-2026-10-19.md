# Diagnóstico de relación objeto/propiedad — 2026-10-19

Ninguno de los tres modelos supera la puerta semántica. La variante reduce algunas selecciones incorrectas del diagnóstico anterior, pero pierde evidencia útil y sigue confundiendo predicados o referentes. No se incorpora al producto, no se agrupa y no se abre V6. El objetivo de cinco pasos permanece activo.

## Prueba congelada

El [protocolo previo](predicate-relation-probe-protocol-2026-10-19.md) sustituye cinco decisiones booleanas por una elección excluyente: A, dato/parte/estado solicitado; B, refutación explícita; C, contexto relacionado; none, ausencia de relación establecida. A/B se aceptan, C/none no se cuentan como evidencia solicitada. El texto pide distinguir objeto, propiedad afirmada y propiedad o premisa preguntada, pero no genera un testigo de extracción ni prueba que el modelo aplique esa distinción.

Cada modelo recibe exactamente los mismos 128 pares: 126 candidatos conocidos y dos controles de autor, con consulta/resumen literales. No se filtran positivos ni se pasan golds, IDs o anotaciones del revisor al proveedor. Las 384 solicitudes, reglas, modelos/revisiones y fuentes quedaron ligadas antes de red. La ejecución completa duró 216,85 segundos, sin reintentos ni errores técnicos. El auditor Rust reprodujo además la auditoría del diagnóstico fuente contra los pilotos CLI y etiquetas originales.

## Calidad y recursos

| Variante | Útiles aceptados | No útiles rechazados | Aceptaciones incorrectas | Omisiones contra gold | Controles | Coste USD | p50 / p95 por comprobación |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Cinco decisiones anteriores, Perplexity | 21 | 100 | 4 | 1 | 2/2 | 0,001982040 | 418,88 / 1948,69 ms |
| Relación v1, Perplexity | 20 | 101 | 3 | 2 | 2/2 | 0,001229400 | 404,36 / 594,00 ms |
| Relación v1, Respan | 19 | 101 | 3 | 3 | 2/2 | 0,000631170 | 356,03 / 2274,57 ms |
| Relación v1, Liquid | 18 | 102 | 2 | 4 | 2/2 | 0,002395134 | 422,84 / 1174,83 ms |

Cada fila usa 126 candidatos y los controles se informan aparte. La primera fila es la corrida histórica del mismo conjunto, no una ejecución intercalada en este diagnóstico. Los tiempos corresponden a peticiones aisladas; no son latencia CLI de `recall`, ni se usan para aprobar su puerta de recursos. Cambian la instrucción, categorías y composición; en las otras filas también cambia el modelo. No se atribuye una causa única a las diferencias ni se combinan las mejores respuestas por recuerdo.

Las revisiones servidas coincidieron con las fijadas: Perplexity `pplx-decider-v1.2-20261016`, Respan `span-02-20261015` y Liquid `d1-ultra-20261015`. Las **384 operaciones / 384 HTTP costaron USD 0,004255704**, con todos los recibos completos y cero costes desconocidos nuevos. Programación y revisión con Codex quedan fuera. El acumulado es **4061 operaciones / 21 campañas emparejadas / USD 0,250555258 conocidos**, más tres operaciones históricas de coste desconocido.

## Qué falla

Los tres modelos tratan «Teo diseñó las siluetas de Neblina, no escribió ese libreto» como refutación de una pregunta sobre el libreto **Luciérnaga**. La negación se transfiere entre documentos. Perplexity también acepta otro documento de otra autora como corrección y vuelve a tratar «presenta carné» como exclusión de un avalista, aunque no haya exclusividad. Respan confunde ausencia de solista con refutación de una pregunta sobre el total de músicos. Respan y Liquid aceptan un tramo de otro recorrido como parte de la distancia solicitada.

Las pérdidas tampoco se explican únicamente por discrepancias del gold. Respan y Liquid relegan a C el ensayo de seis juegos de faros que funcionaron una hora, parte útil de una pregunta sobre autonomía de todas las parejas. Liquid pierde además la condición explícita de publicar solo cuando estén digitalizadas todas las páginas. Ser conservador sobre el contexto no justifica eliminar esos datos y condiciones.

Los tres rechazan como C la recepción de un mapa por la entidad que decide un permiso y la muestra preparada para frote húmedo, permitidas por el gold original. Son discrepancias de contexto ya documentadas en revisión separada. Se preservan como omisiones en esta tabla, sin cambiar etiquetas ni presentar una diferencia de criterio como mejora del modelo. Aun resolviendo esas discrepancias, permanecen aceptaciones incorrectas y pérdidas de evidencia inequívocamente útil.

Los cuatro errores del diagnóstico anterior no son una lista suficiente para aprobar: aparecen nuevos errores en candidatos que antes se rechazaban. De nuevo, la prueba debe cubrir todo el prefijo y conservar cada pérdida, no solo los casos que motivaron la modificación.

## Decisión y continuación

La opción de menor coste sigue fallando calidad. Las categorías excluyentes ayudan a observar el contexto, pero no establecen a qué objeto se aplica un predicado o una negación. Repetir una decisión genérica no aporta esa prueba. La siguiente hipótesis debe producir y comprobar una representación explícita de la afirmación y su destinatario, con citas fuente, sin confundir extracción literal con correferencia o implicación demostradas. El experimento `witness_v1` anterior ya mostró que coincidir en nombres y citas no basta; una nueva representación necesitará distinguir también propiedad, polaridad y alcance de la negación.

Esa extracción automática añadiría coste y latencia: no se incorporará por su apariencia estructurada. Deberá conservar datos parciales y condiciones, resolver estos contraejemplos y superar la comparación de flujo real antes de adopción. Las anotaciones manuales o de un crítico siguen siendo diagnóstico de desarrollo, no una realización automática. No hay cambio de configuración predeterminada, nuevas bases o servicios, ni afirmación de cierre de los cinco pasos. V6 permanece congelado y sin llamadas de proveedor.

Verificación local: **150 pruebas del workspace**, formato, compilación del evaluador y comprobación de diff. Los artefactos incluyen [informe auditado](../../outputs/iromem/evaluation/predicate-relation-probe-2026-10-19/report.json), [preflight](../../outputs/iromem/evaluation/predicate-relation-probe-2026-10-19/preflight.json), solicitudes y diario completos; snapshot026 conserva la contabilidad y el archivo parcial de fuentes. No hay commit, push ni despliegue.
