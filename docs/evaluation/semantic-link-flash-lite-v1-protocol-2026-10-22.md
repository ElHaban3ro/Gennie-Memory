# Candidata automática de vínculo semántico

Fecha: 2026-10-22. Paso 3 del objetivo íntegro. Una sola candidata, `semantic-link-flash-lite-v1`, sobre TRAIN existente. La ficha manual anterior obtuvo 51/52 decisiones concordantes y una identidad indeterminada fiel; su puerta prospectiva permanece fallida. No se convierte ese resultado manual en extracción automática ni se modifica gold.

## Mecanismo y contrato

Gemini 2.5 Flash Lite (`google/gemini-2.5-flash-lite`), proveedor estándar `google-ai-studio` exclusivamente. Sin flex, priority, fallback, retries o razonamiento. Temperatura 0, JSON Schema estricto y máximo 1.024 tokens de salida. El alias servido y proveedor se verifican; no hay revisión inmutable de los pesos remotos. Se reutiliza el transporte generativo con plazo total existente; no hay política nueva de producto.

Por cada pregunta/recuerdo completos devuelve los nueve campos de la ficha: ID opaco, foco solicitado, citas de pregunta y documento, vínculo de referente, vínculo de foco, aporte, decisión y justificación. Inputs del modelo: únicamente ID opaco, pregunta y documento. No recibe gold, roles, scores, IDs originales, fichas manuales o ejemplos específicos de los bancos. Los textos se tratan como datos.

Decisión operativa prospectiva: solo `include` selecciona. `exclude` e `indeterminate` no seleccionan, conservando razón y deuda; no se presenta la duda como prueba de irrelevancia. Un útil indeterminado cuenta como omisión y como pérdida si el padre lo aceptaba. Errores de transporte, esquema, citas o recibos son fallos técnicos, nunca incertidumbre semántica. No se responde la pregunta ni se mezcla la selección con el padre mediante OR/AND.

Rechazar campos extra/duplicados, enums/ID/citas no válidos e incoherencias con la regla de decisión. Cada cita debe existir literalmente en su texto correspondiente. Validar procedencia no prueba fidelidad semántica. Guardar body y recibo antes de validar contenido para no perder cargos ante una salida inválida.

## Datos, filtro y detención

432 pares: background176 y pool256 originales, mismo orden, textos y gold. IDs de transporte `pair_0001`…`pair_0432`; mapa original solo para auditoría. Referencia fija: decisiones reales del padre V5matched en la ablación145, ligadas porhash. Es un filtro comparativo en TRAIN ya observado, no un control contemporáneo de recursos ni comparación de producto.

Primero, las24primeras filas background en orden original. Solo con recibos completos, salida técnicamente válida y cero nuevas pérdidas útiles o ajenos frente al padre se ejecutan las408restantes. No se repiten las24primeras ni se elige la mejor corrida. Si falla el prefijo, conservar resultados parciales y no extrapolar al conjunto pendiente. El piloto no exige ganancia en ese prefijo. Toda salida indeterminada útil cuenta como omisión.

Filtro completo TRAIN: al menos una ganancia útil C y cero nuevas pérdidas útiles o ajenos frente al padre. No sustituye la puerta original24/24+64/64, ceroerrores/degradaciones y ganancia histórica, ni la preservación de seis conjuntos. Los88, las sondas y reservas originales/V6 no se abren aquí. Casos nuevos con revisión ciega, comparación productiva, recibos/recursos≤1,5× y estabilidad siguen pendientes antes de adoptar.

## Presupuesto y ejecución

Catálogo actual y saldo consultados sin inferencia. Tarifa estándar observada: USD0,10/Mentrada y USD0,40/Msalida. Estimación y cota de cada request se calcularán con los payloads finales antes de congelar el plan. Techo absoluto de fase: USD0,50; el plan fija el menor presupuesto viable tras el cálculo, con saldo suficiente. El techo provisional0,25 de preparación no era un presupuesto final; cualquier cambio se explica con el cálculo previo. No se afirma tokenización Gemini exacta: se usa cota conservadora UTF8bytes más margen explícito, registrando su límite frente a framing interno del proveedor.

Máximo432HTTP de inferencia únicos, deadline de fase900s y plazo total de20s por request. Antes de cada llamada comprobar coste conocido más cota siguiente y reserva de tiempo. Parar ante error HTTP/429, recibo ausente, truncamiento, cita inválida, presupuesto/plazo agotado o degradación del piloto. Sin fallback, repetición del intento incierto o nueva tanda automática. Diario started/response/failed durable y terminal explícito; una operación sin recibo queda con coste desconocido, no cero.

CLI de prueba propio sin puerto; estimado2–15min según latencia y posible detención temprana. Identificar PID/handle, observar hasta terminal, sin reiniciar por timeout de observación ni detener procesos ajenos. No modificar DB/secretos/dependencias/defaults/servicios/ramas o entregar commits. El coste de agentes Codex y preparación local es desconocido y separado de API. No hay afirmación de ahorro o rendimiento de producto.

Congelar protocolo, contrato, datos, payloads, fuentes, catálogos, cotas, hash del binario de prueba exacto y plan antes de inferencia. El runner exige ese hash antes de credenciales/diario; un binario distinto no ejecuta inferencia. Rechazar recibos con más de1.024tokens de salida; preflight independiente y pruebas locales con respuestas falsas. Después, auditoría offline de diario, recibos, contenido, selección y gate. Artefactos: `outputs/iromem/evaluation/semantic-link-flash-lite-v1-2026-10-22/`.
