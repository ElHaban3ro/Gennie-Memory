# Utilidad V3: datos ampliados frente a más actualizaciones

Fecha: 2026-10-22. Protocolo prospectivo, dos brazos finales predefinidos y seriales. V2 logró59/64conocidos, pero14/17útiles y2ajenos: falló. No se rescatan checkpoints ni se ajustan margen/coeficiente/umbral. La expansión128fue redactada, revisada ciegamente sin cuotas, conciliada y congelada antes de puntuarse.

## Pregunta y variables

Comparar la distribución ampliada con la original usando **1056pasadas con backward y264actualizaciones por brazo**. Control `original`:48train×22épocas. Candidata `expanded`:176train×6épocas, concatenando los48originales y128nuevos, cada fila una vez por época sin ponderaciones o filtros. Conjunto ampliado:72útiles/104ajenos; original24/24. Los cambios de prevalencia, repetición y orden forman parte de la distribución ampliada: este contraste distingue expansión de meramente aumentar actualizaciones, pero no aísla cada aspecto de la expansión.

Ambos parten separadamente de los pesos base originales Qwen3-Reranker-0.6B/revisión e61197ed45024b0ed8a2d74b80b4d909f1255473, nunca de un adaptador anterior. BaseFP16congelada, LoRAFP32q/vúltimas8capas/rank8/scale8/dropout0,327680parámetros/32tensores. SemillaMLX20261022justoantes de insertar adaptadores; exigir hash inicial igual aV1/V2y ambos brazos. NumPy default_rng/PCG64semilla20261022, todas las permutaciones de AMBOS brazos, código compartido y HP específicos congelados en plan global antes del primer score del primer brazo. Para original, las primeras seis deben coincidir conV2; expandedpermutaciones176íntegras, no mismo orden que original porque difieren índices.

PérdidaV2sin cambios: CE FP32sobre no/yes+y·max(0,2−(logit_yes−logit_no))². Margen2,coeficiente1,corte0,5. AdamWlr0,0001,betas0,9/0,999,eps1e-8,weight_decay0,01,bias_correction=True; acumulación4(media antes de clipglobal1). Eager, sin K/Ventre ejemplos o compilación que capture pesos. No scheduler, parada por desarrollo, selección de época, promedio de pesos, semilla alternativa o búsqueda de hiperparámetros.

## Datos y separación

Hashes ligados a preparación: train48/development24del banco utility-learning-data-v1y128train-extension-final-v1conSHA b721e7a3aaa8e61c80e420363954e62c5c0b321003243f957e2fce82a1d3ade4. Evaluar los mismos cuatro splits en ambos brazos:train48,extension128,development24,known64. En expanded, train+extensionson entrenamiento; en original, extensionno recibe gradientes. No interpretar esos resultados como validación final independiente: son parte del diseño observado. Ni development,known,closedoV6se entrenan. Entradas solo query/document bajo instrucción/tokenización íntegraV1; IDs/roles/familias/etiquetas/razones fuera del modelo, target binario train solo a la pérdida. Contexto>32768falla sin truncar.

Closed24yV6continúan sin abrir. La separación de construcciones reservadas y deduplicación normalizada de la expansión fue comprobada por otro revisor sin divulgar contenido cerrado a raíz. Misma familia de agentes; no revisión humana, independencia de familia de modelos o generalización poblacional.

## Controles e instrumentación

Antes de actualizar cada brazo: puntuar264filas(48+128+24+64)sin entrenar y64con adaptadorB=0. Comparar los136scores anteriores contraV2con cero cambios yΔ≤0,02; los264scores de base deben ser compatibles entre brazos, igualtolerancia. Tokenids64conocidos exactos contraV1; mismos hashes de inputs de evaluación y assets. Comparar versiones/configuración y hashes de fuentes canónicos del runtime, no el JSON completo que contiene tiempos variables. Verificar cada entrada del manifiesto original.

