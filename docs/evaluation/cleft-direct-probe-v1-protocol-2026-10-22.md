# Contraste controlado: construcción escindida frente a directa

Fecha:2026-10-22. Diagnóstico del paso3 tras fallar ambos brazos V6. El ajuste diverso selecciona sus ocho nominales útiles con «quien ... fue» y pierde uno de dos nominales observados con «lo ... [agente]». No demuestra causalidad: el puente se conserva y cambian léxico/personas. Esta sonda mantiene cada texto factual y cambia únicamente la construcción indicada, para identificar sensibilidad local; no es otro entrenamiento ni validación nueva del paso4.

## Material y revisión anterior a modelos

Se toman SOLO los16 nominales de los32TRAIN nuevos f17: ocho útiles B y ocho controles de otro objeto. Para cada uno se conserva el texto original escindido y se crea una versión directa: «quien lo/la [verbo] fue [persona]»→«lo/la [verbo] [persona]». Query, título, personas, verbo, negación y prefijo de corrección se mantienen byte a byte; no se introducen alias o fechas.32celdas=8escenarios×2construcciones×2identidades,16útiles/16ajenos. Controles de identidad conservan las mismas personas/evento, con otro título.

El reemplazo acotado solo genera fixtures de evaluación a partir de un patrón ya presente y no es un normalizador o mecanismo del producto. Guardar originales/IDs/transformación reproducible. No derivar casos del banco88, la sonda8 o reservas. Un revisor nuevo solo ve preguntas/documentos y rúbrica, sin etiquetas del autor ni scores; antes de congelar exige32/32concordantes y16equivalencias pertinentes sin ambigüedad. Conservar desacuerdos/cambios previos a resultados, sin arbitraje por predicciones. Independencia de contexto nativo, no humana/familiar.

La mitad original ya fue entrenada en V6diverse; las16directas son contrapartes del mismo material TRAIN. Ni32 ni16 equivalen a cohorte fresca o generalización independiente. Para reweight esos f17no se entrenaron. Se mide invariancia bajo una intervención textual sobre los mismos escenarios, sin inferencia poblacional.

## Estados y límites fijos

Una instancia GPU MLX, sin adaptar FP16 antes de insertar LoRA, seguida de padre V5matched y los finales V6reweight/V6diverse. Checkpoints exactos: padre7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74;reweight7eea01861727fecc3f338a08af09b228c4fd32009f1bc97f476fd8e6d81cf056;diversec7f7eeb216822c64b2dc5e75a92a77de55cae515127ac932f37cd9178ae3a952. Base Qwen3-Reranker-0.6B e61197ed45024b0ed8a2d74b80b4d909f1255473. LoRA32FP32arrays q/vúltimas8,r8scale8dropout0,baseFP16. Mismos instrucción/tokenizador/runtime/semilla/corte0,5, contexto completo, modeloeval/caché nueva por forward.

Orden sin adaptar→padre→reweight→diverse,32casos cada uno.128forward,0backward,0updates; no entrenamiento/cambio de pesos aparte de cargar estados exactos. Congelar protocolo/código/inputs/gold/tokens/checkpoints/refs/runtime y revisión antes del primer score. Preflight técnico nativo obligatorio antes de journal. Verificar pesos/archivos/base antes/después de cada estado, con fingerprints separados de base plain/envuelta.

Los16originales se verifican contra scores previos del mismo checkpoint en V6: padre inicial,finalreweight/finaldiverse; tolerancia0,02y mismas decisiones, con referencia por source_id. Sin referencia histórica para sin adaptar. Una discrepancia es fallo técnico y se conserva, sin reintentar en ese directorio.

## Lectura prospectiva

Para cada estado informar aciertos/32,útiles/16,ajenos/16,todas las decisiones/márgenes/scores. Para cada escenario e identidad: margen/score directo−escindido y cambio de selección. Resumir por útil/ajeno, y cuántos útiles se conservan solo en una forma. Guardar cambios por celda entre estados.

Si una pareja equivalente cambia de selección, hay sensibilidad a esta intervención en ese escenario y estado. Si solo varía margen, no es una pérdida de selección. Si se conservan ambas, no implica invariancia universal. Diferentes estados mantienen otros cambios de pesos; no atribuir un efecto puro a una causa de entrenamiento. No contar efectos agregados como nuevos aciertos del banco88, ni afirmar que corregir una construcción solucionará omisiones A/C. No buscar cortes, escalas o subgrupos tras resultados.

Puerta original24/24+64/64,cero pérdidas/ajenos/degradaciones y≥1ganancia sigue fallida y no se reevalúa aquí. Producto intacto, reservas24/cohorte finalV6cerradas. Diagnóstico no autoriza adopción ni paso4; siguen comparación productiva/calidad/recursos≤1,5×/recibos/estabilidad y evaluación final.

## Ejecución

CLI temporal sin puerto/API, estimado1–2min de modelo,watchdog600s. Registrar handle/PID/terminal real; inspeccionar el mismo proceso si se pierde observación, sin reiniciar por timeout. Sin descargas/dependencias/servicios/DB/secretos/procesos ajenos/ramas/commits. API nuevaUSD0,Codex/local económico desconocidos. Auditor offline reconstruye128scores/contrastes/refs/hashes/eventos y terminal; integridad separada de calidad y de reconstrucción independiente de pesos base finales no persistidos.
