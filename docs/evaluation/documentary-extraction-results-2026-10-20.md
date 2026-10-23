# Extracción documental automática: resultado de desarrollo

La extracción automática implementada **no supera la puerta de calidad**. El 100/100 anterior sigue siendo un resultado sobre representaciones manuales revisadas. Al intentar producir esas representaciones desde texto aparecen fallos de formato y de significado. No hay mejora automática demostrada ni cambio de configuración predeterminada. El objetivo de cinco pasos sigue activo, en el paso 3.

## Qué se implementó y probó

Dos etapas independientes: cada fuente recibe únicamente su texto y una ontología conocida del dominio; cada pregunta recibe únicamente su texto y esa ontología. No se envían anotaciones manuales, etiquetas, resultados, candidatos ni catálogo de entidades. Se prepararon 83 fuentes y 33 preguntas únicas, con cuatro ablaciones locales sobre los 100 pares conocidos. La ontología compartida es configuración declarada: no se prueba descubrimiento general de esquemas.

El validador comprueba forma, contrato de dominio y citas literales; no rellena campos ausentes ni corrige la salida con gold. Las citas prueban procedencia textual, no fidelidad semántica. Cada error conserva el denominador y produce degradación explícita. Preparación, fuentes ejecutadas, payloads, diario sincronizado, cuerpos de respuesta y recibos quedan archivados. No se usó base de datos ni servicio local.

Los [protocolos V1](documentary-extraction-v1-protocol-2026-10-20.md) y [JSON V2](documentary-extraction-json-v2-protocol-2026-10-20.md) se congelaron antes de sus respectivas llamadas. Modelo `google/gemini-2.5-flash-lite`, proveedor fijo Google AI Studio estándar, máximo cuatro peticiones concurrentes, sin fallback ni reintentos.

| Intento | Peticiones terminadas / previstas | Resultado técnico | DTO válidos completos: fuentes / preguntas | Duración del runner |
| --- | ---: | --- | ---: | ---: |
| V1, JSON Schema nativo | 4/116 | Cuatro HTTP 400: el proveedor rechaza la complejidad del esquema | 0 / 0 | 2,034 s |
| V2, mismo esquema dentro del mensaje y JSON object | 56/116 | 53 HTTP 200 y tres HTTP 429; se detienen batches nuevos | 0 / 3 | 26,121 s |

V2 cambia la entrega del esquema, no el significado, los textos ni los criterios. Sus 60 peticiones no enviadas y los 112 casos no enviados de V1 no son errores semánticos observados. Tampoco son aciertos. El harness de los runners termina `ok` porque conserva y registra los fallos; eso no significa que el experimento pase.

## Calidad y revisión ciega

Tres críticos nuevos recibieron únicamente instrucciones/esquema, texto y salida automática, sin frames manuales, etiquetas ni resultados de las ablaciones. Se contabilizaron los 116 elementos: **53 contenidos disponibles revisados y 63 sin salida** —60 no enviados y tres HTTP 429—. Es revisión del mismo sistema de modelos, no revisión humana ni validación externa.

| Tipo | Fiel según crítico | Defectuoso | Incierto | Sin salida |
| --- | ---: | ---: | ---: | ---: |
| 83 fuentes previstas | 8 | 18 | 1 | 56 |
| 33 preguntas previstas | 0 | 26 | 0 | 7 |

La cobertura semántica completa prevista no se alcanzó; no se certifica fidelidad. Los ocho juicios de contenido fiel no convierten DTO inválidos en válidos. Las tres preguntas con DTO válido también fueron juzgadas defectuosas. Se preservan los juicios y sus límites, sin reparar salidas ni cambiar el banco.

Errores representativos observados:

- «El formulario […] no exige teléfono» se convierte en un teléfono desconocido y negado: requisito y disponibilidad no son la misma propiedad.
- Se pierden propietarios, condiciones y componentes; el objetivo de una cámara acaba sustituido por la cámara.
- Se convierten asignaciones de programas en métodos de medición y períodos observados en cantidades totales.
- Una pregunta de procedencia del agua pierde el huerto, la premisa de red o la distinción entre alimentación inmediata y origen aguas arriba.
- Se usa capacidad donde la pregunta pide uso real y se crean premisas a partir de incógnitas.

