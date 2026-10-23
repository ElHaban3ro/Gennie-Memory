# Presentación separada V2: piloto detenido por coordenadas de hallazgo

UTC 2026-10-21. Paso 3 del objetivo activo. **La prueba se detuvo tras cuatro respuestas: tres válidas y una rechazada por un path inexistente. Quedaron 42 entradas sin enviar. No hay pase de capacidad ni mejora de recuperación demostrada.** [Protocolo previo](semantic-presentation-v2-protocol-2026-10-21.md), [informe Rust](../../outputs/iromem/evaluation/semantic-presentation-v2-2026-10-21/report.json) y [diagnóstico con respuesta conservada](../../outputs/iromem/evaluation/semantic-presentation-v2-2026-10-21/failure-diagnosis.json).

## Cambio y comprobación antes de red

Se mantuvieron los 46 inputs canónicos, IDs, etiquetas congeladas, normalizador, modelo/ruta, cuatro dimensiones, parser y aceptación de V1. El mensaje ahora identifica por separado texto original, campos propuestos y resultado operativo. Las citas generales de roles V1 pasan a un canal con su path original y valor exacto; las menciones completas de entidades/propietarios y sus clases permanecen en el árbol. En la normalización solo se separa /text copiado. No hay decoder, inventario generativo nuevo ni reparación.

La alternativa conservadora respondió a la crítica de diseño: mover una mención podría relegar sus calificadores a información ignorada aunque el JSON fuera reversible. Se conservaron mention_quote, type_quote, bearer_quote, owner_quote y subject_quote, se explicó que ambos canales deben examinarse y se mantuvo el diagnóstico de restricciones sin realización operativa. La reversibilidad estructural no acredita equivalencia semántica del prompt; esta intervención conjunta de envoltura/instrucciones sigue siendo diagnóstica y conocida.

La reconstrucción exacta de ambos árboles se ejecutó sobre las 46 entradas, junto con pruebas de null/ausencia, vacíos, Unicode, arrays, claves y escapes, campos quote desconocidos y key=quote como valor semántico de una condición. No cambian parámetros ajenos a mensajes. La separación se limita a roles del esquema V1, no a cualquier JSON arbitrario que imite sus sufijos.

El revisor de código detectó que las copias de etiquetas V2 podían ser internamente coherentes sin compararse con las copias congeladas del padre. Se corrigió antes de inferencia: el preflight reconstruye y compara en bytes preparación, requests, paquete, mapa, las cuatro anotaciones y parent-bindings. Los dictámenes anterior y posterior se conservan. Las etiquetas/revisiones se reutilizan exactamente; no hubo nueva revisión ciega de calidad ni exposición de etiquetas al modelo. La comparación con V1 es histórica, no un control contemporáneo que aísle deriva del proveedor.

## Resultado completo del intento, con denominadores

| Medida | Resultado |
| --- | --- |
| Operaciones planificadas | 46 |
| Iniciadas / HTTP200 terminales / recibos conocidos | 4 / 4 / 4 |
| Respuestas válidas para el contrato | 3 |
| Respuesta rechazada por path | 1 |
| No enviadas | 42 |
| Comparaciones por dimensión válidas / previstas | 12 / 184 |
| Discrepancias en las doce comparaciones válidas | 0 |
| Tokens informados de entrada / salida | 8.067 / 1.260 |
| Coste nuevo informado | USD 0,026214 |
| Costes nuevos desconocidos / reintentos | 0 / 0 |
| Duración de intento terminal p50 / p95 | 2.272 / 3.089 ms; muestra de cuatro |
| Duración del runner | 3,108 s; un lote de cuatro |

Las tres respuestas válidas son controles que debían aceptarse. Cero discrepancias entre sus doce juicios no es un 100 % del piloto ni una mejora frente a V1. El caso no aceptable del lote tiene salida inválida y no se contabiliza como bloqueo semántico correcto. Se conservan también las 42 operaciones no enviadas; ausencia no es acierto ni coste desconocido de una llamada que nunca se realizó. El test ignorado terminó correctamente como proceso porque drenó/persistió el fallo; eso no equivale a pase del experimento.

## Fallo observado y siguiente contrato

La respuesta s001 identifica expresamente el conflicto temporal: la propuesta y la normalización dicen hoy, mientras el original dice ayer. Sin embargo, devuelve raw_paths con `/fields/document/certain/0/context/0/value`. Ese path corresponde al canal mostrado; el contrato V2 exigía coordenadas relativas al árbol raw original, cuyo path no empieza por /fields. El parser sin cambios lo rechaza. No se quitó el prefijo, reescribió la respuesta ni reanudó el diario.

Esto ofrece una observación parcial de reconocimiento de contradicción, no una predicción válida ni un pase de todas sus dimensiones: evidence_attribution sigue faithful en la respuesta, frente a defective en la etiqueta congelada. No se recalcula una puntuación arreglando el path retrospectivamente. La negación invertida y los demás controles posteriores no se ejecutaron en V2.

Una próxima versión debe declarar referencias de campos o coordenadas alineadas con lo que el modelo recibe, validarlas antes de inferencia y conservar la traducción exacta a los campos originales. Ese mecanismo aún no está implementado. No será un arreglo a las respuestas V2 ni una excusa para abrir V6: siguen pendientes el cribado semántico completo, extracción/tubería automática, conocidos100/126, baseline, recursos/estabilidad y cohorte nueva ciega.

## Coste y conservación

La consulta pública de catálogo confirmó la ruta OpenAI y precedió a la estimación anunciada de USD 0,43 y 1–3 minutos para hasta 46 peticiones. La ilustración conservadora de bytes de payload más salida máxima fue USD 1,72, sin garantía de factura. Solo se enviaron cuatro peticiones antes del fallo; se suman todos sus recibos, sin estimar cargos de las 42 no enviadas. Catálogo: un GET de metadatos, separado de las cuatro operaciones de inferencia. Alias solicitado/servido openai/gpt-4.1, proveedor informado OpenAI, revisión interna no verificada independientemente. No se afirma negociación HTTP/2 observada.

Acumulado: **6051 operaciones lógicas, 23 campañas emparejadas completas, USD 1,082116508 conocidos más 16 operaciones históricas de coste desconocido**. Codex está excluido. Tokens y tiempos anteriores pertenecen a este verificador parcial; no miden entrada de Codex, recursos de una tubería completa o recall/end-to-end. Tampoco se comparan sus percentiles parciales con los 46 casos de V1.

Pasaron 500 pruebas con/sin HTTP2, formato y diff. Cuatro bancos manuales, diez informes de extracción y cuatro auditores/reportes locales previos se reproducen idénticos; el informe V2 también. El refactor comparte archivos y cambia la captura de fuentes de preparaciones V1 futuras: no se afirma que la fuente o preparación histórica sea idéntica. Se preservan los artefactos archivados y sus resultados/request canónicos.

Snapshots001–061, gold, predeterminado, producto, SQL, dashboard, servicios, README/gráficas y V6 se conservan. La falta de una mejora completa del producto impide reemplazar sus figuras actuales por porcentajes del intento parcial. El objetivo sigue activo en el paso3, sin promoción.
