# Aportación textual directa V4: mejora del piloto sin preservación posterior

Fecha: 2026-10-22. Paso 3 activo. [Protocolo](semantic-link-direct-v4-protocol-2026-10-22.md). V4 supera el criterio de preservación del piloto, pero añade pérdidas y falsos positivos al ampliar los casos. La petición 127 termina en HTTP429. **No es candidata para adopción: la interrupción técnica no explica ni elimina los daños semánticos ya observados.**

## Comparación sobre los mismos pares

| Piloto fijo de 24 pares | Padre fijo | V3 | V4 |
| --- | ---: | ---: | ---: |
| Decisiones correctas | 18/24 | 16/24 | 22/24 |
| Útiles conservados | 12/12 | 12/12 | 12/12 |
| Ajenos seleccionados | 6/12 | 8/12 | 2/12 |
| Nuevas pérdidas útiles frente al padre | — | 0 | 0 |
| Nuevos ajenos frente al padre | — | 2 | 0 |

V4 supera la puerta prospectiva del piloto: cero daños nuevos. Esa puerta permite ampliar; no exige ni certifica perfección. Se ejecuta el prefijo una sola vez.

| Prefijo ampliado de 126 pares con juicio válido | Padre fijo | V4 |
| --- | ---: | ---: |
| Decisiones correctas | 106/126 | 101/126 |
| Útiles conservados | 51/53 | 46/53 |
| Ajenos seleccionados | 18/73 | 18/73 |
| Nuevas pérdidas útiles frente al padre | — | 6 |
| Nuevos ajenos frente al padre | — | 9 |

El mismo total de 18 ajenos oculta cambios: V4 elimina nueve errores anteriores y añade nueve distintos. Recupera un útil omitido por el padre, pero pierde seis que este conservaba. No hay ganancia C nueva en el prefijo. No se extrapola a 432 ni a producto. Los 306 pares sin juicio incluyen la petición fallida y 305 posteriores sin intentar; no son negativos implícitos. Todo el prefijo pertenece a background; pool no se alcanzó. TRAIN completo, puertas originales 24/24+64/64, seis conjuntos y generalización quedan no medidos. Los daños observados ya incumplen la preservación exigida.

## Qué se probó y qué falló

Una sola llamada por par al mismo Flash Lite devuelve cuatro campos: review_id, judgment, document_evidence_ids y rationale. Se retiran categorías intermedias y mapa de pregunta; el máximo de salida pasa a512. El documento y la pregunta siguen íntegros. Se conserva raw_card, decision_origin=model_judgment y semantic_fidelity_verified=false. V1–V3 mantienen su comportamiento y techo1024. Cambian varias partes del paquete; no se atribuye causalmente la mejora del piloto a una sola.

El modelo descarta algunas correcciones explícitas porque no obtiene la respuesta completa, pese a reconocer la corrección en su justificación. También omite un límite de medición pertinente. En sentido contrario, toma encabezados, casillas, opcionalidad o uso del mismo formulario como evidencia del dato concreto consignado. No se arregla con integridad JSON, referencias literales o una regla local sobre categorías. Se mantienen datos, gold, respuestas y fallos. El diagnóstico completo de los 25 errores está en errors-root-diagnostic.json.

La revisión separada de fuente cubre **solo el prefijo fijo24**, antes de abrir métricas/gold de esta versión:16 fieles,3 con afirmaciones no sustentadas y5 con citas incompletas. Las referencias y procedencia son exactas24/24, pero eso no acredita suficiencia semántica. Entre los defectos hay transferencias futuro→pasado y omisiones de restricciones. Es el mismo agente/familia con historia del proyecto; no es revisión humana ni independencia completa. Los otros102 juicios no recibieron esa revisión; no se atribuye fidelidad a126 o432.

## Interrupción y coste

El registro del error127 conserva su JSON analizado y declara lectura exitosa sin truncamiento; no es una captura de bytes de red. OpenRouter identifica limit_source=upstream_provider_shared_pool y Google AI Studio informa que el modelo está temporalmente limitado aguas arriba. No es evidencia de agotamiento del límite mensual de la clave. No se reintenta, cambia proveedor o reinicia la versión. La operación fallida no tiene recibo de coste; su importe queda desconocido, nunca cero.

