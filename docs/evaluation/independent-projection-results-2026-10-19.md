# Proyecciones separadas y comparador por rol — 2026-10-19

Ninguna de las dos variantes supera calidad. Separar entradas recupera más evidencia que el extractor conjunto anterior, pero no impide aceptar hechos de otro referente o atributo. La candidata no se incorpora al producto, V6 permanece cerrado y el goal sigue activo.

## Implementación y alcance

Se implementó un contrato puro de evaluación con tres etapas: interpretar cada pregunta una sola vez sin recuerdos; extraer cada memoria/revisión una sola vez sin preguntas; y contrastar después originales y proyecciones. Los 128 pares conocidos contienen 20 preguntas y 105 memorias/revisiones distintas. La reutilización dentro de la ejecución hace que los candidatos de una pregunta compartan exactamente su proyección; esto garantiza estabilidad del artefacto, no que la interpretación sea correcta.

Las afirmaciones de memoria separan sujetos, predicados, objetos, valores, contexto, negadores y condiciones. Todos son conjuntos de IDs de palabras existentes de su fuente, restringidos por enums de esquema y revalidados. Un antecedente puede estar en otra cláusula de esa misma memoria; negación y condición pueden coexistir. No hay un enum que confunda polaridad con incertidumbre del valor. Estas garantías de fuente y forma tampoco prueban correferencia, completitud o implicación semántica.

Se usó GPT‑4.1 mini por OpenRouter, esquema estricto, temperatura cero, sin fallback y máximo 1400 tokens de salida. Se registraron alias, proveedor y recibos, sin afirmar una revisión interna inmutable. No se pasaron etiquetas ni resultados al proveedor. No hay política runtime, DB, caché persistida, secreto, dependencia o puerto nuevos: bootstrap compila estos runners únicamente bajo `cfg(test)` y el evaluador posee los contratos y auditores.

## Primera variante: enlaces a slots y afirmaciones

Los 125 payloads de proyección se congelaron antes de red. Se completaron 20 preguntas y 105 memorias; tres respuestas de memoria fallaron por afirmaciones duplicadas. Sus cuatro pares quedaron no disponibles, sin tratarlos como abstenciones fiables. Después se congelaron los 124 payloads de comparación a partir de los frames reales, antes de las llamadas de matching.

El comparador elegía una conexión entre un slot pedido o premisa y una afirmación de memoria, además de context/none. Elegir una conexión existente no demostró que fuera semánticamente válida: hubo 14 selecciones no permitidas por el gold.

## Segunda variante: cuatro roles semánticos

El [protocolo v2](independent-role-matcher-v2-protocol-2026-10-19.md) sustituyó la tabla de conexiones propuestas por cuatro categorías: contribución solicitada, refutación, contexto relacionado e irrelevancia. Es una prueba de interfaz/instrucción; no demuestra por sí sola que el número de opciones causara los errores.

Se reutilizaron las 125 respuestas originales de preparación. Se validó cada afirmación antes de normalizar **solo duplicados con exactamente los mismos conjuntos de IDs por campo**. Se conservaron la respuesta original, primera aparición y mapa de posiciones. Siete repeticiones exactas en tres recuerdos se redujeron sin eliminar afirmaciones distintas: una, cuatro y dos respectivamente. No se reparó el informe v1 ni se volvió a llamar al extractor.

El preparador verificó que los 124 pares compartidos conservaran exactamente originales y proyecciones. Los cuatro restantes se habilitaron gracias a esa normalización. Se congelaron 128 comparaciones nuevas y se conservaron todas sus respuestas, sin elegir la mejor salida por par ni modificar etiquetas. No hubo errores de transporte, matching o recibos, pero persistieron 11 selecciones no permitidas.

## Métricas contra el gold original

| Variante, 126 candidatos | Útiles aceptados | No útiles rechazados | Aceptaciones no permitidas | Útiles omitidos con comparación válida | Pares no disponibles |
| --- | ---: | ---: | ---: | ---: | ---: |
| Enlaces v1 | 19 | 87 | 14 | 2 | 4 |
| Roles v2 | 19 | 93 | 11 | 3 | 0 |

Los dos controles de autor pasan en ambas variantes y se informan aparte. El prefijo contiene 22 recuerdos anotados útiles; v2 conserva 19. No se compensan omisiones con ganancias ni se considera que una proyección de forma válida contenga necesariamente todas las afirmaciones correctas.

Algunos errores de v2 son inequívocos: acepta la hora del equipo de Bruno para Dalia o una contraseña de Salitre para Ribera. También trata «presentar carné» como exclusión de avalista, sin exclusividad. Las discrepancias de contexto previamente revisadas se siguen contando según el gold original; no explican las selecciones de otro referente. Una selección no permitida no significa que el texto guardado sea falso ni demuestra una respuesta final generada falsa: aquí se evalúa selección de evidencia, sin integración de respuesta final.

## Diagnóstico retrospectivo de identidad literal

Se implementó una recomposición local **posterior**, sin llamadas nuevas: tomar los tokens capitalizados que el proyector marcó como identidad fija, excluir el inicial de frase y exigir su presencia literal normalizada en el recuerdo. El auditor revalida antes todos los artefactos del comparador v2. Se examinan los 128 pares, no solo sus errores.

