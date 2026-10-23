# 0019 — Compatibilidad del referente antes de utilidad

Fecha: 2026-10-18. Estado: experimento de desarrollo descartado para promoción; no cambia la base.

La decisión 0018 descartó `utility_v1` tras seleccionar datos de otra edición, equipo o entorno. Se añade `selection_policy: utility_v2` explícita, con una instrucción que comprueba compatibilidad de sujeto/contexto antes de admitir respuesta, refutación o parte útil. Conserva la opción `none` de v1, una llamada y las reglas de autorización y composición. El chequeo es semántico dentro de la instrucción, no un filtro determinista. No se añaden dependencias, estado persistente, conexiones ni servicios.

El [protocolo previo](../evaluation/utility-v2-development-protocol-2026-10-18.md) conservó los bloqueos de selecciones no permitidas, pérdida de evidencia útil y degradación nueva. El [resultado](../evaluation/utility-v2-development-results-2026-10-18.md) en 64 preguntas conocidas mantuvo 16/16 respuestas, 16/16 refutaciones y recuperó contexto parcial de 13/16 a 16/16, pero redujo abstenciones de 16/16 a 14/16. Persistió una cuenta de pruebas para una consulta sobre producción y apareció un código de cuarzo para una consulta sobre feldespato. No se compensan con ganancias ni se promueve por tener dos fallos frente a tres históricos. El coste y tokens reportados de la variante crecieron 28,41 % respecto a su base emparejada.

Los datos ya motivaron la hipótesis y se declaran desarrollo conocido, aunque conservan anotación ciega anterior. No se ajustó tras ejecutar ni se abrió nueva validación de una candidata que falla el filtro. Se mantiene `baseline`, límite 20 y cierre limitado de M5. Las políticas experimentales permanecen explícitas para reproducirlas.

Una propuesta posterior necesita hacer inspeccionable la correspondencia entre referente solicitado y del recuerdo, con evidencia; ampliar su contrato y validación tiene coste y no se incorpora aquí. Ninguna extracción semántica se presume infalible. Criterios, datos nuevos y recursos deben fijarse antes de medir otra variante, sin reabrir esta cohorte como reserva.
