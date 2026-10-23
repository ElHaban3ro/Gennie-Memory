# Tres vías completas: contraste prospectivo de modelos

Protocolo anterior a las llamadas; solo evaluación. Las dos familias anteriores no cumplieron calidad/coste. La vía exclusiva perdió refutaciones; el rescate OR añadió una corrección de una premisa no requerida. El catálogo oficial muestra modelos adicionales compatibles, sin demostrar su rendimiento. No se ajusta ningún prompt durante este contraste.

## Petición y modelos

Un solo recuerdo por petición, con las tres preguntas completas existentes: dato/parte V1, corrección ordinaria V4 y contraste explícito V4. Consulta y resumen son el único estado. Se conservan instrucciones y criterios íntegros; solo cambian nombres técnicos de las preguntas para agruparlas. Cada respuesta es choice A/none y el resultado es su OR. No se usa el tipo propuesto para omitir preguntas. Las etiquetas esperadas, IDs y metadatos de evaluación quedan fuera del payload enviado.

Los tres modelos prospectivos se fijan en `tests/fixtures/role-complete-provider-screen-v1.json`: Perplexity Decider V1.2, Respan Span-02 y Liquid d1-ultra. Se usa el ID de catálogo específico, sin alias latest, y se exige que el modelo servido coincida con la revisión fechada del catálogo observado. Una revisión distinta impide calificar el resultado. No cambiar IDs silenciosamente ni sustituir proveedor por caso. La consulta web de endpoints detallados no fue accesible con esa herramienta; no demuestra ausencia de servicio. La respuesta real determinará disponibilidad, revisión y recibo.

## Datos y etapas

Los mismos 43 positivos históricos V2 del contraste anterior se preparan para los tres modelos **antes de la primera llamada**: 129 solicitudes potenciales, con fuente, protocolo, catálogo observado y ejecutable vinculados por huella. Consultas, resúmenes y gold permanecen intactos; V6 no se abre ni se llama.

Primera etapa: doce pares caso/recuerdo comunes, fijados en el fixture. Incluye dos refutaciones, contraste ligado, dos parciales, condición, paso previsto, dos contextos ajenos, carné, otro libreto y el control de ausencia de solista. No se elimina el ID 06912 por su discrepancia semántica histórica ni se retira el control 06217 para pasar. La pertenencia al gold original define utilidad esperada; no se inventa una relación semántica exclusiva por respuesta positiva.

Cada modelo completa estos doce casos aunque cometa un error de calidad, para conservar patrones negativos. Solo si los doce tienen juicio esperado, revisión servida coincidente y recibos completos se ejecutan los otros 31 positivos de ese modelo. Las doce observaciones se reutilizan; nunca se repiten para elegir la mejor. Un fallo HTTP o esquema inválido detiene las llamadas restantes de ese modelo y deja su etapa incompleta; no se reintenta ni se trata como abstención fiable. Los modelos se ejecutan en el orden fijo del fixture.

## Verificación, costes y puertas

La prueba manual ignorada `live_complete_provider_probe` usa la autorización general de pruebas sintéticas por OpenRouter y el secreto existente, sin imprimirlo. Registra started/respuesta/fallo, tiempos y recibos. Directorio de un solo uso. Invocación explícita: `cargo test --offline -p iromem live_complete_provider_probe -- --ignored --test-threads=1`. No cambia enums, modelos predeterminados, SQL, puertos ni proveedores admitidos en producto.

Inicialmente son 36 HTTP; máximo 129 si los tres pasan. Estimación USD 0,003 y uno a dos minutos, CLI sin puerto. Los precios publicados solo orientan la estimación: usar recibos reales, conservar importes desconocidos y no asumir gratuidad ante error. El gasto nuevo incluye cada llamada ejecutada, también las de modelos descartados.

El auditor Rust debe reconstruir el contraste fuente y todo el roster, recomponer templates completos y OR, comprobar ausencia de gold en payloads y revisión servida. Debe volver a calcular la elegibilidad para la segunda etapa, sin confiar en `runner-status.json`, y rechazar llamadas fuera de etapa o solicitudes repetidas. Si hubo fallo técnico, verificar que no se siguió llamando a ese modelo.

Solo un modelo con las 43 observaciones completas puede juzgar preservación y coste del mecanismo reconstruido. Mismos trece casos V5/cinco V4: sin IDs ajenos, pérdidas de base/scopedV1 o degradación; alguna ganancia V5 y cinco sets exactos V4. Coste por grupo: nuevas comprobaciones completas + propuesta histórica, ≤1,5× de base histórica. Mostrar también omisiones permitidas, vías reales y tiempos de etapa, sin llamar contemporáneo/end-to-end al flujo reconstruido.

Un pase solo habilita implementar una variante explícita y volver a medir el flujo real completo, después campañas conocidas, consistencia y V6 reservada con revisión independiente. Si ninguno pasa, conservar todos los resultados y revisar la causa, sin ampliar por precio o elegir mejores outputs entre modelos.