Sobre las salidas ya observadas, esa regla eliminaría ocho de las 11 aceptaciones no permitidas, sin retirar ninguno de los 19 recuerdos útiles aceptados. Quedarían **19 útiles aceptados, 101 no útiles rechazados, tres aceptaciones no permitidas y tres omisiones**, con controles 2/2. Son cifras de recomposición exploratoria, **no una mejora validada con una nueva ejecución**, y no sustituyen la tabla anterior ni habilitan adopción.

Los tres casos que seguirían fallando muestran límites distintos:

- Carné para Olmo no implica que no se necesite avalista.
- Un depósito de lluvia que alimenta Laurel no identifica una acometida de la red ni excluye explícitamente otra alimentación.
- El precio del contador de Romero no refuta una pregunta sobre Laurel. En ese caso el proyector puso Laurel en premisas y dejó únicamente «contador» en identidad: la literalidad no corrige un rol mal extraído.

La regla tampoco resuelve alias, pronombres, nombres omitidos o calificadores mal proyectados. Las pérdidas potenciales por esos motivos deben medirse en un ensayo nuevo. Que un nombre aparezca no prueba identidad, y que falte no prueba que el hecho sea falso.

## Coste y tiempos

| Concepto | Operaciones HTTP nuevas | Coste USD |
| --- | ---: | ---: |
| Preparación v1: 20 preguntas + 105 memorias | 125 | 0,059891200 |
| Comparador por enlaces v1 | 124 | 0,034739200 |
| Comparador por roles v2 | 128 | 0,031594000 |
| Diagnóstico literal retrospectivo | 0 | 0 |
| **Gasto nuevo de esta fase** | **377** | **0,126224400** |

La candidata v1 completa cuesta USD 0,094630400. El coste lógico completo de v2 es **USD 0,091485200**, incluyendo los USD 0,059891200 de preparación reutilizada; no se presenta solo el precio de matching como coste del mecanismo. En la contabilidad global esa preparación se cobra una sola vez. No se presume ahorro amortizado futuro.

Medianas/p95 medidos por etapa: pregunta **1088,55 / 1581,11 ms**; memoria **1304,86 / 1904,26 ms**; matching v1 **837,53 / 1228,68 ms**; matching v2 **859,29 / 1532,97 ms**. Las ejecuciones completas del harness duraron 90,38 s y 43,31 s, con hasta cuatro llamadas simultáneas. Estos tiempos no son latencia CLI de `recall`, integración Codex ni un pase de recursos; no se suman medianas de etapas como si fueran tiempo real de un usuario.

Todos los 377 cargos nuevos tienen recibo. El acumulado del goal pasa a **4586 operaciones**, **21 campañas emparejadas completas** y **USD 0,499593258 conocidos**, más tres operaciones históricas de coste desconocido. Se excluyen programación y revisión con Codex. Las variantes descartadas siguen contando; esta fase no añade casos nuevos ni una campaña emparejada de adopción.

## Decisión y próximos ensayos

La separación de entradas y la normalización exacta mejoran propiedades del protocolo, pero el comparador todavía atribuye mal hechos y propiedades. No se justifica incorporar esta preparación costosa ni seguir añadiendo campos al mismo grafo por su apariencia estructurada.

La hipótesis literal identifica un control pequeño que merece probarse prospectivamente, con límites explícitos de alias y roles. Conviene contrastarlo con la ruta de una sola petición mejor medida, `per_memory_scoped_v1`, en vez de asumir que más etapas mejoran calidad. Aquel perfil tuvo un pase histórico de V5, un error posterior de estabilidad y coste por encima del límite; no es una candidata aprobada. Cualquier variante deberá conservar evidencia de la base y de ese perfil, comprobar cobertura antes de recortar candidatos y superar calidad, estabilidad y coste/p50/p95 ≤ 1,5× con ejecución real. Ningún resultado retrospectivo omite esas puertas ni abre V6.

El objetivo de cinco pasos no está cerrado: contrato y bancos siguen trazados, el mecanismo no supera calidad, y los casos nuevos con revisión ciega y la adopción continúan pendientes. La configuración predeterminada, etiquetas y fuentes congeladas se conservan.

Verificación: **188 pruebas del workspace**, formato, compilación del workspace y diff. Se canceló una compilación auxiliar propia que esperaba el bloqueo de Cargo y luego se completó una compilación única; no se detuvieron procesos del usuario. No se iniciaron servicios, reiniciaron dashboards ni crearon bases de campaña. El snapshot028 conserva contabilidad, resultados y archivo parcial de fuentes, sin archivar binarios.

Artefactos: [protocolo v1](independent-projection-v1-protocol-2026-10-19.md), [informe v1](../../outputs/iromem/evaluation/independent-projection-v1-2026-10-19/report.json), [informe v2](../../outputs/iromem/evaluation/independent-role-matcher-v2-2026-10-19/report.json), [normalización de proyecciones](../../outputs/iromem/evaluation/independent-role-matcher-v2-2026-10-19/projections.json), [diagnóstico literal posterior](../../outputs/iromem/evaluation/independent-role-matcher-v2-2026-10-19/literal-identity-posthoc.json) y [perfil histórico Scoped](per-memory-scoped-results-2026-10-19.md).
