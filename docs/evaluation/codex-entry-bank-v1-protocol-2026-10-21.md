# Banco completo de extracción Codex con entry_via V1

Continuación del paso 3 después de piloto V2: ocho DTO completos y faithful para ambos críticos; solo pasa desarrollo conocido de ocho textos. El turno anterior fue progreso verificable, no cumplimiento del objetivo total. Se preservan snapshot067, el piloto V1 fallido y todas las referencias/resultados previos.

## Entradas y ejecución congeladas

Se extraen los 83 textos fuente y 33 preguntas del banco documental conocido, 116 representaciones distintas para 100 pares. Se verifica integridad del banco y del pase anterior; reproducir su control manual no demuestra fidelidad semántica de las referencias. No se selecciona un subconjunto por scores/gold. El nuevo vocabulario, guía y esquemas son exactamente los del piloto V2, incluida entry_via y sus restricciones. No se añaden ejemplos, alias, frames manuales ni anotaciones a los paquetes.

Diez lotes deterministas: siete de fuentes (12/12/12/12/12/12/11) y tres de preguntas (11/11/11), orden por ID técnico original. Cada paquete contiene exclusivamente versión, etapa, ontología, instrucciones y sus items de ID opaco/texto/esquema. El mapa de ID opaco a referencia del banco queda fuera de los paquetes. Sin otro lado emparejado, resultados anteriores o gold. No se comparte contexto entre etapas; sí entre items del mismo lote. Cada encargo usa contexto nuevo de agente. Una entrega raw por lote, sin normalizador, feedback o reparación. El packet SHA256 va en la entrega. Protocolo/paquetes y copias parciales de código se ligan antes de lanzar; los hashes no prueban criptográficamente el contexto efectivo.

Campaña local de tareas Codex, sin proveedor remoto: cero llamadas OpenRouter previstas; coste/tokens Codex desconocidos. Duración orientativa 20–40 minutos para extracción y doble revisión, dependiente de las tareas. No es coste/latencia runtime. Se completan todos los lotes aunque aparezca un fallo de contenido: el denominador no se filtra. Se conservan salidas, ausencias y errores. No se reinicia ni reintenta automáticamente una tarea con estado incierto.

## Normalización y crítica

Preparación y reporte Rust separados del producto. El reporte reconstruye todos los paquetes desde los textos congelados y el contrato V2. Protocolo/paquetes alterados son error de integridad. Una entrega ausente o sin cobertura/vínculo/formato fiable degrada todo su lote, conservando su raw original y todas las filas previstas; no se remapea para recuperar respuestas buenas. Una fila de contenido inválido degrada solo esa fila. Se aplican schema grounded, reserva property para entry_via, traductor y parser existentes. No se fabrican journals/recibos HTTP.

Después dos críticos nuevos por lote reciben texto/raw/normalizado/error y contrato, sin gold, frames manuales, histórico, score ni otros dictámenes. Cuatro dimensiones faithful/defective/uncertain/unavailable por fila: fidelidad, completitud, atribución y preservación operativa. Cada nofaithful exige cita literal del texto fuente y path existente. Un normalized ausente no pasa preservación. Revisar todas las cláusulas, valores, restricciones, lecturas, premisas y slots; una cita sidecar no conserva por sí sola significado. No exigir a una fuente la respuesta de una pregunta no presentada. No cambiar atributos de una pregunta para hacerla coincidir. Los árbitros, solo si hay discrepancias, ven todos y solo desacuerdos con A/B alternado, sin gold ni métricas; resuelven diagnóstico, no borran fallos de la puerta original.

Puerta: 116/116 DTO completos y 464/464 juicios faithful PARA CADA familia de críticos, con cobertura de cada lote. Un defecto, incertidumbre, ausencia o desacuerdo impide pasar. No se eligen mejores respuestas según gold ni se modifica el banco para aprobar. La crítica es del mismo sistema de modelos, no validación humana/externa. Si falla, conservar todo y corregir la causa en una versión prospectiva.

## Referencias, utilidad y recursos

Antes de interpretar concordancia con referencias manuales como calidad, revisar por separado la atribución dudosa a028 y cualquier otra discrepancia detectada, con referencias/puntuaciones ocultas a los críticos. La revisión de una representación manual no recibe resultados de extracción. Conservar referencias originales, desacuerdos y adjudicaciones retrospectivas como tales. No reinterpretar pérdida de contexto parcial como irrelevancia. Una etiqueta nueva no valida retrospectivamente el mismo caso observado.

El reporte de normalización no usa etiquetas para juzgar fidelidad ni puntúa recuperación. Solo tras cerrar fidelidad y referencias se mide proyección real, conservación de evidencia útil y comparación emparejada con baseline. Siguen pendientes conocidos126, recursos/estabilidad y V6 ciega. Si las referencias no permiten concluir, reportar incertidumbre y mantener la puerta abierta. Nunca adoptar por exactitud de forma o por el 100/100 manual anterior.

Verificación proporcional: derivación/cobertura completa, exclusión de frames/gold, IDs opacos, metadatos alterados, entrega incompleta/duplicada/ajena y contenido inválido sin filtrado ni reparación; replays de pilotos y bancos históricos sin diferencias, fmt/suite offline del evaluador. No cambiar producto/default, SQL, dependencias, secretos, dashboard, servicios, README/gráficas o V6. Goal activo mientras falte cualquiera de las cinco puertas.
