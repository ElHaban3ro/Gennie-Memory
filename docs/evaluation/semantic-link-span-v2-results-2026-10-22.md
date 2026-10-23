# Vínculo semántico V2: referencias válidas, fidelidad insuficiente

Fecha: 2026-10-22. Paso 3 activo. [Protocolo](semantic-link-span-v2-protocol-2026-10-22.md). La versión produce once fichas estructuralmente válidas, pero se detiene en la respuesta doce por contradicción entre categorías y decisión. **No completó el piloto y no es candidata para adopción.**

## Resultado y alcance

| Medición | Observado |
| --- | ---: |
| HTTP intentados / recibidos | 12 / 12 |
| Juicios aceptados | 11 |
| Selecciones correctas en esos once | 11 |
| Útiles preservados en esos once | 6 / 6 |
| Ajenos en esos once | 0 |
| Nuevas pérdidas / ajenos frente al padre en esos once | 0 / 0 |
| Sin juicio aceptado | 421 |

Son once filas de un prefijo dirigido de TRAIN, interrumpido por una salida inválida; no equivalen a un porcentaje sobre 24, 432 o datos nuevos. El piloto 24, filtro completo TRAIN, puerta original 24/24+64/64 y preservación de seis conjuntos quedan no medidos. La respuesta doce no se transforma retrospectivamente en un acierto o abstención aceptados. Los 420 pares posteriores no se intentaron.

## Qué mejoró en el transporte

El modelo elige referencias a tokens canónicos de pregunta y documento; no recopía citas ni offsets. El adaptador recompone slices UTF-8 originales, incluyendo separadores, y separa grupos no contiguos. El foco solo refiere a la pregunta. En las doce respuestas, las referencias existen, pertenecen a su fuente y conservan orden e integridad. Eso resuelve el defecto observado de recopia en V1, sin demostrar que los fragmentos seleccionados sean suficientes o pertinentes.

Se comparten categorías, recibos y un solo harness V1/V2 con perfiles y directorios explícitos. V1, sus artefactos y su resultado siguen intactos; no se reejecutó. La interfaz, instrucción y composición cambian juntas: no es una ablación pura de capacidad del modelo.

## Por qué no basta con once selecciones correctas

La revisión separada de fuente examinó las doce respuestas antes de consultar la auditoría/gold de esta versión: **3 fieles, 6 no sustentadas y 3 con anclajes ambiguos**. Es otro agente de la misma familia con contexto previo del proyecto; no se presenta como revisión humana o independencia total de historia.

En varias fichas se llama record_absence al silencio del texto sobre el atributo pedido. Otra interpreta como diferente el mismo proyector Mirlo porque el documento habla de una magnitud distinta. Los anclajes de algunas fichas omiten negación o recortan una cantidad compuesta. Citas literales válidas y una decisión binaria coincidente con gold no validan esas interpretaciones.

En la respuesta doce, la pregunta pide consumo nominal y la fuente indica tensión de funcionamiento. El modelo declara referent_link=unresolved, focus_link=documentary_limit y contribution=record_absence, pero decision=exclude. La regla congelada exigía indeterminate con esas categorías; el decoder produjo inconsistent_decision y detuvo el intento. Además, la fuente sí nombra el mismo proyector y no afirma que el consumo esté ausente del registro: hay defectos semánticos que derivar una decisión local no corregiría.

## Verificación y coste

903 pruebas predeterminadas y 905 HTTP2 pasan, cero fallos; 24/77 ignoradas. Formato correcto. La sonda manual terminó exit101 por parada bajo protocolo. Preflight independiente 28.657/28.657 antes de inferencia; auditoría offline 29.362/29.362 reconstruye 26 eventos, 12 recibos, composición de referencias y el prefijo de once juicios. Son comprobaciones de integridad, no tasas de calidad semántica.

| Recurso | Observado |
| --- | ---: |
| Tokens entrada / salida del clasificador | 7.004 / 1.563 |
| Razonamiento reportado | 0 tokens |
| Coste externo conocido | USD 0,0013256 |
| Nuevas operaciones con coste desconocido | 0 |
| HTTP acumulado de 12 respuestas | 11.978,304 ms |
| Mediana HTTP descriptiva de ese prefijo | 953,169 ms |
| Fase completa del runner | 15,418965 s |

Tokens del clasificador, no de Codex. La mediana es descriptiva del prefijo, no p50/p95 de producto ni pase del límite 1,5×. Coste económico Codex/local desconocido. Presupuesto previo USD 0,50; estimación completa USD 0,123427725 y cota conservadora USD 0,4162965. No se gastó esa estimación: se conservan únicamente los doce recibos. Acumulado: 6.168 operaciones externas, 23 campañas emparejadas completas, USD 4,567807828 conocidos más 24 operaciones sin importe.

La documentación oficial advierte que uniqueItems y longitudes de strings no son garantía de constrained decoding; el decoder local comprueba referencias, unicidad, orden, origen y contenido. [Google: structured outputs](https://ai.google.dev/gemini-api/docs/structured-output?hl=en). Esa documentación no prueba la traducción exacta del intermediario ni fidelidad semántica.

## Continuación

V2 elimina la recopia de citas y acepta11fichas con selecciones correctas en ese prefijo, pero se detiene por decisión contradictoria en la respuesta12. La revisión de fuente encuentra solo3/12fichas fieles,6no sustentadas y3con anclajes ambiguos: corrección binaria no equivale a una interpretación fiel. La siguiente candidata conservará referencias canónicas y quitará decision del output remoto; la política local derivará la decisión de las mismas categorías y marcará su origen. Añadirá tres distinciones generales antes de inferir: identidad separada de magnitud, ausencia documental solo si está afirmada y no inferida por silencio, y citas que conserven cláusulas completas/negaciones/cantidades. Eliminar la redundancia no arregla por sí solo esas categorías falsas: la nueva versión deberá demostrarlo con las mismas puertas. V2 ysu fallo quedan intactos, sin convertir respuesta12 en un juicio aceptado.

Producto, DB, secretos, dependencias, dashboard, rama y README/gráficas sin cambios. No hubo servicios/puertos, commit o push. PID54780/session12461 observados en ejecución y después terminales exit101, sin reinicios. Reservas originales/V6 cerradas. Objetivo íntegro activo; calidad, recursos, estabilidad, casos nuevos y adopción permanecen pendientes.

Artefactos: outputs/iromem/evaluation/semantic-link-span-v2-2026-10-22/. Se conservan fichas, cuerpos, diario, auditoría y revisiones independientes.
