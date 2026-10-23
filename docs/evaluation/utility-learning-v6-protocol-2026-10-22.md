# V6: diversidad de atribución frente a mayor exposición

Fecha: 2026-10-22. Experimento del paso 3. La sonda anterior encontró una refutación útil aceptada en pasiva y rechazada en forma nominal por el padre y la candidata, con otra refutación conservada en ambas formas. No demuestra ausencia de la familia, incapacidad global o mejora del producto. Esta prueba contrasta un paquete de diversidad contra aumentar la exposición a ejemplos antiguos. No se buscarán épocas, escalas, umbrales ni checkpoints por resultados.

## Datos y comparación

Ambos brazos parten exactamente del checkpoint V5 matched `7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74`. Se conserva el fondo TRAIN original de 176 pares (48 originales +128 de ampliación), una presentación por par. Se añaden 32 presentaciones en cada brazo:

- `reweight`: los ocho TRAIN f05 repetidos cuatro veces, cuatro útiles B y cuatro irrelevantes por repetición. Los textos y sus etiquetas se conservan.
- `diverse`: 32 pares nuevos, ocho escenarios × dos realizaciones de corrección × mismo/otro objeto. Dieciséis útiles B y dieciséis irrelevantes; las personas, objetos y escenarios concretos no proceden de los casos observados. Los controles negativos solo cambian el nombre del objeto respecto a su contraparte útil.

Un autor nuevo no ve bancos observados, sondas o reservas. Otro revisor recibe únicamente query/document y una rúbrica, sin etiquetas del autor o scores. Antes de congelar se exige concordancia completa o resolver las ambigüedades preservando originales; no arbitrar después de puntuación. Registrar equivalencia pertinente de formas y asimetrías accesorias. La independencia es de autoría/contexto nativo, no humana ni familiar. Las coincidencias exactas con TRAIN/observados se verifican; no afirmar ausencia de paráfrasis semánticas cuando el tipo de relación está compartido deliberadamente.

208 presentaciones por brazo: 88 útiles y120 irrelevantes. La misma permutación de índices, semilla Python `20261022`, y los mismos targets/roles en cada posición de ambos brazos. Los 176 del fondo son idénticos en las mismas posiciones. Cada presentación adicional nueva se empareja con un antiguo del mismo role/target; cada antiguo f05 aparece cuatro veces en el suplemento de control. Así se igualan peso de clase, presentaciones y updates; no se separa el efecto individual de léxico, sintaxis, nombres y emparejamiento dentro del paquete de diversidad.

## Intervención fija

Un solo entrenamiento por brazo, orden `reweight` y luego `diverse`. Desde el padre, **una época**, 208 forward/backward y52 updates por brazo. Optimizador AdamW nuevo con estados iniciales cero, lr1e-4, betas0,9/0,999, eps1e-8, weight_decay0,01, bias_correction true. Pérdida V2 sin cambios: CE + target × max(0,2−margen_yes_no)^2. Media de cuatro gradientes antes de clip global1; cada update consume cuatro posiciones. No checkpoint intermedio seleccionable, solo final.

Modelo base Qwen3-Reranker-0.6B, revisión congelada e61197ed45024b0ed8a2d74b80b4d909f1255473, FP16; LoRA FP32 en q/v de últimas ocho capas, rank8, scale8, dropout0;32 arrays y327680 parámetros. GPU MLX, misma instrucción/tokenización, contexto completo≤32768 y corte0,5. Semilla20261022. No adaptar base ni datos de evaluación. El reinicio del optimizador es idéntico en ambos brazos y no representa continuación de su estado histórico AdamW.

Congelar protocolo, código, conjuntos, checkpoint inicial, órdenes, hiperparámetros, tokens, runtime/API y revisiones antes de inferencia. Preflight técnico independiente obligatorio antes del primer journal. Registrar por cada backward el ID, target, posición, pérdida/CE/penalización/margen, norma en updates, estados finitos y número de updates. Verificar fingerprint del padre inicial idéntico, checkpoint final persistido antes de evaluación final y base inmutable. Entrenamiento no compilado, sin caché de atención entre pares.

## Evaluación y puertas intactas

Antes y después de entrenar, evaluar los mismos 304 pares en cada brazo: fondo176, ampliación nueva32, desarrollo24, conocidos64 y sonda observada8. Son1216 scores de evaluación más416 forward/backward de entrenamiento y104 updates entre ambos brazos. Medir todo aunque falle calidad, salvo fallo técnico. El suplemento nuevo está entrenado en `diverse`; su resultado es fit, mientras en `reweight` no fue entrenado: ese contraste no prueba transferencia poblacional. La sonda8, desarrollo y conocidos son observados y nunca entran al entrenamiento nuevo.

Verificar que las predicciones iniciales en desarrollo/conocidos/sonda concuerdan con el mismo padre anterior (mismas decisiones y diferencia máxima de score0,02); si no, detener antes de entrenar y conservar fallo técnico. Ambos estados iniciales y todas sus evaluaciones se contrastan entre brazos, sin exigir igualdad bit a bit cuando la ejecución de GPU no la garantiza.

Puerta original: **24/24 desarrollo y64/64 conocidos**, cero pérdidas útiles, ajenos o degradaciones frente a la base sin adaptar, y al menos una ganancia. Usar las referencias sin adaptar históricas de los mismos casos, declaradas como tales. Comparar además frente al padre contemporáneo y entre los dos brazos. Informar pérdidas y ajenos por separado y no sustituir el criterio por ranking, top1 o porcentaje agregado. Un eventual pase solo autoriza preparar la siguiente validación separada, no adopción.

Preservar fuentes y resultados negativos. No entrenar los88 observados, los ocho de la sonda o reservas; no cambiar etiquetas tras resultados. Reserva original24 y cohorte final V6 cerradas (el nombre V6 de este entrenamiento no es esa cohorte). Paso4 nuevo con revisión ciega y comparación productiva completa, recursos/recibos/estabilidad y evaluación final siguen pendientes hasta las puertas correspondientes.

## Ejecución y coste

Dos CLI locales secuenciales sin puerto ni API, estimados3–5 minutos de modelos una vez preparados; watchdog1200s por brazo. Identificar handle/PID y salida terminal real. No reintentar por pérdida de observación ni parar procesos ajenos. Sin dependencias, descargas, DB, secretos, servicios, ramas o cambios del producto. API nuevaUSD0; costes económico local y Codex no cuantificados. Los tiempos de entrenamiento/evaluación aquí no prueban coste/p50/p95 completos del producto≤1,5×. Auditoría offline independiente reconstruye órdenes, counters, métricas y puerta antes de concluir.
