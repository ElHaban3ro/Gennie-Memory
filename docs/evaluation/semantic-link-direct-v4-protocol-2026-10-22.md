# Aportación textual directa V4

Fecha: 2026-10-22. Investigación del paso 3, sin adopción. V3 completó el piloto con 24 fichas válidas, 12 útiles conservados y 8 ajenos seleccionados, incluidos 2 nuevos. Se preserva ese fallo y no se reclasifican sus respuestas.

## Hipótesis y contrato

Se prueba una sola candidata con el mismo Gemini 2.5 Flash Lite: clasificar directamente si el documento aporta evidencia útil para la pregunta. Se eliminan categorías intermedias y el mapa de tokens de pregunta. No está demostrado que la carga de categorías haya causado los errores; el contraste evalúa este paquete nuevo y no una ablación causal pura: cambian instrucción, categorías, mapa de pregunta, origen de decisión y límite de salida.

La pregunta y el documento se transmiten íntegros como datos no confiables, junto a review_id opaco y el mapa documental de tokens canónicos. El modelo devuelve exactamente review_id, judgment (include/exclude/indeterminate), document_evidence_ids y rationale. Solo include selecciona. Una abstención sobre un útil cuenta como omisión. Una salida inválida es error técnico, no abstención aceptada. La decisión procede del modelo; no se la atribuye a una regla de derivación local.

La instrucción general exige alguna aportación explícita al dato solicitado, una corrección pertinente o una ausencia expresamente registrada del dato en el registro pertinente. Puede aportar solo un componente solicitado. Compartir referente, hablar de otro atributo o no mencionar el pedido no basta. Conservar identidad, condiciones, atribución, modalidad y tiempo; un plan futuro no afirma que el evento pasado haya ocurrido, y una negación explícita pertinente puede corregir una premisa. No se agregan ejemplos del banco, IDs originales, gold, scores, fichas manuales ni respuestas de críticos.

Las referencias documentales deben existir, ser únicas, no vacías y estar en orden numérico. El modelo debe referir cláusulas completas con sus restricciones. El adaptador conserva slices originales UTF-8 de los runs elegidos; no comprueba que formen cláusulas completas o incluyan toda negación/condición, ni inventa texto o concatena grupos separados como una sola cita. La literalidad comprueba procedencia, no pertinencia o verdad semántica. Se conserva raw_card, origen de la decisión y semantic_fidelity_verified=false. El índice V4 tiene solo document_spans; V1–V3 conservan sus índices y comportamiento.

## Puertas y alcance

Mismos 432 TRAIN, orden, gold y referencia fija que V3, idénticos byte a byte. Primeras 24 como piloto: cero pérdidas útiles nuevas y cero ajenos nuevos frente al padre, sin fallos técnicos o recibos incompletos. Solo entonces las otras 408, sin repetir el prefijo. Para continuar tras TRAIN completo: al menos una ganancia útil C y cero daños nuevos. Una mejora en un prefijo no demuestra generalización ni autoriza adopción.

Puertas originales 24/24+64/64, ganancia histórica, preservación de seis conjuntos, fidelidad, casos nuevos con revisión ciega, recursos y recibos ≤1,5× y estabilidad intactas. Reservas originales/V6 cerradas hasta que corresponda. No se usa revisión manual como selector de runtime ni se reparan etiquetas o respuestas anteriores.

## Ejecución y coste

Una llamada por par con google/gemini-2.5-flash-lite, proveedor estándar google-ai-studio, temperatura 0, razonamiento desactivado, salida máxima 512 tokens y esquema estricto. Sin fallback ni reintentos. Hasta 432 peticiones, USD 0,50 conocidos más cota de la siguiente, plazo total 900 segundos y 20 segundos por petición; reservar ese plazo antes de cada llamada. Detener ante fallo técnico, presupuesto, plazo, recibo incompleto o regresión del piloto. Modelo y proveedor son alias del intermediario, sin revisión inmutable de pesos.

Recalcular payloads, tarifas, saldo, estimación y cota antes de congelar plan, fuentes, binario e índice. La cota por bytes UTF-8 más framing y máximo de salida es una estimación de ingeniería, no un tokenizer exacto o un límite de facturación impuesto por el proveedor. El piloto V3 costó USD 0,0031114; extrapolarlo no sustituye la nueva estimación ni el gasto observado. Coste Codex y coste económico local desconocidos.

Se reutiliza un solo harness experimental cfg(test), con perfiles explícitos de interfaz y límite de salida. V1–V3 mantienen 1024; V4 exige 512. Preparador Rust offline, marcador exclusivo antes de credenciales, guard de fuentes/binario, diario durable y recibo/cuerpo guardados antes de decodificar. Probar regresiones anteriores, límite de salida, refs/índices y fidelidad separadamente. Preflight independiente antes de inferencia; revisión de fuentes sin gold/métricas de la versión antes de auditoría offline posterior.

CLI propio sin puerto; 2–15 minutos o parada temprana, con PID/handle conservados y observación terminal sin reinicios. Sin cambios de producto/default, DB, secretos, dependencias, servicios, dashboard, rama, commit o push. No actualizar gráficas como mejora de producto sin evidencia. Artefactos: outputs/iromem/evaluation/semantic-link-direct-v4-2026-10-22/.