Las cuatro ablaciones conservan los 100 pares como denominador:

| Fuente / pregunta | Coincidencias completas de límites / 100 | Pares degradados | Selecciones ajenas computadas | Útiles no seleccionados computados | Útiles sin cobertura por degradación |
| --- | ---: | ---: | ---: | ---: | ---: |
| Manual / manual, control | 100 | 0 | 0 | 0 | 0 |
| Automática / manual | 0 | 100 | 0 | 0 | 40 |
| Manual / automática | 9 | 88 | 0 | 3 | 37 |
| Automática / automática | 0 | 100 | 0 | 0 | 40 |

En manual/automática solo se computan 12 pares: nueve coinciden y tres fallan; hay tres errores de rol, dos de cuenta de hechos y uno de límites, categorías que pueden solaparse. No se presenta 9/100 como precisión de un buscador operativo, ni cero selecciones ajenas como un pase: casi todo está degradado. Diferencias de identificadores internos por sí solas tampoco prueban falsedad semántica. El control manual reproduce exactamente el resultado congelado.

## Coste y latencia

V1 conserva cuatro importes desconocidos. V2 informa **USD 0,020467710**: USD 0,010807010 para fuentes y USD 0,009660700 para preguntas, más tres importes desconocidos de respuestas HTTP 429. Sumando ambos intentos: **60 operaciones / 60 HTTP, USD 0,020467710 conocidos y siete importes sin confirmar**. No se infiere que los errores fueran gratuitos. Los 53 recibos de éxito informan 176.681 tokens de entrada y 18.463 de salida; son tokens de esta prueba del proveedor, no tokens enviados a Codex.

V2: fuentes p50/p95 **1473,932 / 2086,931 ms**; preguntas **1489,368 / 2096,236 ms**. Incluyen peticiones terminadas fallidas. Son tiempos por petición y duración del runner, no latencia de `recall`, de un hook ni del recorrido hasta una respuesta final. La posible reutilización de extracción al indexar no es ahorro de runtime medido.

Acumulado del objetivo: **5941 operaciones / 23 campañas emparejadas completas; USD 0,602563308 conocidos más 12 importes desconocidos**, sin Codex. Estos diagnósticos no añaden campañas emparejadas completas.

## Verificación y siguiente puerta

Pasan **370 pruebas** tanto con configuración predeterminada como con HTTP/2, sin avisos; formato comprobado. Los informes de V1, V4, V5, known100 y ambos intentos de extracción se reproducen byte a byte. Se verificaron hashes del banco, código ejecutado, recibos y contabilidad anterior. V6 conserva su manifiesto y sigue sin abrirse semánticamente ni recibir llamadas. Baseline y dashboard no cambian; no hay promoción.

El siguiente ensayo debe ser un piloto pequeño de capacidad con el mismo esquema nativo y otra ruta/modelo compatible, antes de gastar en los 116 textos. Consultar y archivar catálogo/precios actuales, fijar roster sin etiquetas y conservar cada intento. Probar por separado cumplimiento del contrato y fidelidad de sujeto, negación, ámbito y premisa; no relajar el validador para aceptar las salidas defectuosas de esta tanda. Si no supera ambos, no ampliar. Si los supera, faltarán todavía el banco completo, los 126 casos difíciles, contraste contra baseline, recursos y estabilidad antes de V6.

Artefactos: [V1](../../outputs/iromem/evaluation/documentary-extraction-v1-2026-10-20/report.json), [V2 y cuatro ablaciones](../../outputs/iromem/evaluation/documentary-extraction-json-v2-2026-10-20/report.json), [crítica ciega](../../outputs/iromem/evaluation/documentary-extraction-json-v2-2026-10-20/blind-review-summary.json), [verificación](../../outputs/iromem/evaluation/documentary-extraction-json-v2-2026-10-20/verification.json), [contabilidad](../../outputs/iromem/evaluation/quality-goal-2026-10-19/index.json).
