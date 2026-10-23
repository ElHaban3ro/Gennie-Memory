# Dimensión solicitada en el prefijo completo — 2026-10-19

La hipótesis no supera el filtro de calidad. Corregir el caso de formato TIFF no basta: aparecen cuatro aceptaciones incorrectas al revisar todos los candidatos. No se incorpora al producto ni se ejecuta la variante agrupada, condicionada a superar esta prueba. La configuración predeterminada continúa igual y el objetivo de cinco pasos sigue activo.

## Alcance y método

El [protocolo previo](full-prefix-dimension-probe-protocol-2026-10-19.md) fija 126 comprobaciones: los siete candidatos de cada una de las 18 preguntas conocidas del piloto anterior, incluidos los que el proponente descartaba. Se añaden dos controles escritos por el autor: preguntar expresamente el formato o la resolución del mismo recuerdo TIFF. Son desarrollo, no un banco nuevo independiente.

Se mantienen las cuatro preguntas del comprobador anterior y se añade una quinta sobre la dimensión solicitada. La composición es `(dato && dimensión) || (corrección && premisa incompatible) || contraste explícito`. Cada candidato recibe una petición aislada a `perplexity/pplx-decider-v1.2`, revisión servida `perplexity/pplx-decider-v1.2-20261016`. Las peticiones y las fuentes se congelaron antes de la red; se conservaron las 128 respuestas, sin reintentos ni selección de la mejor repetición. El auditor Rust reconstruye las peticiones y verifica composición, versión, costes y procedencia.

## Resultado contra las etiquetas originales

| Medida | Resultado |
| --- | ---: |
| Candidatos comprobados | 126 |
| Útiles aceptados | 21 |
| No útiles rechazados | 100 |
| No útiles aceptados | 4 |
| Útiles omitidos | 1 |
| Controles formato/resolución | 2/2 |
| Errores técnicos / costes desconocidos nuevos | 0 / 0 |
| Supera el filtro semántico | No |

Los controles no se mezclan con los 126 candidatos. Dos recuerdos permitidos por las etiquetas de las preguntas están fuera del prefijo; el informe los registra aparte. Revisar el prefijo completo mejora la cobertura del diagnóstico frente a comprobar solo positivos congelados, pero no prueba recuperación fuera de ese prefijo ni permite comparar directamente estas cifras con el piloto anterior, cuyo universo de comprobación era distinto.

El recuerdo `cev5_068_01` deja de aceptarse como evidencia de terminación y sí se acepta cuando la pregunta pide su formato o resolución. Los errores restantes son:

| Pregunta / recuerdo | Error observado |
| --- | --- |
| `cev5q_066_07` / `cev5_066_18` | Acepta el nombre del buzón como dato de su contraseña. |
| `cev5q_066_07` / `cev5_066_22` | Trata la funda de un telescopio homónimo como corrección sobre la contraseña de un buzón. |
| `cev5q_069_07` / `cev5_069_20` | Trata el plastificado de mapas como refutación de una pregunta sobre cuota de inscripción. |
| `cev5q_068_03` / `cev5_068_05` | Confunde otro documento y otra autora con una refutación sobre el libreto solicitado. |
| `cev5q_070_05` / `cev5_070_11` | Rechaza una muestra preparada y sellada, permitida por el gold original de una pregunta sobre grados de frote. Es una discrepancia de criterio documentada; no se cambia el gold para mejorar el resultado. |

Aplicar retrospectivamente la dimensión también a la vía de corrección elimina tres aceptaciones incorrectas, pero pierde dos refutaciones válidas y mantiene el error nombre/contraseña. Esa recomposición es un diagnóstico posterior sobre las mismas respuestas, no una mejora evaluada con llamadas nuevas; queda descartada.

## Revisión ciega de fuentes

Un subagente crítico recibió exclusivamente ocho conversaciones sintéticas y sus 64 preguntas, sin etiquetas ni resultados. Explicitó identidad, dimensión solicitada, premisas, relaciones y límites de respuesta. La validación posterior comprobó las 64 preguntas, pertenencia de IDs, separación de contexto y evidencia útil, y literalidad de 135 citas. El paquete sin etiquetas se conserva; la restricción de acceso del revisor es declarada, no aislamiento del sistema de archivos.

La revisión discrepa del conjunto útil original en siete preguntas, todas por contexto que el revisor considera insuficiente para el dato solicitado. Es revisión del mismo sistema/familia, no humana ni una garantía de verdad. No coincide plenamente con todas las revisiones anteriores y no se utiliza para sustituir las etiquetas ni recalcular el rendimiento histórico. En el caso de la muestra de frote coincide con una auditoría y adjudicación anteriores a esta prueba: preparación y sellado no aportan un grado ni prueban la ejecución. Los cuatro errores de aceptación siguen siendo problemas aunque se resuelva esa discrepancia.

## Coste y latencia

Las 128 operaciones realizaron 128 peticiones HTTP y costaron **USD 0,001982040**, con todos los recibos presentes. La mediana de una comprobación aislada fue **418,88 ms** y el p95 **1948,69 ms**. Son tiempos de llamadas de investigación; no son latencia de `recall`, ni demuestran el límite de coste/latencia del mecanismo completo. La revisión y programación con Codex no están incluidas en estos importes.

El acumulado contable pasa a **3677 operaciones**, **21 campañas emparejadas completas** y **USD 0,246299554 conocidos**, más **tres operaciones históricas con coste desconocido**. Se preservan resultados negativos y todos los gastos de desarrollo.

## Estado y continuación

Contrato y revisión de datos avanzan; el mecanismo todavía no cumple su objetivo. La próxima hipótesis debe vincular el objeto y la propiedad afirmada por la fuente con la propiedad o premisa concreta de la pregunta, conservando negaciones y evidencia parcial. Añadir otra decisión genérica de relevancia o un veto global no ha demostrado resolverlo. Cualquier representación automática propuesta necesitará comprobar extracción, selección, coste y latencia; las anotaciones manuales del revisor no se presentarán como un mecanismo automático.

No se abren los contenidos semánticos de V6 ni se envían a proveedores; sus hashes siguen congelados. No se crean bases temporales ni servicios en esta fase. La implementación optativa anterior se conserva como experimento fallido; no hay promoción, afirmación de cierre ni despliegue.

Artefactos: [auditoría Rust](../../outputs/iromem/evaluation/full-prefix-dimension-probe-2026-10-19/report.json), [recomposición posterior](../../outputs/iromem/evaluation/full-prefix-dimension-probe-2026-10-19/posthoc-global-dimension-recomposition.json), [revisión de fuentes](../../outputs/iromem/evaluation/prefix-intent-source-review-2026-10-19/review.json), [validación de citas](../../outputs/iromem/evaluation/prefix-intent-source-review-2026-10-19/validation.json) y [comparación sin modificar gold](../../outputs/iromem/evaluation/prefix-intent-source-review-2026-10-19/comparison-original-preserved.json).
