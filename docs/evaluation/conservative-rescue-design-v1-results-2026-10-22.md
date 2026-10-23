# Preservación conservadora: diagnóstico y diseño

El resultado de calidad vigente sigue siendo el contraste158: interacción355/432 frente a raw336/432, rechazado por16pérdidas útiles y11ajenos nuevos frente al control. Este trabajo no añade una medición de calidad ni adopta un modelo. El objetivo sigue activo en el paso3, con2/5etapas cerradas.

## Qué referencia estamos preservando

El padre de los ensayos432 es el reranker experimentalV5: Qwen3-Reranker0.6B con LoRA, ya entrenado en176pares. No es la ruta productiva FTS5/Decisions. En la ruta real, recall consulta primero feedback revisado del dashboard; si no hay resolución, recupera candidatos y los clasifica mediante el adaptador OpenRouter antes de componer. No comparar métricas del padre como si midieran todo ese recorrido.

La unión S_final=S_base ∪ S_rescate preserva los IDs seleccionados por definición, incluidos sus errores previos. No garantiza la pertinencia de las adiciones ni la preservación de la semilla final si top-k, presupuesto, ordenación o composición expulsan evidencia. La política fallback ya existe históricamente: usar OR no constituye por sí solo una nueva mejora.

## Límite comprobado de las señales disponibles

Entre los13C útiles que el padre omite y los203ajenos que rechaza, cada C tiene al menos un ajeno con coordenadas mayores o iguales en las tres señales: margen del padre, probabilidad raw158 y probabilidad interaction158. Se conservaron los206pares dominantes completos, no solo un ejemplo porC.

Una combinación global no decreciente en cada una de esas tres coordenadas, con decisión por umbral inclusivo, no puede rescatar ninguno de esos13C sin seleccionar también un ajeno nuevo. Es un diagnóstico posterior sobre estos datos observados y estos scores, sin ajuste de cortes ni predictor. No excluye funciones no monótonas, features/contextos nuevos, otros modelos ni mejora en otros datos.

AuditoríaCPU separada aprobada:13/13C,203ajenos,206testigos y todas las coordenadas/hashes/listas exactas. Cero modelos, fits, nuevas APIs o cambio de etiquetas. No es un porcentaje de calidad ni validación fresca.

## Intervención distinta propuesta

Un verificador pequeño aprende relaciones entre tokens de pregunta y documento antes de pooling, usando la tabla de embeddings base ya disponible y congelada. No recibe los tres scores anteriores, IDs como features, roles gold, fichas manuales o reglas por caso. Se aplica solo tras una abstención; el baseline no se actualiza. Su salida es una predicción aprendida, no una prueba certificada de entailment.

232abstenciones disponibles:29útiles(2A,14B,13C) y203ajenos. El mapa de cuatro componentes tiene solo tres con abstenciones: test23(7útiles/16ajenos),49(14/35),160(8/152); sus TRAIN complementarios209(22/187),183(15/168),72(21/51) conservan ambas clases. El cuarto componente no tiene abstenciones: pasa por el padre sin fit ni supuesta validación del verificador. El soporte es pequeño y desigual; no acredita generalización.

Antes de entrenar: congelar una arquitectura, objetivo, semilla, orden, épocas/updates y corte0,5; verificar embeddings y máscaras, separar todo el componente fuente y conservar todos los resultados. Una sola prueba, sin barridos o rescate de checkpoints. El criterio mínimo sigue siendo≥1gananciaC, cero pérdidas útiles y cero ajenos nuevos frente al padre, más las puertas originales posteriores. Recursos adicionales de un verificador pequeño deben medirse; no asumir≤1,5×. El padre conserva29ajenos previos: la unión no satisface por sí sola ninguna puerta absoluta de cero ajenos.

No se ha ejecutado entrenamiento del verificador en esta fase. Coste API nuevoUSD0; coste Codex/local desconocido. Tampoco se abrió original88, reserva original, V6 o producto. La implementación de evaluación y su revisión estática no se presentan como calidad lograda. Las218fuentesRust no cambian; suites928default/930HTTP2heredadas de150, sin repetir. El cierre ejecuta fmt y conserva hashes y evidencia.
