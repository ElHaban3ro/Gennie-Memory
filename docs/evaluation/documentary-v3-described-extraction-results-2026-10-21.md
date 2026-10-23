# Extracción V3 con descripciones: ejecución interrumpida, revisión y límites

La tanda no supera la puerta. Se recibieron 80 de 116 documentos, todos estructuralmente completos; 72 de 116 tienen las cuatro dimensiones faithful para ambos críticos originales. Los otros 36 textos siguen en el denominador, sin extracción disponible. No hay utilidad automática, comparación con la base, recursos/estabilidad ni V6. No se cambian producto o configuración vigente.

## Ejecución y verificación

Los tres primeros autores fueron despachados en el turno anterior. Tras la interrupción explícita de ese turno, el listado autorizado de agentes solo devolvió root y no existía ninguna de sus tres salidas. Se registraron los 36 textos como unavailable mediante el receptor Rust --assembled-only. No se confundió un timeout con terminación ni se repitieron sus inferencias. El motivo interno por el que desaparecieron los contextos no está demostrado más allá de esa observación de ausencia tras la interrupción.

Se ejecutaron los siete lotes restantes y se conservaron sus bytes originales, derivados ensamblados, hashes y recibos. Los 47 textos fuente y 33 preguntas recibidos tuvieron dos críticas aisladas por lote, catorce entregas en total, todas con integridad comprobada. Las 36 entradas sin raw no recibieron crítica semántica: sus 144 dimensiones por crítico figuran como no disponibles, nunca faithful. El protocolo original y los diez paquetes permanecen intactos; no se cambió una expectativa después de ver resultados.

| Medición sobre el denominador original | Resultado |
| --- | --- |
| Documentos completos | 80/116 |
| Sin extracción disponible | 36/116 |
| Cuatro dimensiones faithful para ambos críticos | 72/116 |
| Dimensiones crítico A | 308 faithful, 12 defective, 144 unavailable |
| Dimensiones crítico B | 311 faithful, 9 defective, 144 unavailable |
| Desacuerdos entre críticos | 5 dimensiones |
| Entradas entregadas con algún defecto señalado | 8 |
| Puerta 116 completas y 464 faithful por crítico | falla |

Los 80 documentos completos y las 72 aceptaciones son resultados parciales de desarrollo conocido, no recall ni un porcentaje de calidad poblacional. La extracción anterior tenía 101/116 completas y 98/116 aceptadas por ambos críticos. La nueva falta de 36 salidas impide atribuir la diferencia a calidad semántica o rendimiento del mecanismo. Tampoco se presenta la proporción sobre los 80 entregados como aprobación de una muestra reducida.

## Hallazgos conservados

- s4-046: ambos críticos cuestionan la completitud del cierre de una configuración. Conservar una cantidad bajo un modo y cerrar los modos solo bajo esa cantidad deja abiertas otras configuraciones. Hay diferencias adicionales entre críticos sobre fidelidad y atribución.
- s4-059: ambos críticos señalan que una frecuencia literal se especializó como sample_rate sin que el texto afirmase muestreo. El número y su unidad no justifican esa función.
- s4-061: ambos señalan una cita de clase tomada de la cláusula de otro referente. Igual palabra no autoriza atribución entre cláusulas.
- s4-073: el crítico A cuestiona la lectura de entrega como inscripción; B la acepta. Se mantiene el desacuerdo, sin adjudicar una corrección de etiqueta.
- q4-005/008/010: ambos críticos exigen dependencias explícitas hacia referencias usadas dentro de calificadores. El validador actual acepta los documentos; debe resolverse la alineación entre el alcance del requisito de dependencia y la comprobación directa del grafo antes de atribuir una mejora a nuevas etiquetas.
- q4-017: ambos críticos cuestionan occurred_year por no figurar en la notación resumida. Una auditoría posterior confirma que el contrato vigente y el código ya permiten ese slot y format:year, omitidos en la notación suministrada a revisión. Este defecto del paquete no se arregla modificando el raw ni reinterpretando el score original. La adenda prospectiva separada corrige la descripción del contrato para la próxima versión.

La auditoría del año es del coordinador, retrospectiva y con contenido conocido; no es un tercer crítico ciego. Con la misma pregunta raw y una fuente sintética manual, la CLI valida ambos documentos y reconoce un término de año en respuesta parcial, pero conserva premise pendiente. No prueba una respuesta completa ni calidad de extracción. Los intentos iniciales están conservados: un error del adaptador del diagnóstico (scope en vez de pair_scope) y dos expectativas falsas de conversión desde literal/date. El motor exige año explícito con format:year; no proyecta fechas. La nueva prueba confirma solo el slot y esa limitación, sin reetiquetar la campaña.

## Calidad, costes y verificación del código

No se cambió código. Sus hashes y el binario coinciden con el snapshot 093: 656 pruebas pasadas, 24 omitidas, 18 controles CLI anteriores y 226 históricos idénticos. Esa suite no se repitió ni se atribuye al turno actual. La ejecución actual verifica ensamblado/recepción, catorce entregas de críticos y el diagnóstico manual; integridad de dictámenes no demuestra que sus juicios sean correctos.

Esta tanda usó diez contextos de autor contando los tres perdidos, y catorce de crítico. De ellos, siete autores y catorce críticos se abrieron en esta continuación. Todos los contextos observables están terminados; no hay servicios nuevos, puertos o inferencias en curso. Coste y tokens Codex desconocidos, no gratuitos ni incluidos en el ledger de API. Cero HTTP de proveedor y USD 0 de API nuevos. Acumulado intacto: 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos, sin Codex.

El tiempo de coordinación incluye una interrupción y esperas de agentes; no se presenta como latencia de producción. No hay medición nueva de tokens de recall ni de respuesta final. Las gráficas de rendimiento anteriores conservan sus mediciones históricas.

## Siguiente trabajo del objetivo

Resolver con controles genéricos el contrato de dependencias de calificadores y la premisa temporal pendiente, y alinear notación/validador/revisión. Corregir la selección semántica excesivamente específica y comprobar cierres de configuraciones completas y citas locales. Solo después preparar otra versión completa, sin reparar ni rescatar la actual, con evidencia de ejecución y revisión íntegra antes de utilidad automática y comparación. V6 queda reservada. El objetivo sigue activo en el paso 3; no se reduce el criterio de éxito.

Artefactos: outputs/iromem/evaluation/documentary-v3-described-extraction-2026-10-21/, incluido report.json, raws originales, recibos, revisiones, integridad, interrupción, desacuerdos, diagnóstico, adenda prospectiva y manifiestos. Snapshot 095 conserva esta tanda y todos los snapshots previos.
