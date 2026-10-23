# Vínculo semántico V2 con referencias a spans

Fecha: 2026-10-22. Paso3activo. Una sola candidata, `semantic-link-span-v2`. V1 permanece detenida con1respuesta,0juicios aceptados yUSD0,0001028; no se repara su cita, reejecuta su intento o cambia gold. Se contrasta una interfaz nueva después de diagnosticar la recopia de una mayúscula y un foco ocupado por el valor del documento.

## Interfaz y fuente

Mismo Gemini2.5FlashLite/GoogleAIStudio estándar, temperatura0, reasoningfalse, max1024, sin fallback/retries. Los textos íntegros se acompañan de mapas compactos de IDs→tokens. Tokens: runs máximos de caracteres no-whitespace según Unicode char::is_whitespace de Rust, con offsets UTF8byte start/end exclusivos en la vista canónica. Sin cambios de case, puntuación, Unicode o espacios. IDsq0… y d0… exactos, sin aliases con ceros adicionales. El mapa mostrado sigue el orden numérico de fuente.

Ocho campos remotos: review_id, query_focus_ids, document_evidence_ids, referent_link, focus_link, contribution, decision, rationale. Se elimina requested_focus libre y las citas libres. Las referencias deben existir en la fuente correcta, ser únicas, no vacías y estar en orden numérico creciente. El adaptador rederiva la vista desde la fuente y reconstruye cada run de tokens consecutivos como un slice original, conservando sus separadores; runs no consecutivos producen citas separadas. No une artificialmente, cambia mayúsculas o repara referencias.

El foco solo puede referir tokens de pregunta: un valor exclusivo del documento no puede ocuparlo mediante copia libre. Un ID existente no demuestra que el foco o evidencia elegidos sean pertinentes. La fuente íntegra, categorías y razón se conservan. Una cita construida mecánicamente prueba procedencia, no fidelidad semántica, verdad o suficiencia.

Include selecciona. Exclude/indeterminate no seleccionan y conservan razón/deuda; toda abstención útil cuenta como omisión. Se mantienen las mismas reglas de referente, foco, corrección, parcial y ausencia documental. Referencias inválidas o errores técnicos no son incertidumbre semántica. El cambio de interfaz incluye instrucción, campos, vista y composición: no se atribuye cualquier diferencia futura a una causa pura de capacidad del modelo.

## Datos y puertas

Mismos432TRAIN y decisiones fijas del padre del diagnóstico145; inputs.json idéntico byte a byte al deV1. IDs opacos pair_0001…0432; no gold/roles/scores, IDs originales o fichas manuales en mensajes. Span-index.json se reconstruye porfuente antes de marcador/credenciales; el preparador offline produce la vista y payloads, sin red.

Primeras24filas background originales: solo sin pérdida útil nueva o ajeno nuevo y sin fallo técnico/recibo se ejecutan las408restantes. No se repite ese prefijo dentro del intento. El piloto no exige ganancia; el filtro432exige≥1ganancia útilC y cero daños nuevos contra el padre. No equivale a la puerta original24/24+64/64, preservación de seis conjuntos, casos nuevos ciegos o comparación productiva. Reservas yV6cerradas.

## Recursos y verificación

Catálogo ysaldo actuales consultados sin inferencia; tarifa estándar observadaUSD0,10/Mentrada yUSD0,40/Msalida. Recalcular todos los payloads y costes antes de inferencia; cota de planificación porrequest: bytesUTF8delpayload+1024framing estimado, más1024salida. No es tokenización exacta ni garantía del proveedor. Plan con techo≤USD0,50 y coste conocido+cota siguiente comprobados antes de cada llamada; máximo432HTTP,900stotal y20sporrequest, sin reintentos. Parar ante error/429, recibo incompleto, truncamiento, referencia inválida, piloto degradado o límites agotados. No convertir pendientes en predicciones.

Un único harness manualcfg(test) distingue V1/V2porconfig explícita y directorios/versiones. V1 conserva reglas/recibos y su marcador impide reejecución. Dos helpers compartidos para categorías/recibos no cambian semánticaV1. El binario exacto y las fuentes se ligan antes del marcador/credenciales. Guardar body/recibo antes de decode; fuente y respuesta canónica son evidencia, no wirebytes originales.

Pruebas de Unicode/offsets, whitespace, grupos contiguos y separados, refs cruzadas/inexistentes/duplicadas/desordenadas, foco soloquery y regresiónV1. Suite workspace predeterminada/HTTP2 y formato, preflight independiente de432vistas/payloads/cotas/binario antes de red. Despuésauditoría de diario, recibos, composición, selección y gate. Recursos productivos≤1,5×, estabilidad y fidelidad siguen pendientes aunque el formato pase.

CLI propio sin puerto, estimado2–15min o detención temprana. Mantener PID/handle y observar terminal sin reiniciar por timeout o detener procesos ajenos. Ningún cambio a producto/default, DB, secretos, dependencias, dashboard, rama, commit o push. Coste Codex/local desconocido. Artefactos: outputs/iromem/evaluation/semantic-link-span-v2-2026-10-22/.
