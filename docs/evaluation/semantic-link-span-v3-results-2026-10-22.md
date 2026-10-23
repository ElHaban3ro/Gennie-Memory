# Vínculo semántico V3: piloto completado con regresión

Fecha: 2026-10-22. Paso 3 activo. [Protocolo](semantic-link-span-v3-protocol-2026-10-22.md). Las 24 respuestas del piloto son estructuralmente válidas, pero la candidata selecciona dos recuerdos ajenos que la referencia descartaba. **Falla la puerta de calidad y no se incorpora al producto.**

## Calidad observada

| Medición del mismo prefijo TRAIN | Referencia fija | V3 |
| --- | ---: | ---: |
| Decisiones correctas | 18/24 | 16/24 |
| Recuerdos útiles conservados | 12/12 | 12/12 |
| Recuerdos ajenos seleccionados | 6/12 | 8/12 |
| Nuevas pérdidas útiles | — | 0 |
| Nuevos ajenos | — | 2 |

V3 emite cuatro decisiones indeterminadas, todas sobre ajenos; ninguna cuenta como selección. Los 408 pares restantes no se intentan y no reciben una clasificación implícita. No hay resultado completo sobre 432 ni nueva medida sobre las puertas originales 24/24+64/64 o los seis conjuntos. La referencia es el padre histórico congelado del diagnóstico de contribución, no una ejecución contemporánea del producto.

## Cambio y causa del fallo

El modelo devuelve siete campos, sin una decisión redundante. La política local deriva include/exclude/indeterminate de las mismas categorías anteriores y conserva `decision_origin=local_policy`, `raw_card` y `semantic_fidelity_verified=false`. Esto permite completar el piloto sin la contradicción estructural que detuvo V2. No convierte categorías falsas en evidencia verdadera ni rescata aquel intento.

En seis de los ocho ajenos seleccionados, el modelo llama `record_absence` al silencio sobre el atributo pedido. Los dos errores nuevos cruzan mantenimiento y transporte: la propia justificación reconoce que la fuente no menciona el atributo solicitado, pero los campos lo presentan como pertinente y como ausencia registrada. La instrucción V3 prohibía expresamente esa inferencia; la regla local aplica correctamente categorías equivocadas. También se transfiere un plan futuro a un hecho pasado y se interpreta sin apoyo «qué día» como día de la semana. Se conservan textos, gold y respuestas originales.

La revisión de fuente de otro agente se realizó antes de abrir métricas/gold de esta versión, con cobertura y limitaciones en el artefacto correspondiente. Sus recuentos son `{"reviewed": 24, "faithful": 8, "unsupported": 16, "ambiguous": 0, "reference_integrity_passed": 24, "local_policy_coherence_passed": 24}`. Es un agente de la misma familia con historia del proyecto; no equivale a revisión humana ni independencia completa. Citas y referencias válidas solo acreditan procedencia.

## Integridad y recursos

915 pruebas predeterminadas y 917 con HTTP2 pasan, cero fallos; 24/78 ignoradas. Formato correcto. Preflight previo 28241/28241; auditoría offline 29165/29165. La auditoría reconstruye respuestas, siete campos originales, decisión derivada, citas, diario, recibos y métricas. Es una comprobación de integridad, no una tasa de calidad semántica. La sonda manual termina exit101 por la parada de calidad prevista.

| Recurso del piloto | Observado |
| --- | ---: |
| Peticiones / respuestas HTTP | 24 / 24 |
| Juicios válidos | 24 |
| Tokens de entrada / salida | 15.942 / 3.793 |
| Razonamiento reportado | 0 tokens |
| Coste externo conocido | USD 0,0031114 |
| Nuevas operaciones con coste desconocido | 0 |
| Tiempo HTTP acumulado | 29.948,763 ms |
| Mediana HTTP descriptiva | 1.086,896 ms |
| Duración de fase del runner | 35,103539 s |

Son tokens del clasificador, no de Codex. La mediana del piloto no es p50/p95 del producto ni un pase del límite 1,5×. Coste económico Codex/local desconocido. La estimación completa previa era USD 0,128482125 y la cota de ingeniería USD 0,4365141, con presupuesto USD 0,50; se informa el gasto real de los 24 recibos, no esas estimaciones. Acumulado: 6.192 operaciones externas, 23 campañas emparejadas completas, USD 4,570919228 conocidos más 24 operaciones históricas sin importe.

## Continuación

La V3 completa el piloto con 24 fichas válidas y preserva 12/12 útiles, pero acepta 8 ajenos, incluidos 2 nuevos frente al padre. Se conserva el fallo: no se ejecutan los 408 pares restantes ni se abren reservas. La causa observada es semántica: confunde silencio sobre el atributo pedido con ausencia registrada, y proyectos futuros con hechos realizados. Derivar la decisión localmente resuelve la redundancia estructural, no esas interpretaciones. La siguiente candidata debe reducir la carga de categorías y evaluar directamente la aportación de evidencia al dato solicitado, preservando correcciones y componentes útiles. Su diseño, pruebas y coste se congelarán antes de nuevas inferencias.

La candidata siguiente tendrá una única decisión de aportación textual con evidencia y razón breve. Se evalúa la hipótesis de que reducir tareas intermedias ayuda; no está demostrado que el esquema haya causado los errores. No se añaden reglas por ID, ejemplos del banco, revisores manuales al runtime ni selección retrospectiva de respuestas. Piloto fijo, cero daños nuevos, ganancia útil en TRAIN completo y todas las puertas originales se mantienen.

No cambió producto/default, DB, secretos, dependencias, dashboard o rama. Sin servicios/puertos, commit ni push. PID58987/session5563 observado en ejecución y después terminal, sin reintentos ni procesos ajenos detenidos. Reservas originales/V6 cerradas. El objetivo sigue activo: pasos 1 y 2 cerrados, paso 3 en validación; pasos 4 y 5 pendientes de sus puertas. No se publican gráficas de mejora de producto porque esta versión empeora el piloto.

Artefactos: `outputs/iromem/evaluation/semantic-link-span-v3-2026-10-22/`.
