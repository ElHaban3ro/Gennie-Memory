# Perfil de embeddings local: plan y puertas

Fecha: 2026-10-24. Etapa M2. Estado: completado para el perfil local v1.

1. Comparar dos perfiles independientes: multilingual-e5-small y paraphrase-multilingual-MiniLM-L12-v2. Revisiones y artefactos fijados; runtime Candle 0.11.0 CPU f32 y tokenizers 0.23.2, optativos con `local-embeddings`. Conservar licencias, hashes, configuración, prefijos y procedimiento.
2. Fragmentar original y síntesis por tokens reales, conservando rangos UTF-8, texto y hashes. Desactivar truncamiento/padding implícitos. Reservar tokens especiales y prefijos; rechazar consultas que excedan el límite. Verificar textos largos, multilingües y texto normalizado.
3. Publicar índices de fragmentos únicamente completos para un manifiesto de memorias/revisiones y perfil. Inferencia fuera de transacciones; revalidar texto/revisión al publicar. Los índices obsoletos o parciales no se activan; los espacios históricos se preservan. Mantener búsqueda filtrada y coseno exacto, cotas/plazo con fallo cerrado.
4. Medir calidad de recuperación y latencia en banco sintético nuevo congelado antes de inferencia, con FTS5 como control y desglose de consultas con vacío léxico. Candidata: E5 si cumple al menos 90% top-5, no pierde top-5 frente al control MiniLM y completa textos largos; en empate priorizar contexto máximo. No atribuir verdad de respuesta a recall de candidatos. Objetivos locales: p95 caliente query+search <=1000ms y coseno de 10.000 vectores <=250ms; medir límites mayores sin adoptar ANN por defecto.
5. Probar migración, perfil alterado, dimensión/valores inválidos, revisión antigua, conjunto incompleto, plazo y modelo no disponible. Validar CLI/JSONL y repetir suite offline con/sin feature. Documentar todas las puertas y resultados antes de cerrar M2.

Las pruebas autorizadas usan DB temporal y datos de prueba; sin inferencia remota ni modificación de base personal. Se descargarán unos 950 MB de pesos en `.local/embedding-models/`, además de dependencias Rust fijadas, sin ejecutar código de los repositorios de modelos. El banco es del mismo autor, no validación humana o poblacional. La cola persistente y las políticas semánticas de recall quedan en sus etapas propias.

Cierre: perfil E5 seleccionado según la puerta previa; ambas corridas semánticas conservan 25/25 top-5. La corrección de cota RRF y su repetición V2 están registradas, con V1 preservada. Suite con/sin feature, controles CLI y límites medidos en las [cuatro páginas de verificación](../evaluation/local-embeddings/README.md).
