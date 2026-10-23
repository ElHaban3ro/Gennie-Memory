# Críticos condicionales: mejora de calidad conocida, latencia pendiente

Las variantes con contexto completo V2/V3 conservan evidencia útil histórica y contemporánea y añaden seis IDs útiles frente a Decisions/base en las18preguntas conocidas. Tampoco el contrasteHTTP/2V4pasa todas las puertas: p95 excede1,5×base. No se adopta ninguna, no se abreV6 y el objetivo de cinco pasos sigue activo. Esto mide clasificación sobre candidatos congelados; no es recall integrado ni verdad de una respuesta final.

## Comparación emparejada

Cada variante incluye18pares alternados contra Decisions/base contemporáneo. Baseline recibe todos los candidatos recuperados, hasta20; propuesta recibe los primeros siete. Las listas, fuente/gold y modelos/revisiones están ligados por hashes. Las tres bases seleccionaron12de19IDs permitidos,12/18sets completos y15/18preguntas con evidencia permitida o abstención correcta.

| Métrica | V1: contexto reducido, secuencial | V2: contexto siete, paralelo | V3: contexto siete, preguntas dirigidas |
| --- | ---: | ---: | ---: |
| IDs permitidos recuperados /19 |17|18|18|
| Sets completos /18 |16|17|17|
| Preguntas con evidencia permitida o abstención correcta /18 |17|18|18|
| IDs no permitidos |0|0|0|
| Pérdidas útiles frente a base actual |0|0|0|
| Pérdidas útiles históricas |1|0|0|
| IDs útiles adicionales frente a base actual |5|6|6|
| Regresiones iniciales resueltas /5 |5|5|5|
| Coste candidata USD |0,000814880|0,001201744|0,000925144|
| Coste base USD |0,001119900|0,001119900|0,001119900|
| Razón coste candidata/base |0,728|1,073|0,826|
| p50 candidata/base ms |665,35/526,40|508,42/376,94|520,26/390,05|
| p95 candidata/base ms |3061,63/847,36|1781,54/893,14|1888,38/881,08|
| Razón p95 |3,613|1,995|2,143|
| Puerta de calidad |Falla|Pasa|Pasa|
| Puerta de recursos |Falla|Falla|Falla|
| HTTP reales |45|46|46|
| Coste total de comparación USD |0,001934780|0,002321644|0,002045044|

«18/18 preguntas con alguna evidencia permitida o abstención correcta» no es completitud100%: V2/V3 seleccionan18de19IDs y17de18sets. La omisión conservada es `cev5_070_11`, muestra de Índigo preparada para frote húmedo, segundo candidato recuperado: no es pérdida por recortar a siete. Se conserva gold y se informa el límite, sin eliminar esa etiqueta para mejorar porcentaje. Los tres casos cuyo gold está vacío se abstuvieron correctamente.

## Qué cambió y qué demuestra

V1implementa propuesta Mercury/RoleV5, conserva A con testigo literal y revisa B por lote reducido Jev, seguido por Microsoft sobre supervivientes. Ambos reciben solo las memorias propuestas, sin juicios anteriores. Falla preservación: ante «¿Qué integración usa M4 en Cendal para el perfil Silente?», Jev rechaza la memoria de M4/Umbral que corrige la premisa; el modelo de propuesta la había conservado. La validación retrospectiva con contexto completo no había previsto ese rechazo.

V2conserva los siete recuerdos como contexto de ambos críticos y los ejecuta en paralelo cuando hay B. Selecciona A(Mercury) más B(Mercury) aceptados como útiles por ambos; no acepta por iniciativa de los críticos otros IDs. Recupera la corrección perdida. Se cambian contexto y transporte, por tanto no se atribuye una causalidad aislada a uno solo. El acuerdo de utilidad no demuestra que Bsea la categoría correcta ni independencia estadística.

V3conserva todo ese contexto y retira únicamente las preguntas de relación de recuerdos no propuestosB. No envía probabilidades/juicios previos ni gold. Los índices originales se mantienen en el payload; el auditor normaliza solo respuestas existentes y conserva mapa aID/revisión. Conserva calidad observada y reduce coste frente a V2; no se demuestra reducción de latencia: su p95 empeora en esta muestra.

La llamada más lenta de V1fue Microsoft2028,55ms; enV2yV3 vuelve a ser la primera de Microsoft,1154,10y1303,98ms. Es evidencia compatible con apertura de otra conexión, sin separar red/TLS/servidor. El antecedente de consenso ya mostró picos posteriores: no asumir que el primer arranque explica todo. No se quitan llamadas lentas ni se recombinan mejores respuestas. Con18observaciones, p95nearest-rank es el máximo; el límite del piloto sigue aplicándose.

## Implementación y verificación

Los tres runners permanecen bajo `cfg(test)`; no hay política runtime nueva, proveedor predeterminado nuevo, servicio, tabla, conexiónSQLite ni puerto. El único refactor de producto extrae sin cambiarlo el constructor de payload que usa `choose_measured`, compartido con el control experimental. Los controles reales recibieron las mismas instrucciones y criterios que baseline.

