# Ficha semántica automática V1: detención técnica

Fecha: 2026-10-22. Paso 3 activo. [Protocolo congelado](semantic-link-flash-lite-v1-protocol-2026-10-22.md). **Una respuesta recibida, cero juicios aceptados; calidad no evaluable.** Se detuvo la candidata antes de completar el piloto, sin ampliar o repetir llamadas.

## Qué se implementó

Builder/decoder Rust para nueve campos de vínculo semántico y citas, preparador offline `prepare-semantic-link-payloads` y runner manual ignorado que reutiliza el transporte generativo de bootstrap. Solo inputs opacos, pregunta y documento en mensajes. Gemini2.5FlashLite/GoogleAIStudio estándar, temperatura0, reasoning desactivado,1024tokens máximos, sin fallback/retry. No es una política de producto.

Solo include selecciona; exclude/indeterminate conservan razón sin selección. Toda abstención útil sigue contando como omisión. Fallos de esquema, citas o recibos son técnicos y no se convierten en indeterminate. El código valida claves duplicadas/extra, enums, coherencia, citas, ID, modelo/proveedor, finish_reason/refusal, tokens y coste; liga fuentes y binario exacto antes de credenciales.

## Fallo observado

Pregunta: «¿Cuántos cajones tiene el armario Lirio?» Fuente: «El armario Lirio tiene catorce cajones, según la ficha del inventario.»

El modelo devolvió query_quote «cuántos cajones tiene el armario Lirio»: cambió la mayúscula de «Cuántos» y la cita dejó de ser literal. La cita del documento sí era literal. El decoder rechazó la ficha con `missing_or_nonliteral_quote`. Se preservan body original de contenido, recibo, diario y parada; no se normaliza retrospectivamente la cita.

La revisión independiente detecta además que requested_focus contiene «catorce cajones», el valor de respuesta del documento, en lugar del foco solicitado en la pregunta. Ese es un defecto semántico adicional del campo; no fue el validador que produjo la detención técnica. La propuesta include coincide con el gold, pero pertenece a una ficha inválida y **no cuenta como juicio aceptado o acierto del pipeline**.

Resultados:1HTTP200,0juicios válidos,431pares no intentados y432sin juicio aceptado. La puerta del piloto24, el filtro432, los88 y las seis series están **no medidos**, no son0% de accuracy ni un fallo semántico de432casos. Las reservas originales/V6 permanecen cerradas.

## Verificación y ejecución

Suite final del código actual: **892 pruebas default y894 HTTP2**, cero fallos;24/76ignoradas. Formato correcto. La sonda manual ignorada posterior terminó con exit101 al detenerse según protocolo; no se confunde ese fallo vivo con las suites locales verdes.

Preflight independiente13.255/13.255 antes de inferencia. Se comprobaron432textos/gold/referencias de padre, payloads sin metadatos,215hashes de fuentes, cotas y binario exacto. La auditoría offline independiente pasó13.757/13.757 y reconstruyó4eventos,1respuesta/recibo y la parada. Validar estructura y procedencia no acredita fidelidad de un extractor.

Cambios de fuente:dos módulos experimentales y sus registros mínimos; sin dependencias nuevas, DB, secretos, política/default, dashboard, servicios, ramas, commits o push. RustCLI sin puerto, PID51777/session9465 terminales. La primera observación ps ocurrió después de su terminación; solo demuestra ausencia, no una confirmación previa de PID vivo. El handle había reportado ejecución y después exit101.

## Métricas de tiempo y coste

| Medición | Observado |
| --- | ---: |
| Inferencias externas intentadas / recibos completos | 1 / 1 |
| Entrada / salida del clasificador | 392 / 159 tokens |
| Razonamiento reportado | 0 tokens |
| Coste conocido | USD0,0001028 |
| Operaciones de coste desconocido nuevas | 0 |
| HTTP de esa única respuesta | 1.535,247ms |
| Fase del runner | 3.328727s |

No hay percentiles con una sola respuesta ni latencia de ruta productiva. Los tokens son del clasificador remoto, no los enviados aCodex. Coste Codex/local y tiempo de preparación no cuantificados. Acumulado externo: **6.156 operaciones**,23campañas emparejadas completas, **USD4,566482228 conocidos +24operaciones sin importe**.

El presupuesto congelado era USD0,50; estimación completa USD0,112930975 y cota conservadora USD0,3743095, calculadas antes de red. Se gastó solo el recibo indicado. Dos GET de catálogo/saldo no fueron inferencias. El proveedor/modelo sigue siendo un alias remoto sin revisión inmutable de pesos. Recursos completos≤1,5× y estabilidad siguen pendientes.

## Siguiente intervención

V1 queda detenida tras la primera respuesta por cita no literal; el coste se conserva y no hay juicio aceptado ni métrica de calidad. No se repite este intento ni se normaliza la cita para hacerlo pasar. La siguiente intervención cambiará la interfaz de procedencia: el modelo elegirá IDs de spans/segmentos literales suministrados en el payload, y el adaptador obtendrá citas del texto original. El foco solicitado se vinculará solo a spans de la pregunta, para impedir que el valor del documento ocupe ese campo. Esto corrige dos defectos observados del transporte de evidencia, no demuestra utilidad semántica. Misma candidata económica, datos/padre/orden/piloto/puertas intactos, sin excepciones para el primer caso. Las representaciones nuevas y su coste se congelarán antes de una nueva versión separada.

Se mantendrán fuente íntegra, incertidumbre y todas las puertas. IDs de spans válidos solo demostrarán referencias válidas; no que el span sea pertinente ni que mejore la selección. La versión nueva necesitará contrato, pruebas, presupuesto y preflight propios. Objetivo íntegro activo. Artefactos: `outputs/iromem/evaluation/semantic-link-flash-lite-v1-2026-10-22/`.
