# Utilidad V4: agrupación de gradientes por consulta

Fecha: 2026-10-22. Protocolo prospectivo de dos brazos seriales. V3 falló: original 10/17 útiles conocidos y ampliado 9/17. El ajuste 166/176 del ampliado tampoco fue completo. [Revisión y límites](utility-learning-v3-results-2026-10-22.md): nueve de sus diez errores finales ya fallaban en la última observación de entrenamiento. No se atribuye todo a olvido ni se afirma una causa demostrada.

## Hipótesis y factor

Comparar acumulación de cuatro filas aleatorias (`rows`, control contemporáneo) con acumulación de cuatro contribuciones agrupadas por consulta (`queries`, candidata). Hipótesis: agrupar ejemplos pertinentes para la misma pregunta podría facilitar distinguir cambios mínimos de significado y reducir interferencia. No se han medido alineamientos de gradientes ni descartado límites de optimización, cobertura o capacidad. Si falla, se conserva el resultado; no se prueba otra agrupación sobre estos scores.

Cada brazo empieza por separado de la misma base Qwen3-Reranker-0.6B/revisión e61197ed45024b0ed8a2d74b80b4d909f1255473 y la misma inicialización LoRA de V1–V3. Ambos entrenan los mismos 176 pares: 48 originales y 128 de extensión, 72 útiles y 104 ajenos. Seis épocas, 1056 forward/backward, 264 actualizaciones, cada fila exactamente una vez por época. Único factor cambiado: empaquetado y orden de las contribuciones de cada actualización; la forma de combinar su gradiente no cambia. No añadir pérdidas de ranking, top-1, pesos de clase, cuotas de selección, contenido o etiquetas.

`rows`: NumPy default_rng PCG64 semilla 20261022, permutación de 176 filas por época, exactamente igual a V3 expanded. `queries`: con RNG independiente de la misma clase y semilla, en cada época permutar las 24 consultas antiguas (dos filas cada una) y emparejarlas consecutivamente en 12 grupos de cuatro; añadir los 32 pools nuevos íntegros de cuatro filas. Permutar los 44 grupos y permutar las cuatro filas dentro de cada grupo, en ese orden de llamadas al RNG. Todas las órdenes y grupos quedan congelados antes del primer score del primer brazo. El sampler lee índices, query y pool_id; nunca label, role o rationale. Query/pool solo organizan el entrenamiento, no entran como metadatos al modelo ni a la pérdida.

## Constantes y separación

Conservar V3 completo: base FP16 congelada, LoRA FP32 q/v últimas ocho capas, rango 8, escala 8, dropout 0, 32 tensores y 327680 parámetros. CE sobre no/yes FP32 más `y*relu(2-margin)^2`, margen 2, peso 1. AdamW lr 0,0001, betas 0,9/0,999, eps 1e-8, weight_decay 0,01, bias_correction verdadero; media de cuatro antes de clip global 1. Sin scheduler, compilación, caché de K/V entre ejemplos, cambio de semilla, umbral, checkpoint o época según métricas. Corte fijo 0,5, solo checkpoint final elegible.

Textos íntegros, instrucción y tokenización fijadas en V1; >32768 tokens falla sin truncar. Target binario solo para pérdida en train. Desarrollo 24 y conocidos 64 son regresión observada, nunca entrenamiento ni validación fresca. Los 48 originales y 128 de extensión son in-sample en ambos brazos. No leer los 24 cerrados ni V6; comprobar únicamente hashes/metadata de reconciliación como en V3. No cambiar etiquetas ni documentos para mejorar resultados.

Congelar runner, protocolo, sampler, HP, entradas, órdenes y runtime antes de scores. Mismas versiones y seis fuentes canónicas de runtime. Verificar assets, objetivo/derivadas (10 controles escalares), nombres/dtypes de adaptadores, hash de inicialización igual a V1–V3 y base antes/después. Antes de entrenar: 264 scores base y 64 con adaptador cero por brazo, mismos IDs conocidos/tokenids históricos y decisiones de base, delta máximo 0,02. Registrar cada inicio/respuesta, pérdida, CE, penalización, margen, norma antes de clip y update en diario UTC RFC3339 con milisegundos y ordinal. El checkpoint y sus hashes de HP/datos/órdenes se registran antes de evaluar pesos entrenados. Verificar directamente alineación de grupos por consulta sin roles ni etiquetas.

## Puertas y selección

Ejecutar rows y confirmar su handle terminal antes de queries; ambos modelos nunca residentes simultáneamente. Cualquier fallo técnico detiene el intento sin rescate/reintento en el mismo directorio; no convertir ausencias en errores semánticos. La calidad fallida del primer brazo no cambia el plan del segundo, pero ambos deben ser técnicamente evaluables para selección.

Puerta original sin rebajar: 24/24 desarrollo y 64/64 conocidos, cero pérdidas útiles/ajenos/degradaciones y al menos una ganancia sobre base. Informar todos los errores entrenados. Si ambos completan, elegir queries si pasa; si queries falla y rows pasa, elegir rows; si ambos fallan o alguno es técnicamente no evaluable, ninguna candidata. Solo tras ambas corridas completas y elegible congelada, abrir closed 24 una vez para la elegida y exigir 24/24; nunca cambiar de brazo para repetir reserva. V6 no se abre en este ensayo. Si ninguna pasa, cerrado permanece intacto.

Después siguen pendientes casos nuevos con pools mixtos y revisión ciega, comparación contra configuración vigente FTS5/Decisions, calidad sin pérdidas, recibos completos, coste/p50/p95 ≤1,5×, estabilidad y finalmente V6. La base de esta comparación es el reranker sin adaptar, no el producto. No modificar defaults, README o gráficas de rendimiento vigente con ajuste de entrenamiento.

## Presupuesto y conservación

Estimación 20–30 minutos total; watchdog independiente de 1200 s por brazo, máximo 2400 s seriales. Cada brazo: 1056 forward/backward y 264 updates; 592 scores (264 base +64 cero +264 final), 1648 forward transformer. Total: 3296 forward, 2112 backward, 528 updates y 1184 scores repetidos sobre 264 pares distintos. Hasta 24 scores cerrados adicionales únicamente tras puerta y selección, no autorizados por un resultado parcial.

CLI temporal propia sin puerto; ningún HTTP/API, dependencia, instalación, descarga de pesos o servicio nuevo. No tocar DB, secretos, env, Rust/Cargo, procesos ajenos, rama, commit o push. API USD 0; coste económico local y Codex desconocidos. Artefactos separados e inmutables en `outputs/iromem/evaluation/utility-learning-v4-2026-10-22/{rows,queries}/`, con comparación y auditoría independiente posteriores. El objetivo original sigue activo y no se anticipa éxito.
