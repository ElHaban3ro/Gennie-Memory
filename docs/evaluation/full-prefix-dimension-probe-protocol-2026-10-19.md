# Dimensión solicitada: diagnóstico de prefijo completo

Solo evaluación, sin cambiar la variante optativa anterior ni `baseline`. El flujo real añadió un formato previsto como evidencia de terminación, que no estaba en los positivos congelados. Se evaluará cada candidato del prefijo, independientemente de su propuesta anterior.

## Población e hipótesis

Congelar los siete primeros candidatos de cada uno de los dieciocho casos de los pilotos reales: 91 V5 y 35 V4, **126 juicios**. Mantener consulta, resumen, ID/revisión, corpus y gold originales. Incluir tanto positivos como negativos de Jev. No seleccionar por acierto, error o coste del verificador anterior.

Añadir dos controles sintéticos de desarrollo declarados en `asked-dimension-controls-v1.json`: preguntar formato y resolución usando exactamente el mismo recuerdo TIFF que fue incorrectamente admitido para terminación. Ambos deben conservarse cuando se pregunta por ese atributo. Son controles del autor, sin revisión independiente ni condición de holdout; no cambian etiquetas anteriores.

Modelo/revisión: Perplexity Decider V1.2 y `perplexity/pplx-decider-v1.2-20261016`. Una memoria por petición. Preservar íntegros los cuatro campos anteriores y añadir `asked_dimension` desde `asked-dimension-v1.json`. El estado contiene solo consulta/resumen; gold y metadatos no se envían.

Composición: **(dato AND dimensión solicitada) OR (corrección ordinaria AND incompatibilidad necesaria) OR contraste explícito**. La nueva pregunta no veta correcciones o contraste. Debe conservar partes, límites, estados explícitos desconocidos/pendientes, condiciones y comprobaciones previstas del resultado preguntado; no deduce ejecución o no ejecución de un plan. Formato/método son útiles cuando se preguntan, no por ser atributos del mismo trabajo.

## Ejecución y auditoría

Preparar las 128 solicitudes y todas las huellas de datos/templates/protocolo antes de red. Test manual ignorado `live_full_prefix_dimension_probe`, una sola ejecución con clave existente de OpenRouter, bajo autorización general sintética. Comando: `cargo test --offline -p iromem live_full_prefix_dimension_probe -- --ignored --test-threads=1`. Directorio de un solo uso; started/respuesta/fallo, recibos, tiempos y ejecutable conservados. No reintentar una operación incierta. Error HTTP, esquema inválido o revisión distinta detiene las llamadas restantes y deja el diagnóstico incompleto; un error de calidad no detiene la población fijada.

Máximo **128 HTTP**, estimación **USD 0,003 y uno a dos minutos**, CLI sin puerto. Todos los cargos ejecutados cuentan; solicitudes preparadas sin ejecutar no son operaciones facturadas. No hay datos reales, tablas, conexiones, dependencias o proveedor de producto nuevos.

El auditor Rust reconstruye roster y expectativas desde pilotos/gold originales y los dos controles declarados, comprueba los cinco templates, campos, revisiones y OR. No confía en runner-status. Debe conservar cada vía real y todos los errores, separar 126 candidatos de dos controles y mostrar cobertura fuera del prefijo aparte. El fixture atómico y la quinta pregunta tienen huellas directas; el template de cuatro vías se vincula al diagnóstico previo congelado y a la equivalencia del payload de producto.

## Puerta y siguientes etapas

Para pasar esta prueba semántica aislada: 128 respuestas completas y recibos/revisiones válidos, **cero IDs no permitidos seleccionados y cero permitidos rechazados dentro del prefijo**, con ambos controles correctos. Es una comprobación de todos los candidatos, más fuerte que conservar únicamente seleccionados anteriores. No se cambia gold para pasar. Tampoco se declara calidad de recuerdos fuera del prefijo, corpus completo o datos reales.

Este diagnóstico no es una implementación eficiente del producto: registra costes y tiempos individuales, sin etiquetarlos end-to-end. Solo si pasa podrá investigarse agrupación en una petición por consulta con reglas compartidas y resumen local a cada pregunta, para quitar la propuesta secuencial. Los fallos anteriores de lotes obligan a comparar calidad, coste y latencia contra las observaciones aisladas; no asumir equivalencia. La definición agrupada se congelará antes de llamar y no se elegirá la mejor salida por recuerdo. No se llama a esa segunda etapa si la puerta semántica falla.

Después siguen variante optativa de flujo real, campañas completas conocidas, puertas de calidad/coste/p50/p95 ≤1,5×, consistencia y V6 reservada/revisada aparte. No hay promoción por un diagnóstico aislado ni se usa V6 para ajustar.
