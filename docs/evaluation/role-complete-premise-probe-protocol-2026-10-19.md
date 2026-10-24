# Comprobación atómica de premisa: protocolo previo

Solo evaluación, sin proveedor nuevo de producto. El filtro anterior de Perplexity conservó los siete útiles pero aceptó un carné como si negara un avalista. Ambos requisitos pueden coexistir. Se añade una pregunta independiente sobre incompatibilidad de una premisa necesaria; no se cambia el texto de las tres preguntas anteriores ni el estado consulta/recuerdo.

Modelo fijado: `perplexity/pplx-decider-v1.2`, revisión servida exigida `perplexity/pplx-decider-v1.2-20261016`. La selección del modelo responde al patrón conocido del desarrollo; no demuestra superioridad general ni autoriza elegir outputs de distintos proveedores.

La cuarta pregunta se conserva íntegra en `tests/fixtures/premise-incompatibility-v1.json`. Pregunta si los hechos explícitos pueden coexistir con las suposiciones necesarias de la consulta, sin añadir exclusividad o completar información. Un requisito mencionado no agota los demás; un valor desconocido solicitado no es una premisa afirmada.

Composición prospectiva: **dato OR (corrección ordinaria AND incompatibilidad necesaria) OR contraste explícito**. La nueva condición solo limita la corrección ordinaria. El contraste mantiene su comprobación propia de vínculo contextual; no hay veto global sobre datos, parciales o contraste. Se exigen las cuatro respuestas reales incluso cuando una vía ya sea positiva. Falta de una respuesta o error HTTP degrada y detiene ese modelo; no se fabrica un booleano ni se interpreta fallo como abstención.

Mismos doce controles del fixture anterior y mismos 43 positivos congelados. Se preparan las 43 solicitudes antes de red. Ejecutar primero doce una vez; si todos tienen utilidad esperada, revisión coincidente y recibo, continuar con los otros 31 reutilizando los doce resultados. Si no, detener sin ampliar. No modificar gold, omitir negativos, probar bandas hasta pasar o repetir idénticos payloads para escoger los mejores. La nueva pregunta cambia el payload y define un experimento distinto, cuyos fallos anteriores permanecen registrados.

Pruebas offline: igualdad de los tres campos anteriores al retirar la nueva pregunta; verdad de la composición, requisito faltante, dato/contraste con incompatibilidad falsa y ausencia de gold en estado. Auditor Rust independiente reconstruye templates, preguntas, revisión y etapas desde diarios, con todos los recibos y roster común. La pregunta atómica es otro juicio fallable, no una prueba lógica garantizada.

El test manual ignorado `live_complete_premise_probe` usa el secreto existente de OpenRouter sin imprimirlo, bajo la autorización general sintética vigente. Invocación: `cargo test --offline -p gennie-memory live_complete_premise_probe -- --ignored --test-threads=1`. Directorio nuevo de un solo uso, started/respuesta/fallo y huella del ejecutable antes de llamadas. CLI sin puerto, menos de un minuto estimado; doce HTTP iniciales, máximo 43, estimación USD 0,001. No reintentar operaciones inciertas.

Solo con las 43 observaciones completas: reconstruir los mismos trece casos V5/cinco V4 sin IDs no permitidos, pérdidas de base/scopedV1 ni degradación, con ganancia V5 y cinco sets exactos V4. Mostrar todos los omitidos permitidos. Coste por grupo: nuevas comprobaciones + propuesta histórica, ≤1,5× de la base histórica, separado del gasto nuevo. Esto todavía no es coste o latencia end-to-end de un `recall` actual.

Un pase solo habilita implementar una variante optativa con proveedor explícito y medición real del flujo. Después siguen pilotos completos, campañas conocidas, consistencia y V6 reservada/revisada aparte. Ningún fallo autoriza promoción, cambio de defaults o apertura de V6.
