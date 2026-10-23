# Verificación por rol en lote V1: protocolo previo

Hipótesis experimental, sin cambiar la base predeterminada. Las verificaciones individuales V1 rechazaron correctamente las dos confusiones V4; su compactación V2 añadió errores y elevó el número de checks. Se conserva íntegramente la comprobación de datos V1 y la de correcciones V4. La propuesta es la de tipo real V2, incluido su contexto parcial de terminación. No se mezclan selecciones de corridas previas.

## Representación y contabilidad

`per_memory_role_batch_v1` conserva búsqueda y prefijo de siete recuerdos, Jev 1.13 para propuesta y Decisions para verificación. Todo positivo necesita su respuesta propia; una corrección requiere las dos vías tipadas y su OR. Si hay un positivo, la petición aislada previa es idéntica. Con varios positivos, el estado comparte consulta e instrucciones completas por tipo; cada pregunta contiene exclusivamente su resumen literal entrecomillado como dato. Otros recuerdos no aparecen en el estado común. Esta separación del payload no prueba aislamiento dentro del proveedor.

Se hace una propuesta y, solo si hay positivos, una petición de comprobación: máximo dos HTTP por operación. `verification.batch` contiene `usage` global y `verdicts` por ID/revisión. `checks` individual queda vacío; no se inventan recibos por recuerdo. Servicio y auditor JSON independiente rechazan mezcla de esquemas, respuestas omitidas, paths impropias, revisiones ajenas y costes duplicados. Un fallo de lote degrada la operación completa conservando recibos conocidos de propuesta y lote; ausencia de recibo no equivale a cero.

No hay cambios SQL, conexiones, dependencias, secretos, variables de producto, puertos o modelos por defecto.

## Comprobaciones antes del banco

Pruebas offline de petición única idéntica, instrucciones completas compartidas, cambios limitados a la pregunta que contiene un recuerdo alterado, vinculación de respuestas, un solo recibo, falta de respuesta, fallo atómico y validación independiente de servicio/auditor.

Diagnóstico remoto fijo: cuatro escenas nuevas de desarrollo, no holdout, con un dato correcto, un dato de otro contexto, una corrección explícita y una corrección de otro objeto. Cada escena se presenta sola, junto a dato ajeno y junto a texto adversario que intenta imponer respuestas. Las doce peticiones se ejecutan una vez; no se repiten fallos inciertos ni se elige la mejor respuesta. Se exige el juicio esperado en las doce y que el distractor no cambie el juicio del objetivo. Esto es evidencia limitada, no garantía de aislamiento o generalización.

El test manual ignorado `live_role_batch_isolation_probe` conserva los doce payloads y expectativas antes de llamar, huella del ejecutable, estado started y respuesta/fallo por operación con recibos. Su directorio es de un solo uso. Normalmente `cargo test --offline --workspace` lo omite. Solo se invoca expresamente con autorización sintética vigente, `--ignored --test-threads=1`; lee el secreto existente desde la raíz del workspace y no lo imprime.

Estimación del diagnóstico: menos de un minuto, USD 0,001 y máximo doce HTTP. CLI breve sin puerto. La autorización del usuario cubre pruebas sintéticas mediante OpenRouter. Si falla un juicio, detener ampliación y conservar resultados.

## Puertas posteriores intactas

Solo si el diagnóstico anterior y las pruebas de contratos pasan: mismos trece casos V5 y cinco V4 conocidos y congelados, una corrida completa de cada grupo. Máximo 36 HTTP, estimación USD 0,003 y uno a dos minutos. Exigir V5 sin ID ajeno, pérdidas de base/scopedV1 ni degradación, con alguna ganancia; V4 cinco conjuntos exactos; coste por grupo ≤1,5× del subconjunto histórico. Mostrar también IDs permitidos omitidos y percentiles, sin confundir preservación con completitud.

Solo tras esos pases: campañas completas conocidas V5/V4/V2 con puerta conjunta global Y preservación, recursos coste/p50/p95 ≤1,5×, prueba de consistencia fija y finalmente V6 congelada/revisada aparte. No usar V6 para ajustar, modificar gold o sustituir la comparación por una selección retrospectiva de mejores casos.
