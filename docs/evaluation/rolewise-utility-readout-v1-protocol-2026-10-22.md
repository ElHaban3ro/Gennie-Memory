# Unión de tres lectores de utilidad: ensayo único de capacidad

Paso 3 del objetivo; ensayo prospectivo, fuera del producto. El diagnóstico154 mostró que un lector anterior resuelve16/18contrastes con el mismo encoder que los lectores ampliados8/18y7/18. La ampliación puede perjudicar una relación que esos vectores permiten representar. No se ha demostrado que la mezcla de roles cause el deterioro. Una propuesta de ajustar solo interceptos se rechazó sin entrenarla: dos inversiones por componente hacen imposible preservar simultáneamente todos los útiles/ajenos exigidos con cualquier offset.

Se elige una sola hipótesis exploratoria: una unión de tres fronteras lineales supervisadas por A, B y C puede mejorar la utilidad frente a una frontera binaria única. Cambia conjuntamente capacidad, objetivos y desequilibrio de clases; una eventual ganancia no aislaría causalidad de roles. No se ejecuta a la vez otra candidata para elegir después la mejor.

## Datos y receta fijos

Mismos560pares y features1024:432originales observados y128TRAIN nuevos ya revisados152. No nuevos ejemplos, encoder, instrucciones, tokenización, embeddings o preguntas. Etiquetas/roles originales inmutables, incluida ausencia documental A. Mismo mapa de15componentes, cinco con targets originales312/72/32/8/8. Cada fold ajusta solo el TRAIN ampliado fuera de su componente completo y evalúa sus filas originales. Los nuevos homólogos se excluyen con el componente; roles gold son objetivos de TRAIN, nunca entradas del predictor o selectores runtime.

Una media/escala compartida por fold, calculada sobre su TRAIN ampliado y exacta a152. Tres ajustes independientes one-vs-rest: A frente aB/C/none; B frente aA/C/none; C frente aA/B/none. Todos los15fits tienen positivos y negativos; mínimos de soporteA16,B33,C26, sin presumir suficiencia. Importar readout151sin modificar: FP64CPU, CE binaria media+0,01/2·||w||² incluido sesgo, desde cero, Newton/Woodbury máximo50pasos, gradiente infinito≤1e-7, Armijo1e-4 y máximo30reducciones. Sin pesos de clase, normalización por rol, λ alternativo, warmstart, barrido o rescate de checkpoint.

Inferencia: aplicar las tres cabezas a toda feature y seleccionar si max(sigmoidA,sigmoidB,sigmoidC)≥0,5. Esto equivale a OR de regiones, no a sumar probabilidades ni a una probabilidad calibrada de utilidad. Riesgos explícitos: útiles de un rol son negativos de los demás; roles minoritarios pueden omitirse y OR puede acumular falsos positivos. No se cambia la regla después de los resultados.

## Ejecución y verificación

CLI local CPU sin puerto, estimada menos de1minuto y límite120s. Quince fits reales como máximo750pasos; cero encoder forwards/backwards/updates y cero API/USD0. Reutilizar features560por hashes/fingerprint/orden; no volver a contarlas como inferencias. CosteCodex,energía/amortización desconocidos. TiempoCPU no mide latencia del recorrido productivo.

Controles sintéticos antes de datos reales verifican exclusión de componentes, normalización TRAIN compartida, aislamiento de heldout, runtime sin etiquetas, OR frente a suma y ausencia de rol rechazada antes de ajustar. Preflight independiente anterior a ajustes liga plan/código/receta/runtime/datos/lectores por hash. Marcador exclusivo,diariofsync,sin sobreescritura o reintentos de ejecución incierta. Auditor independiente posterior reconstruye15objetivos/gradientes/probabilidades/OR/métricas con pesos persistidos, sin refit o encoder. Preservar resultados negativos y fallos de verificadores separados del experimento.

## Comparadores y puertas

Comparar todos los432originales con padre histórico374/432,control152343/432 y binario ampliado152328/432. Este último tiene mismos datos/features/folds/normalización; no es comparación del mismo objetivo o complejidad. Reportar aciertos,útiles conservados,ajenos,gananciasC y cada pérdida útil/ajeno nuevo frente al padre y ambos controles. DesglosarA/B/C/none y componentes; no compensar daños con ganancias de agregado ni buen fit TRAIN.

Filtro preliminar inalterado: al menos una gananciaC frente al padre y cero pérdidas útiles nuevas o ajenos nuevos. Pasarlo no cierra el paso3: siguen24/24desarrollo,64/64conocidos,seis conjuntos conservados,recursos coste/p50/p95≤1,5×base,recibos,estabilidad yV6ciega. Reservas y contenido cerrado no se abren si falla. Los18contrastes154 no se emplean para ajustar/elegir esta arquitectura ni se reevalúan en este ensayo; su diagnóstico anterior no es validación fresca. No cabeza final de despliegue ni cambio del producto tras fallo. Objetivo completo vigente.