El auditor Rust recompone payloads, modelos/revisiones, namespace, probabilidades, testigos y selección desde cada cuerpo original; comprueba inicios/fin de operaciones y ambos inicios paralelos antes de sus respuestas. Cuenta cada recibo una vez. La operación se degrada atómicamente ante fallo y recoge todas las llamadas ya en vuelo, sin reintentos. Un diario interrumpido a mitad de lote no se califica y requiere reconciliación específica; estos tres diarios completaron sin errores, inciertos ni costes ausentes. La revisión local del código no es revisión ciega nueva.

211pruebas de workspace pasan con y sin la featureHTTP/2; formato y compilación offline pasan. Los tres informes originales se reprodujeron exactamente con el auditor final. Se conservaron hashes y fuentes de cada ejecución antes de los cambios posteriores; no se reescriben protocolos/informes previos.

## Coste y estado del objetivo

Los tres primeros pilotos suman108operaciones de clasificación y137peticionesHTTP, USD0,006301468,148176tokens de entrada y3166de salida. V4añade36operaciones/46HTTP yUSD0,002044444. Total de la tanda:144operaciones/183HTTP y **USD0,008345912**, incluidos controles y variantes fallidas. Son tokens enviados a los proveedores del experimento; no consumo ni coste de Codex. Hay0costes nuevos desconocidos. Acumulado:5292operaciones y23campañas completas, **USD0,549614862 conocidos más4operaciones históricas de importe desconocido**. Los pilotos no se cuentan como nuevas campañas completas.

Pasos1/2 siguen documentados. El paso3demuestra aquí una mejora de calidad conocida, pero queda pendiente por recursos, integración runtime y comprobaciones completas; no habilita pasos4/5 ni declara cerrado el objetivo. Default, README, gold yV6se conservan.

La próxima investigación debe reducir una espera medida de la cadena. HTTP/2no está habilitado; la comprobación aislada offline volvió a fallar resolución de `h2 ^0.4.14`, sin cambiar dependencias del producto. La resolución conectada posterior sí encontróh2=0.4.20; se conservan el fallo offline y el éxito conectado. El contrasteV4siguiente midió esa hipótesis sin relajar el presupuesto. No relajar el presupuesto ni repetir hasta seleccionar una ejecución favorable.

Artefactos: [comparación derivada](../../outputs/iromem/evaluation/context-critics-comparison-v2-2026-10-20.json), [V1](../../outputs/iromem/evaluation/sparse-role-critic-pilot-v1-2026-10-20/report.json), [V2](../../outputs/iromem/evaluation/context-role-critics-v2-2026-10-20/report.json), [V3](../../outputs/iromem/evaluation/context-targeted-critics-v3-2026-10-20/report.json). Protocolos: [V1](sparse-role-critic-pilot-v1-protocol-2026-10-20.md), [aclaración previa](sparse-role-critic-pilot-v1-execution.md), [V2](context-role-critics-v2-protocol-2026-10-20.md), [V3](context-targeted-critics-v3-protocol-2026-10-20.md).


## Contraste posterior HTTP/2 V4

La feature `iromem/http2-transport`, optativa y desactivada por defecto, añadió únicamente soporte de transporte y dependencias fijadas. Las versiones existentes de Cargo.lock se conservan. El [protocoloV4](context-critics-http2-v4-protocol-2026-10-20.md) precede las llamadas y exige negociación realmente observada; no se fuerza ni supone por el nombre de la build.

Las46respuestas negociaron **HTTP/2.0**. Se conserva la selecciónV3, contexto completo, preguntas dirigidas, críticos paralelos y mismo roster.18pares/36operaciones válidas, cero errores/incertidumbres/costes ausentes. Calidad:18/19IDs,17/18sets,18/18preguntas con alguna evidencia permitida o abstención correcta, cero extras, cero pérdidas contemporáneas/históricas y seis IDs útiles adicionales. Coste candidataUSD0,000924544 frente aUSD0,001119900base, razón0,826. p50candidata561,95/base391,09ms, razón1,437; p95candidata1776,19/base853,16ms, razón2,082: **falla recursos**. No se adopta ni recalificaV3por una reducción de unos112ms en otro momento.

HTTP/2operativo no basta para resolver el síntoma; no prueba qué parte de la espera es red o servidor. No afirmar que la apertura de conexión causaba todo ni repetir para escoger un p95favorable. Baseline se midió también bajoHTTP/2. Esta vía ya no está bloqueada por disponibilidad, pero sigue sin candidata aprobada. El siguiente mecanismo debe reducir la dependencia de la etapa lenta conservando las refutaciones/partes y todos los rechazos previos, y superar el mismo piloto antes de ampliar.

[InformeV4](../../outputs/iromem/evaluation/context-critics-http2-v4-2026-10-20/report.json). [Dependencias y límites de verificación](../decisions/0022-opt-in-http2-evaluation.md).
