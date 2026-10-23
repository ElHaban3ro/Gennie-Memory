# Extracción desde texto mediante subagentes Codex V1

Paso 3; desarrollo conocido. El turno anterior fue progreso: añadió reporte Rust y validación negativa del diagnóstico semántico, preservó errores individuales y cerró snapshot065. El arbitraje posterior no aprobó la puerta original. Ahora se prueba extracción, no se vuelve a puntuar construcciones manuales.

## Alcance congelado antes de generar

Ocho posiciones del piloto anterior grounded-reference: cuatro fuentes y cuatro preguntas, mismos textos, ontología, instrucciones y esquemas. No se seleccionan por resultado ni etiqueta. Se copian únicamente estos datos públicos; ningún frame manual, gold, otro lado emparejado, respuestas previas o scores llega al extractor. Un agente nuevo recibe las cuatro fuentes y otro las cuatro preguntas. El contexto se comparte dentro de cada etapa; no se afirma aislamiento entre los cuatro elementos ni equivalencia con ocho HTTP independientes.

Cada salida declara el SHA256 del paquete recibido antes de sus items. El protocolo registra hashes de ambos paquetes y esta especificación antes de lanzar agentes. No es prueba criptográfica del contexto efectivo del modelo. Una entrega por agente, sin feedback del normalizador ni revisión/corrección de la salida. Se conservan raw, errores y ausencias; nunca se reconstruye un raw para conseguir el pase. Los agentes no consultan otros archivos ni usan red/API, normalizador o resultados anteriores. Esta es ejecución de tareas Codex, no un transporte productivo ni una inferencia única.

El normalizador Rust usa el esquema grounded-reference V1, traductor y parser V5 existentes sin cambiar su significado. Intenta las ocho filas, conserva error por etapa y no rellena ausencias. Rechaza paquetes alterados, IDs ajenos/duplicados/ausentes, hash de entrada incorrecto o exposición incompatible. El coste/tokens Codex y latencia de producción son desconocidos; cero llamadas OpenRouter previstas. No se inventan recibos HTTP para representar una tarea local.

## Crítica congelada

Después de normalizar, dos críticos nuevos reciben los ocho textos/raw/normalizados/error con contrato público, sin frames manuales, gold, resultados históricos ni métricas. Ambos juzgan fidelidad al texto, completitud, atribución de evidencia y preservación operativa; labels faithful/defective/uncertain/unavailable. Cada finding contiene razón, cita textual literal y path JSON existente del raw/normalized que localiza el defecto; un error de parser usa el path /normalization_error. Un DTO ausente/inválido no puede pasar preservación. Revisar todas las cláusulas/premisas/slots, polaridad, modalidad, referente, restricciones, medición, unidades, fundamento y lecturas. No exigir a la fuente la respuesta de una pregunta no presentada, no tratar una pregunta como afirmación. Una clase literal no demuestra que se atribuya al referente correcto. Sinónimos de identidad sin pérdida de referente no son falsedad, pero una pérdida operativa sí cuenta.

Se conservan ambos dictámenes completos. Si difieren, un árbitro nuevo ve todos y solo los desacuerdos con propuestas A/B alternadas, sin gold ni métricas, antes de cualquier revisión del extractor. Su resolución es diagnóstica y no borra el fallo de la puerta original.

## Puertas y siguiente acción

Puerta del piloto: 8/8 raws completos válidos y 32/32 dimensiones faithful para CADA crítico. Cualquier defecto, incertidumbre, ausencia o desacuerdo impide ampliar. El arbitraje identifica causas y no relaja esa puerta. Una eventual nueva versión se congela antes de repetir todo el piloto, conserva V1 y no añade ejemplos tomados del gold. Solo después de un pase se prepara el banco conocido completo, revisión semántica, proyección y comparación con baseline; siguen pendientes conocidos100/126, recursos/estabilidad y cohorte nueva ciega. No hay adopción por aprobar ocho textos conocidos. La clasificación API permanece autorizada; los modelos frontera se consultan mediante subagentes Codex.

No se alteran datos/gold anteriores, producto, secretos, SQL, dependencias, dashboard, servicios o V6. Verificación proporcional: normalizador y negativos de integridad/conformidad; pruebas offline del evaluador, replay de bancos/reportes afectados, fmt. Registrar hashes de fuente/binary ejecutados y costes separados. No actualizar gráficas de recuperación por un diagnóstico sin medición nueva de recall.
