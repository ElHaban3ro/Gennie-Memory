# 0020 — Testigo de aplicabilidad con citas y comparación local

Fecha de ejecución: 2026-10-19. Estado: experimento descartado para promoción tras contraste nuevo; base intacta.

Se implementa `witness_v1` explícita: proponente Decisions con instrucción `utility_v2`, extracción JSON Schema de referentes/citas con GPT-4.1 Mini y comprobación pura en `retrieval::applicability`. El servicio revalida la prueba antes de componer. `RecallResponse.applicability` es opcional; registra propuesta, extracción y resultado local. La comprobación literal no certifica completitud semántica. No se agregan dependencias, tablas, conexiones, puertos o tareas de fondo. El adaptador usa reqwest ya existente.

El [contrato](../evaluation/applicability-witness-v1-contract.md) fija bindings y citas, estados aceptado/incompatible/no demostrado y errores de evidencia. El [protocolo previo](../evaluation/witness-v1-fresh-protocol-2026-10-19.md) exige cero selecciones no permitidas, cero pérdidas de evidencia respecto a la base y cero degradaciones nuevas, además de ganancia para continuar. Se define el coste adicional de una extracción por propuesta y se conserva gasto conocido en errores.

El [resultado en 64 casos nuevos](../evaluation/witness-v1-fresh-results-2026-10-19.md), con revisión ciega entre agentes, perdió evidencia: respuestas 6/16 frente a 16/16 de la base, refutaciones 5/16 frente a 16/16, parciales 0/16 frente a 14/16. Ambas rutas se abstuvieron en 16/16 ausencias; el proponente ya se abstuvo en ellas sin invocar al extractor, por lo que esa cifra no valida la capacidad de rechazo del testigo. Hubo tres degradaciones por evidencia inválida y ninguna ganancia. De 45 trazas válidas, 34 propuestas correctas fueron rechazadas y 11 aceptadas.

La versión mezcla identificadores de contexto con atributos pedidos/premisas negadas, y el extractor omite subject o devuelve citas de extensión diferente. La igualdad local rechaza incluso información correctiva o cantidades útiles. No se promueve ni se presenta el vaciado de semillas como seguridad demostrada. Se conserva el prototipo para reproducirlo y se mantiene `baseline`, límite 20 y cierre limitado de M5.

La siguiente propuesta debe separar roles de identidad, pregunta y afirmación, y contrastar el comprobador con testigos anotados antes de añadir extracción remota. Cambiar modelo o relajar igualdad sin protocolo nuevo no convierte esta cohorte consumida en reserva. La independencia humana, preguntas reales y respuesta final siguen pendientes.
