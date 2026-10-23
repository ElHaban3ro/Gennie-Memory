# Reranker local V1: falla al discriminar evidencia útil

El diagnóstico local completo **falla**:53/64decisiones binarias correctas,16/17evidencias útiles conservadas,1pérdida y10selecciones ajenas. No hay adopción, ampliación a difíciles, neutral20 ni aperturaV6. Estas son decisiones útil/noútil; no64rolespredichos. La retención observada no constituye una mejora comparada de producto.

| Relación esperada del fixture | Total | Seleccionadas |
| --- | ---: | ---: |
| A, aporte afirmativo/estado solicitado | 10 | 9 |
| B, refutación | 4 | 4 |
| C, contraste/parcial | 3 | 3 |
| Irrelevantes | 47 | 10 |

Se ejecutaron16forwardbatches locales de4pares,64pares y14877tokens sin padding. Modelo Qwen3-Reranker-0.6B revisión `e61197ed45024b0ed8a2d74b80b4d909f1255473`, pesos/verificaciónSHA, float32/MPS y runtime fijados, sin generación ni fallbackCPU. La instrucción final incluyó los contrastes útiles sin exclusividad tras crítica **anterior a scores**, preservando los3C. Texto original íntegro, sin truncar, máximo249tokensporpar. Umbral0,5congelado. No se ajusta tras observar el banco.

La auditoría reconstruyó las64decisiones desde el journal, comprobó inputs/IDs/revisiones y ausencia de gold en el texto enviado al modelo, reprodujo agregados y volvió a verificar todos los archivos del modelo. No hubo error técnico ni un defecto nuevo del gold que explique los11errores, según revisión crítica nativa separada. Esta revisión es de desarrollo conocido y no se presenta como validación ciega nueva.

## Qué aprendimos

El modelo acepta asociaciones sin aporte solicitado: teléfono de otro pedido, ventanilla/correa, credencial ante tutor, otra cámara sin contraste completo, exportación futura ante entrega realizada, frecuencia de ingreso ante retención y otro puente ante corrección de autoría. Omite la alimentación intermedia de Nardo. Esto demuestra discriminación insuficiente de entidad, atributo, modalidad y parcialidad; no identifica el proceso interno que la causó.

Hay una inversión decisiva en la misma consulta `cv3q_10`: la corrección útil sobre Vado/Leo obtiene0,9755463600 y la fuente irrelevante sobre Nerea/Puerto0,9912844896. Con estos scores fijos, ningún umbral global ni específico de esa pregunta que seleccione valores mayores puede conservar la útil y rechazar la ajena. Una transformación monótona creciente tampoco lo arregla. Es diagnóstico retrospectivo de estos scores, no un clasificador afinado ni imposibilidad universal de aprender la tarea. No corresponde rescatarV1con calibración, otra instrucción o un desfile de modelos.

## Tiempo, recursos y coste

| Medida local de4candidatos/operación | Resultado |
| --- | ---: |
| p50,16operaciones completas | 1814,32ms |
| p95, rango próximo, incluye primera operación | 4387,24ms |
| Primera operación | 4387,24ms |
| p50/p95 descriptivos sin la primera | 1768,19 /2622,25ms |
| Importación del runtime | 20,805s |
| Carga y preparación de tokenizer/modelo | 8,343s |
| Descarga de assets públicos, aparte | 24,677s |
| Pico RSS del proceso | 2229141504bytes |
| MPSdriver al registrar resultado | 2827698176bytes |
| Inferencias remotas | 0 |
| API de clasificación | USD0 |
| Coste económico completo/Codex | No cuantificado |

No sumar RSS y memoriaMPScomo si fueran independientes: esta Mac tiene memoria unificada. La ventana de inferencia fue32,306s; la carga fría y los imports no se esconden en la latencia caliente. La referencia histórica553,38/920ms no es control contemporáneo, y este batch4no acredita el gate del recorrido de20candidatos, búsqueda o respuesta final. No se ejecutó batch20porque falló calidad. APIUSD0no significa coste total cero.

## Decisión y alcance

V1queda descartada. No se incorpora dependencia de producto, runtimepolicy, motor Rust ni servicio. Se conservan assets en caché compartida y entorno uv aislado para reproducibilidad; los procesos y watchdogs propios terminaron. El evaluador Rust, baseline, DB del usuario, etiquetas y reserva permanecen iguales. No commit/push.

El objetivo original sigue en paso3. La siguiente investigación debe abordar discriminación semántica con evidencia y separar mejora de ejecución de mejora de calidad. El entrenamiento específico podría modificar el orden de pares —a diferencia del umbral—, pero requiere datos nuevos de entrenamiento, negativos difíciles, separación por familias y validación reservada, coste de preparación y backend viable; no está implementado ni prometido. No entrenar sobre el banco que luego se anuncie como independiente. La latenciafloat32observada también exige una hipótesis de viabilidad distinta antes de inversión amplia. No reabrir la gramática documental64/116fallida ni la fragmentación no justificada para compensar estos errores.

Evidencia: `outputs/iromem/evaluation/local-reranker-v1-2026-10-22/` (protocolo/script/inputs antes de scores, runtime, assets/hashes, journal, resultados y auditoría), [protocolo](local-reranker-v1-protocol-2026-10-22.md), [dependencias y límites](../decisions/0030-local-reranker-feasibility.md). El snapshot127conserva también la revisión crítica y el intento remoto429separado.
