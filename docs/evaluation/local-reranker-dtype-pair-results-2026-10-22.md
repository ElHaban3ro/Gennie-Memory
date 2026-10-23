# Precisión FP16: mejora ejecución, no calidad ni viabilidad suficiente

El contraste completo conserva las64decisiones originales en ambos brazos y reduce tiempo conFP16, pero **falla el filtro instrumental previo**. La calidad continúa fallida53/64,16/17útiles,1pérdida y10selecciones ajenas; el cambio numérico no corrige ninguno de esos errores. No se entrena, amplía o incorpora esta ejecución.

| Medida contemporánea | FP32 | FP16 |
| --- | ---: | ---: |
| Batch4conocido,16operaciones:p50 | 1730,51ms | 1360,74ms |
| Batch4conocido:p95 | 2469,93ms | 2367,66ms |
| Batch20neutral,8operaciones:p50 | 9283,22ms | 7064,47ms |
| Batch20neutral:p95 | 10812,68ms | 8110,05ms |
| Cambios de elección frenteV1 | 0 | 0 |
| Diferencia absoluta máxima de score frenteV1 | 0 | 0,00394619 |

FP16reduce aproximadamente23,90%la mediana y25,00%elp95delbatch20respecto al controlFP32contemporáneo. Aun así supera ampliamente la orientación830,07/1380ms. Esa orientación procede de una cohorte histórica y **no es el gate de recursos del producto**. La comparación de dtypes sí es contemporánea y usa los mismos datos/modelo.

Se verificaron las tres comparaciones de scoreFP32↔V1,FP16↔V1yFP32↔FP16, con tolerancia0,02fijada antes de medir. La máxima0,00394619y cero cambios permiten describir compatibilidad de ejecución en estos64pares; no equivalencia universal o de futuros pesos entrenados. El controlFP32reproduce exactamente los scores previos. La inversión Puerto>Vado sigue intacta y el umbral0,5no cambia.

Se ejecutaron48forwardbatches y448scores:32batch4con128scores conocidos —64pares repetidos en dos brazos, no128ejemplos nuevos— y16batch20con320scores neutrales sin gold. La ventana de inferencia fue179,177s. Todos los tiempos individuales y primeras inferencias se conservan; medianas y p95por rango próximo incluyen todas las muestras de cada fase. Las8repeticiones no acreditan estabilidad poblacional ni cola de producto.

Ambos modelos residieron simultáneamente en esta Mac. Se mantuvieron MPSsinCPUfallback, SDPA,2threadsCPU, mismos safetensors/tokenizer/textos sin truncamiento y scoringúltimotokenyes/noenFP32. El único cambio entre brazos fue dtype del modelo. Los tiempos no prueban latencia de un proceso standalone, otras distribuciones de documentos o MLX. El ensayo no resuelve el coste o velocidad de runtimes que no ejecutó. Tampoco mide búsqueda/recall/integración final.

La auditoría reconstruyó96eventos y todos los scores/selecciones/tokens, comprobó alternancia, conteos, medianas/percentiles, tres compatibilidades y gatefalse. El informe se publicó al completar ambas fases; no hubo fallo técnico, reintento, descarga, instalación o modelo nuevo. Revisión crítica nativa separada confirma la decisión estrecha: **no invertir entrenamiento amplio en esta ejecución**. Standalone sería una pregunta distinta, necesaria antes de descartarlo, no para respetar el filtro actual.

Coste APIinferenciasUSD0y0llamadas remotas nuevas; electricidad/amortización/Codex no cuantificados. Acumulado externo intacto6155operaciones,23campañas emparejadas completas,USD4,566379428conocidos+24importes desconocidos. Los scores locales no se suman a operaciones HTTP. No se repitió Cargo: se verificaron213hashes de implementación/Cargo idénticos a la fuente que pasó879pruebas default/881HTTP2; son pruebas heredadas del mismo código, no una suite ejecutada hoy. El nuevo script compiló y se ejecutó completo; no cambia producto,DB,secreto o rama.

El objetivo original sigue en paso3; no se abrenV6ni puertas posteriores. Una investigación de aprendizaje específico sigue siendo hipótesis: necesitaría datos nuevos y revisados, negativos de entidad/atributo/modalidad/parcial, splits por familia, una corrida predefinida y control sin entrenar. No entrenar los64conocidos oV6ni usar partición cerrada para elegir umbrales/checkpoint. No preparar ahora ese entrenamiento amplio mientras esta ejecución no cumple viabilidad. Antes de otra inversión se debe resolver una causa de ejecución concreta o justificar otra ruta que ataque discriminación y recursos, sin un desfile de instrucciones/modelos.

Evidencia: `outputs/iromem/evaluation/local-reranker-dtype-pair-v1-2026-10-22/` conserva protocolo/código antes de scores, inputs, neutral20, runtime, journal, informe y auditoría. [Protocolo prospectivo](local-reranker-dtype-pair-protocol-2026-10-22.md), [fallo semánticoV1](local-reranker-v1-results-2026-10-22.md).