Comprobar las diez fórmulas/derivadas escalares deV2antes de inferir; no cuentan como forward de transformer. Nombres/dtypes trainables exactos, B=0al inicio yBnozero/gradiente positivo al final, todos finitos. Hash íntegro de base antes/después, sourcefreeze de modelo/loader/tuner/optimizador y controles antes de scores. Registrar CE/penalty/margen/target por paso, media de pérdida por época, norma por update y UTC RFC3339milisegundos/ordinal en cada evento.

Guardar un solo checkpoint final por brazo con SHA/datos/órdenes/HP, registrar su evento ANTESde toda evaluación entrenada. No publicar checkpoints intermedios elegibles. Materializar pérdidas/gradientes/acumulación/estado para evitar tiempos lazy. No tener dos modelos o procesos de entrenamiento residentes: ejecutar original y confirmar su handle terminal antes de iniciar expanded. Los tiempos seriales no aíslan latencia causal; recursos del producto se medirían aparte tras calidad.

## Puertas y selección fijadas

Para cada brazo, puerta previa aclosed idéntica:24/24developmenty64/64known, cero pérdidas útiles/ajenos/degradaciones respecto al gold/control y al menos una ganancia. Todos los errores en train/extensiontambién se reportan, sin llamar validación a in-sample. No compensar pérdidas con porcentajes, top1o abstención artificial. Ningún resultado intermedio modifica la ejecución del segundo brazo salvo fallo técnico que impida continuar con seguridad.

La selección solo se permite cuando ambos terminaron completos y evaluables. Si alguno falla técnicamente, el contraste queda incompleto y no se abre reserva ni se convierte ese fallo en rechazo semántico. Cuando ambos sean completos: si expandedpasa, es el único elegido para la siguiente reserva; si expandedfalla y originalpasa, se elige original. Si ninguno pasa, conservar ambos y no abrir reservas. Si ambos pasan se mantiene preferencia expandedpredefinida, sin usar reserva para elegir. Abrir closed24solo una vez para el brazo elegido, con pesos/HPya congelados, exigir24/24y conservar fallo si ocurre; no cambiar de brazo para volver a probarla. V6no se abre aquí. Posteriormente siguen exigidos pools frescos sin útiles/con varios, comparación contra ruta vigente, calidad completa/recursos≤1,5×/recibos/estabilidad yV6antes de adoptar.

La base de este ensayo es el reranker sin entrenar; no equivale a comparar con la ruta productiva FTS5/Decisions. No actualizar defaults o READMEcomo si este piloto fuera rendimiento vigente.

## Presupuesto y artefactos

Estimación8–14minutos en total, límite duro1200sporbrazo(2400sseriales). Cada brazo:1056forward/backwardtrain,264updates,592scores evaluación(264base+64zero+264final),1648forwardstransformer. Total dos brazos:2112backward,528updates,1184scores,3296forwards; hasta24scoresclosedextra únicamente tras puerta y selección fijada. Ningún HTTP/API, instalación/dependencia/peso base nuevo. Electricidad/amortización/Codex desconocidos; APIUSD0no es coste económico total cero.

Un fallo técnico/NaN/Inf/OOM/hash/contexto/timeout detiene ese intento sin reintentar en el mismo directorio ni cambiar backend. Conservar intentos inciertos con su diario y no contarlos completos. El segundo brazo solo se inicia si controles/entorno son coherentes y el primero está terminal; la calidad fallida del primero no bloquea el contraste fijado. CLI temporal sin puerto, procesos/watchdogpropios, sin detener procesos ajenos ni tocarDB,secreto,env,Rust,Cargo,servicios,rama,commitopush.

Artefactos inmutables bajo `outputs/iromem/evaluation/utility-learning-v3-2026-10-22/{original,expanded}/`; protocolo/runner/preparación/órdenes antes de scores, controles, pérdidas, checkpoint, reportes. Comparación agregada y auditoría al terminar. [V2](utility-learning-v2-protocol-2026-10-22.md), [expansión congelada](utility-learning-expansion-v1-protocol-2026-10-22.md). El objetivo original permanece activo; este protocolo no anticipa un pase o adopción.
