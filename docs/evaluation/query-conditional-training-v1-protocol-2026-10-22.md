# Supervisión condicionada a la consulta: contraste con repeticiones

Paso 3 del objetivo. Un único contraste prospectivo, fuera del producto, después de fallar el ensayo OVR3. Hipótesis: añadir consultas de utilidad opuesta sobre un documento fijo puede mejorar transferencia frente a repetir las consultas originales, bajo el lector binario fijo. No se afirma que el modelo haya usado un atajo documental: ya existen14documentos con preguntas de etiquetas opuestas en el fondo560. La ampliación128 sí aporta128documentos con una sola pregunta cada uno. No se ajustan arquitectura,λ o corte según este resultado.

## Selección y revisión anteriores al encoder

De cada una de16familias TRAIN128 del152, seleccionar el ID lexicográficamente mínimo útil y mínimo ajeno. Resultado32fuentes determinísticas,16de cada clase. No escoger por scores o fallos. Conservar cada documento,pregunta original y gold; redactar una sola pregunta distinta que cambie utilidad, sin inventar hechos de fuente.64filas:32originales+32contrapreguntas. Ningún dato de reserva/cerrado,ningún caso nuevo de evaluación del paso4.

Revisión separada recibió solo{id,query,document}opacos:64/64acuerdos binarios y de rol,citas literales verificadas. Borrador/final byteidénticos,sin cambios tras resultados. Misma familiaCodex e historia del proyecto, no revisión humana o independencia de familia. La crítica fuente separada verifica selección,semántica,tipos de cambio y posibles pistas antes de features. No hay pregunta genérica tipo«qué dice el texto»o petición de cita como atajo. Persisten diferencias de estilo y dificultad:«De qué»aparece en6/16contrapreguntas negativas y0/16positivas; no se declara balance léxico perfecto.

Todas las nuevas positivas son A. En las64filas añadidas,tratamiento tiene20A/6B/6C/32none ycontrol8A/12B/12C/32none. Se iguala utilidad binaria, no mezcla de roles. El contraste estima esta intervención de datos completa, no un efecto puro atribuible únicamente a condicionar por consulta. Los cambios pueden afectar atributo,referente,condición o premisa y no son una operación lingüística única común.

## Brazos y cierre de fuente

Fondo560intacto:432originales+128TRAIN152. Añadir64ocurrencias en cada brazo,624en total:

- repeat_control: dos repeticiones de la consulta original por cada fuente.
- conditional: consulta original y contrapregunta por esa misma fuente.

Misma frecuencia documental,mismos IDs/posiciones de ocurrencia,32útiles y32ajenos añadidos; por familia2útiles/2ajenos. Las dos fuentes de cada familia comparten componente, por lo que marginales binarias siguen iguales en cada fold. Etiquetas y metadatos no entran al encoder; solo consulta/documento íntegros con instrucción fija.

La crítica de nuevas consultas añade un puente concreto:conditional_0022 pide temperatura mientras el documento de SaviaClara informa iluminación; es homólogo del contraste de magnitud de cámara t_f04_01. Se conserva el testigo y cierre transitivo. Quedan592IDsdecontenido,14componentes,cuatro con targets320/72/32/8. Los dos brazos usan esa misma exclusión extendida completa. Las432filas originales permanecen; no se fuerza el mapa antiguo para conservar resultados.

Normalización común por fold, calculada solo con el fondo560permitido fuera del componente extendido:200/488/520/552filas. No recalcular por brazo ni con las64ocurrencias añadidas. Cada brazo ajusta su cabeza sobre todas sus ocurrencias624permitidas. Un componente test no aparece en features de normalización ni TRAIN. Los resultados152con cinco particiones se conservan como históricos descriptivos; el control emparejado contemporáneo es repeat_control con las cuatro particiones nuevas.

## Una ejecución y receta

Reutilizar560features por hash/orden y fingerprint del mismo Qwen3baseFP16sinLoRA; solo32contrapreguntas requieren nuevas features1024FP32. Helper de captura152 sin cambio salvo128→32:assets/runtime/tokenizador/instrucción,prefijos,textos completos,310castsBF16→FP16,fingerprints pre/post y proyecciónCPUde los mismos hidden. Sin nuevos pesos,instalaciones o secreto.

Cuatro folds×dos brazos=ocho fits CPU,máximo400pasos. Receta151sin modificación: normalizaciónFP64descrita,CE binaria media+0,01/2·||w||²incluido sesgo,desde cero,Newton/Woodbury50máximo,tolerancia de gradiente infinito1e-7,Armijo1e-4 y30reduccionesmáximo. Selecciónsigmoid≥0,5. Sin pesos de clase,rolesruntime,búsquedas de λ/corte,otros checkpoints o agregaciones. La candidata predeterminada es conditional; no atribuirle un pase del control si falla ni elegir brazo retrospectivamente.

CLI local sin puerto,estimada1–2minutos,límite180s:32forwards nuevos,cero backwards/updates del transformer,ocho fits. Marcador exclusivo y diariofsync,hashes antes/después,ningún reinicio tras observación incierta. Preflight separado previo liga código,datos,mapa,runtime,receta y máscaras; auditorCPU posterior reconstruye exclusiones,normalizaciones,targets,objetivo/gradiente,predicciones y métricas sin refit/encoder. Verificar que repetir originales usa realmente features cacheados y que solo la contrapregunta usa nueva feature.

## Métricas, coste y puertas intactas

Reportar432decisiones por brazo,útiles conservados,ajenos,gananciasC y todos los IDs de pérdida útil/ajeno nuevo frente al padre histórico fijo y el control contemporáneo. Desglosar roles/componentes,mostrar cambio de partición y sesgos del banco. No compensar daños con ganancias o un buen ajusteTRAIN. No reevalúa los18contrastes154para seleccionar variante. Cabeza final de despliegue no se ajusta tras fallo.

Filtro preliminar original:al menos una gananciaC frente al padre y cero pérdidas útiles nuevas/ajenos nuevos. Siguen24/24desarrollo,64/64conocidos,seis conjuntos conservados,recursos coste/p50/p95≤1,5×base,recibos,estabilidad yV6ciega. Pasar este contraste sobre fuentes conocidas no demuestra generalización fresca ni cierra el objetivo. Si falla,no abrir reservas o modificar producto.

API nuevaUSD0/cero llamadas; energía,amortización y Codex desconocidos. Registrar tiempo de captura y CPU,RSS/MLX separados; no presentarlos como latencia del recorrido productivo. Los560vectores cacheados no se cuentan como inferencias nuevas.
