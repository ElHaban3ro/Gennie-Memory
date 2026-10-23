# V8: preservación de margen correcto en TRAIN

Fecha: 2026-10-22. Paso 3 del objetivo activo. Nueva hipótesis exploratoria adaptativa tras [P3](utility-learning-v7-results-2026-10-22.md), cuyos dos brazos siguen rechazados. No rescata P1/P2, no cambia gold ni la puerta global. La selección difícil no se demostró superior: aquí se usa su TRAIN exacto como intervención fija, para cambiar únicamente la loss entre los dos nuevos brazos.

## Pregunta e intervención

¿Una penalización explícita por reducir el margen correcto del padre en TRAIN conserva evidencia útil y mejora calidad donde la loss anterior no lo hizo? Los daños compartidos de P3 motivan la hipótesis; no prueban conflicto de gradientes, olvido ni la causa específica de cada error.

Fijar antes del modelo dos brazos `control` → `preserve`, ambos desde V5matched SHA7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74. TRAIN idéntico a `utility-learning-v7-2026-10-22/hard/training-inputs.json`:224filas,96útiles/128none; fondo176 y suplemento48. Mismos tokens, orden, inicialización, runtime, optimizador y presupuesto. No seleccionar otra muestra por los resultados de V7 ni agregar ejemplos/paráfrasis de conocidos o sondas.

Sea `m=logit_yes−logit_no`, `y∈{0,1}` y `s=(2y−1)m`. La loss de control es la V2 existente:

`L0=CE(no,yes;y)+y·max(0,2−m)^2`.

Para cada fila TRAIN, medir en el estado inicial del control los dos logits reales fuera de autodiff: ese teacher canónico se comparte exactamente entre brazos. Preserve verifica su reconstrucción inicial contra el canónico (IDs/gold/role/texto/tokens idénticos, logits y margen con diferencia absoluta≤0,05, decisiones iguales), pero usa el teacher del control, no otro objetivo distinto. Fijar `mask=1` si su decisión `(score_yes≥0,5)` coincide con gold; en otro caso `mask=0`. Verificar concordancia entre signo de m y decisión por score; m=0 selecciona yes por≥0, igual que score≥0,5. Definir `t=min(8,max(0,(2y−1)m_parent))` y:

`L=L0+λ·mask·max(0,t−s)^2`.

λcontrol=0; λpreserve=1, sin normalizar por número de filas masked. Cap8 no acota loss ni gradiente para s→−∞; limita el objetivo de confianza a una escala finita; λ1 coincide con la unidad de la penalización ya existente. Son decisiones de diseño únicas, no estimaciones de óptimo ni ajustes a los 88 observados. No barrer coeficientes/caps/épocas/semillas/umbrales/checkpoints. `t` y `mask` quedan congelados por ID, gold, fuente y tokens antes de actualizar; no propagar gradiente al padre. La máscara excluye errores del padre, no los convierte en verdad. Una decisión correcta del padre no demuestra que su confianza sea calibrada; penalizar su reducción puede perjudicar aprendizaje y debe probarse.

Las filas TRAIN están contenidas exactamente en fondo176+pool256 de la evaluación inicial560. Reutilizar esos mismos forwards para obtener teachers: cero llamadas de teacher adicionales. **Solo224IDs TRAIN** pueden entrar en la loss. No usar conocidos88, sondas8/32, no seleccionados208 del pool ni reservas para teachers/restricciones. Validar texto, label, role y tokens para evitar correspondencias por ID incorrectas. Los scores/logits de otros conjuntos solo son observación/gate. Ambos brazos calculan y registran los mismos componentes; únicamente λ cambia. Registrar máscara, t, m y componente de preservación aun cuando λ=0.

## Presupuesto y verificación

Cada brazo: una época224forward/backward, media4gradientes y56AdamWupdates; lr1e-4,betas0,9/0,999,eps1e-8,wd0,01,biascorrectiontrue,clipglobal1. BaseFP16,LoRAFP32q/vúltimas8capas,r8scale8dropout0,327.680parámetros. Instrucción/tokenización/umbral0,5 idénticos. Optimizerfresh para ambos. Congelar código/plan/datos/orden/runtime y crítica antes de modelo; preflight técnico independiente previo a inferencia. No cambiar Cargo/producto.

Cada fase inicial/final: fondo176+pool256+desarrollo24+conocidos64+atribución8+escindida32=560. Total2240scores_eval+448trainforward/backward=2688forward,448backward,112updates. Objetivo escalar y derivada se comprueban analíticamente en targets0/1, máscara correcta/incorrecta, teacher0/cap/intermedio y márgenes a ambos lados; estas comprobaciones no son forwards/backwards del transformer. λ0 debe coincidir con la loss V2. Registro antes/después de cada operación, preservación de logits reales, teachers antes de TRAIN, checkpoint antes de final y fingerprint después del primer update para comparación descriptiva entre brazos, finitud, fingerprints base/LoRA, terminal real y auditoría offline separada. Persistir límites si faltan gradientes/estados completos. No afirmar reconstrucción de tensores ausentes.

Reproducir referencias del padre: decisiones idénticas y score≤0,02; comparar iniciales entre brazos. El control reproduce la receta V7hard, pero su nueva corrida no queda obligada a repetir una cifra final exacta: informar diferencias y límites de estabilidad, sin reintento. Completar ambos pese al fallo de calidad; parar ante fallo técnico, conservarlo y no reiniciar intento incierto.

## Decisión previa

1. Puerta original intacta:24/24desarrollo+64/64conocidos,ninguna pérdida útil/ajeno/degradación y≥1ganancia contra reranker sin adaptar histórico. Mostrar padre contemporáneo/control/preserve por separado; no es comparación contra ruta productiva FTS5/Decisions.
2. Preservación suplementaria en los seis conjuntos: cero nuevas pérdidas útiles aceptadas antes por el padre y cero nuevos none que antes rechazaba. Desglosar IDs/roles aunque el promedio mejore. No afirma que corregimos errores previos del padre.
3. Integridad obligatoria. Si ninguno pasa1+2, rechazar ambos. Si uno pasa, congelarlo solo para siguiente validación; si ambos pasan, preferir control. La ventaja preserve/control no equivale a pase. Estabilizar el padre sin corregir sus errores tampoco basta.
4. Antes de adopción siguen casos nuevos con revisión ciega, comparación con producto completo, recursos/recibos/p50/p95≤1,5×, estabilidad y V6final. Reservas original24 yV6 no se abren en V8. No validación fresca a partir de TRAIN/no seleccionados/conocidos ya observados.

## Procesos y coste

Dos CLI locales propios sin puerto, secuenciales, estimado6–8min conjuntamente, watchdog1200sporbrazo. Anunciar antes de ejecutar, registrar PID/handle/terminal, seguir el mismo handle sin reintentar por timeout de observación. No servicios/dependencias/descargas/DB/secretos/ramas/commits ni procesos ajenos. Cero API; coste Codex y económico local no cuantificados. Entrenamiento/forwards no son latencia ni coste de la ruta productiva.