| Recurso | Observado |
| --- | ---: |
| Peticiones intentadas | 127 |
| Respuestas exitosas / juicios válidos | 126 / 126 |
| Respuesta HTTP429 sin recibo de coste | 1 |
| Tokens de entrada / salida facturados en recibos completos | 55.471 / 15.682 |
| Razonamiento reportado | 0 tokens |
| Coste externo conocido | USD0,0118199 |
| Operaciones nuevas con coste desconocido | 1 |
| Tiempo HTTP acumulado de las126 respuestas exitosas | 127.250,516 ms |
| Mediana HTTP descriptiva de ese prefijo | 1.003,776 ms |
| Duración de fase del runner | 146,917486 s |

Son tokens del clasificador, no de Codex. La mediana no es un p50/p95 del producto ni pase del límite1,5×. Coste económico Codex/local desconocido. Estimación previa completa USD0,073078375, cota de ingeniería USD0,2646655 y presupuesto USD0,50; no son gasto observado. Acumulado:6.319 operaciones,23 campañas emparejadas completas,USD4,582739128 conocidos y25 operaciones históricas sin importe. Las dos fases nuevas149–150 suman151 intentos,USD0,0149313 conocidos y1 coste desconocido.

928 pruebas predeterminadas y930 HTTP2 pasan, cero fallos;24/79 ignoradas. Formato correcto. Preflight26.510/26.510 antes de inferencia; auditoría offline29.385/29.385 reconstruye256 eventos,126 juicios, recibos, coste y parada. Los cuerpos exitosos se guardan como JSON canónico con el content del modelo preservado, no como bytes HTTP originales. Las pruebas de integridad no acreditan calidad. La sonda manual termina exit101, con PID65485/session88505 observado vivo y después ausente. Sin reintentos o procesos ajenos detenidos.

## Próximo paso y avance del objetivo

La candidata directa mejora el piloto de 16/24 a 22/24, pero no sostiene la preservación fuera de él: en 126 juicios conserva 46/53 útiles y selecciona 18 ajenos, con 6 pérdidas y 9 ajenos nuevos frente al padre. La parada HTTP429 pertenece al pool compartido del proveedor; no explica esos errores semánticos y no justifica repetir la versión. No se cambia gold ni se abre V6. La siguiente investigación usará el padre local 0,6B congelado para extraer su representación final y aprender solo una cabeza logística de utilidad. Antes de cualquier medida se fijarán la agrupación por familia/pares, receta única y umbral; normalización y ajuste se harán solo dentro de cada fold de entrenamiento. La evaluación fuera de muestra agrupada deberá ganar utilidad C sin pérdidas ni extras nuevos; el ajuste sobre TRAIN completo no será aprobación. No se generan respuestas JSON ni se vuelve a entrenar el transformer.

El padre ya se entrenó con background176: separar folds de la cabeza no elimina esa exposición del encoder. Esas filas son anclas; solo se considerará OOF estricto para grupos de pool256 sin vínculo con familias/plantillas de sus ancestros. La correspondencia de grupos debe verificarse en las fuentes antes de extraer features; si no hay soporte independiente suficiente, se resolverá el banco antes de medir, sin rebajar la puerta.

Se reutilizarán activos locales y se verificará que la representación corresponda al padre congelado. La factibilidad por API estática no demuestra ejecución o separabilidad. Si la sonda falla, no se presentará el ajuste de entrenamiento como mejora ni se moverán sus umbrales después de observar resultados. Un negativo limita esta lectura lineal, no demuestra incapacidad global del modelo.

Hay dos de cinco etapas cerradas: contrato y banco. La tercera tiene implementación y experimentos, pero no validación de calidad suficiente. La cuarta y quinta siguen pendientes de sus puertas. El40% describe etapas cerradas, sin ponderar su esfuerzo; no es porcentaje de código, rendimiento o tiempo restante. El45% orientativo anterior no era comparable y no debe usarse como serie de progreso.

Sin adopción, cambios de política/default, DB, secretos, dependencias, servicios/puertos, dashboard, rama, commit o push. Reservas originales/V6 cerradas. No se publican gráficas de mejora de producto. El objetivo permanece activo y no está bloqueado por permisos.

Artefactos: outputs/iromem/evaluation/semantic-link-direct-v4-2026-10-22/.
