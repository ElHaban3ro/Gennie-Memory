# Verificación de todos los positivos por rol: resultados de desarrollo

Ninguna de las dos variantes cumple todas las puertas. Se mantienen Decisions y `baseline` como configuración predeterminada. El objetivo continúa activo; V6 conserva cero llamadas al proveedor y no se utiliza para ajustar el mecanismo.

## Qué se implementó

`per_memory_role_verified_v1` pide a Jev 1.13 un tipo real para cada uno de los siete primeros recuerdos: dato/parte solicitada, corrección de premisa o ajeno. Decisions comprueba **cada positivo** con la consulta y un solo recuerdo. La comprobación de datos exige el mismo sujeto, objeto y contexto; admite partes, límites y estados desconocidos. La de correcciones conserva las dos vías tipadas anteriores: corrección ordinaria o contraste explícito de contexto.

El servicio y el auditor JSON independiente comprueban IDs, revisiones, tipos, todos los positivos verificados, composición y recibos de cada etapa. No hay aceptación automática por posición ni por haberse propuesto como dato. Una comprobación fallida degrada toda la operación y conserva el consumo conocido.

V2 conserva el mecanismo, compacta el texto de propuesta y comprobación de datos, y aclara cuándo un paso previsto aporta contexto parcial sobre terminación. Las comprobaciones de correcciones no cambian. Cambiaron varios textos; estos resultados no aíslan causalmente una sola cláusula.

Protocolos congelados antes de las llamadas: [V1](per-memory-role-verified-v1-protocol-2026-10-19.md) y [V2](per-memory-role-verified-v2-protocol-2026-10-19.md).

## Mediciones

Dos grupos conocidos por variante: 13 preguntas V5 de preservación y cinco regresiones V4. Son diagnósticos seleccionados, no una evaluación nueva ni una comparación emparejada completa. El coste relativo usa el subconjunto histórico de la base; exige después una comparación completa contemporánea si se supera el diagnóstico.

| Variante y grupo | Pérdidas útiles previas | IDs ajenos seleccionados | Coste USD | Coste/base histórica | HTTP | CLI p50/p95 |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| V1, 13 V5 | 1 | 0 | 0,001222752 | 1,673× | 29 | 903 / 1171 ms |
| V1, 5 V4 | 0 | 0 | 0,000485164 | 1,387× | 12 | 927 / 1118 ms |
| V2, 13 V5 | 0 | 0 | 0,001396120 | 1,911× | 38 | 903 / 1340 ms |
| V2, 5 V4 | 0 | 2 | 0,000798544 | 2,284× | 23 | 810 / 971 ms |

Latencias de estas ejecuciones CLI, incluyendo búsqueda, clasificación remota y composición; no son latencias de una integración Codex. Los grupos son pequeños y sus percentiles no prueban estabilidad.

V1 resuelve los cinco conjuntos esperados V4 y recupera los tres parciales antes perdidos. Rechaza efectivamente `cev4_065_03` —otra ruta— y `cev4_061_10` —un evento individual ofrecido como política general—. En V5 omite `cev5_069_12` desde la propuesta: ningún verificador puede recuperar un recuerdo que no recibe. Ese ID tiene una discrepancia semántica retrospectiva registrada; no se retira del gold ni de la puerta de preservación para convertir el resultado en pase.

V2 supera la puerta conservadora de calidad/preservación V5, pero no implica completitud: recupera 15 de 17 IDs útiles clasificados dentro del prefijo y todavía omite dos. V1 recupera 14 de 17. No se confunden estas cifras con todos los recuerdos permitidos fuera del prefijo ni con el porcentaje de preguntas contestadas íntegramente. V2 vuelve a aceptar los dos IDs ajenos V4 y excede el límite de coste en ambos grupos. Por ello tampoco se amplía.

## Por qué el texto más corto costó más

| V5, 13 preguntas | Entrada de propuesta | Coste de propuesta USD | Checks de datos | Checks de corrección | Coste de comprobaciones USD |
| --- | ---: | ---: | ---: | ---: | ---: |
| V1 | 16.806 tokens | 0,000705852 | 11 | 5 | 0,000516900 |
| V2 | 14.960 tokens | 0,000628320 | 16 | 9 | 0,000767800 |

La entrada de propuesta baja un 11 %, pero los positivos que necesitan comprobación aumentan de 16 a 25. El total sube un 14,2 %. Quitar texto no fue una optimización segura de este flujo. Todos los recibos nuevos están completos; ninguno se elige entre repeticiones y no se reintentan operaciones inciertas.

## Verificación y procedencia

131 pruebas de workspace pasan, junto con formato y compilación. Se cubren aislamiento del dato enviado, primera propuesta positiva verificada, rutas exclusivas por rol, rechazo de trazas incompletas, recibo global único de propuesta y agregación atómica de costes conocidos ante fallo. Esto prueba contratos de software; no garantiza la corrección semántica del modelo.

Los cuatro directorios `outputs/iromem/evaluation/per-memory-role-verified-v{1,2}-{pilot,v4-targeted}-2026-10-19/` conservan casos, preflight, protocolos vinculados, diario, auditoría ordinaria, informe específico y resumen CLI. Cada ejecución conserva una copia parcial de fuentes y las huellas de binarios; no se afirma que sea un archivo autónomo del workspace completo ni se archivan ejecutables. Las etiquetas originales no cambian.

Este avance añade **36 operaciones, 102 HTTP y USD 0,003902580**. Acumulado del objetivo: **3311 operaciones, 21 campañas emparejadas completas, USD 0,235984650 conocidos y tres operaciones históricas de importe desconocido**. El coste de programación y revisión con Codex no está incluido. Los pilotos nuevos no incrementan el número de campañas emparejadas.

## Siguiente investigación

La vía de comprobación de datos V1 aporta evidencia concreta de rechazo correcto en V4 y debe conservarse como referencia. La compactación V2 se descarta. Antes de otra campaña, investigar la representación de preguntas independientes que compartan consulta e instrucciones, con el recuerdo literal dentro de cada pregunta, evitando que otros recuerdos formen parte de su estado compartido. La [documentación de TypeSafe](https://docs.typesafe.ai/introduction) describe preguntas independientes sobre un estado común; el [contrato de OpenRouter](https://openrouter.ai/docs/api/api-reference/alphadecisions/submit-a-decisions-request) admite varias preguntas. Esto **no demuestra** aislamiento ni ahorro para el adaptador Decisions concreto: ambos requieren comprobaciones y recibos propios.

La hipótesis pendiente no está implementada: conservar el texto completo de V1, representar por separado todos los positivos, contabilizar una sola vez cualquier recibo global y contrastar distractores antes del diagnóstico fijo. La ausencia de ahorro o cualquier pérdida detiene la ampliación. Siguen pendientes las campañas completas conocidas, consistencia fija y evaluación nueva V6 revisada aparte. No se rebajan puertas ni se usa el mejor subconjunto de las variantes para declarar una política aprobada.
