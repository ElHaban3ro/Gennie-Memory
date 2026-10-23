# Tres vías completas: contraste de modelos

Ningún modelo supera el filtro común de doce casos. **No se ejecutan los 31 casos restantes de ninguno ni se incorpora otro proveedor en producto.** `baseline` y V6 permanecen intactos; el objetivo sigue activo.

## Qué se probó

Perplexity Decider V1.2, Respan Span-02 y Liquid d1-ultra recibieron exactamente las mismas preguntas completas de dato/parte V1, corrección ordinaria V4 y contraste V4 para un solo recuerdo. Las tres vías se combinan con OR, sin que el tipo propuesto omita una tarea. Los doce casos comunes cubren siete recuerdos permitidos y cinco no permitidos; son regresiones conocidas seleccionadas, no una estimación de calidad general.

Se prepararon antes de red los 43 positivos históricos para los tres modelos, pero solo se autoriza su segunda etapa si el filtro es perfecto. Las 129 solicitudes preparadas incluyen 93 que **no se ejecutaron ni se contabilizan como gasto u operaciones**. Las doce observaciones no se repiten para elegir una respuesta mejor. No hubo errores HTTP ni reintentos.

El auditor Rust reconstruye fuentes y etapas, verifica payloads completos sin gold, OR, IDs/revisiones y revisión servida. No confía en `runner-status.json`. Los tres modelos devolvieron la revisión fechada esperada; la consulta web previa de endpoints detallados no había podido confirmar disponibilidad, que ahora sí tiene evidencia de estas peticiones concretas. [Protocolo](role-complete-provider-probe-protocol-2026-10-19.md).

## Resultados del filtro

| Modelo | Juicios esperados | IDs no permitidos aceptados | Útiles rechazados | Coste nuevo USD |
| --- | ---: | ---: | ---: | ---: |
| Perplexity Decider V1.2 | 11/12 | 1 | 0 | 0,000112680 |
| Respan Span-02 | 9/12 | 1 | 2 | 0,000087060 |
| Liquid d1-ultra | 10/12 | 2 | 0 | 0,000225792 |

Los recibos están completos. Estos precios son los observados para doce comprobaciones de tres vías, no el coste de un `recall` completo ni una proyección validada de las otras 31. La pertenencia al [catálogo oficial](https://openrouter.ai/api/v1/models?output_modalities=decisions) y su precio publicado no se usaron como prueba de precisión.

Perplexity conserva las dos refutaciones, el contraste, los dos parciales, la condición y el paso previsto del filtro. Acepta `cev5_071_03` por **corrección ordinaria**, mientras dato y contraste responden negativo. Ante «¿qué avalista debo llevar para reservar Olmo?», el recuerdo exige carné vecinal. Eso no excluye que además se necesite avalista. Es el único fallo del modelo en este filtro, sin atribuirle superioridad general a partir de doce casos.

Respan rechaza `cev4_063_09`, ingredientes con cantidades ilegibles, y `cev5_068_12`, condición de publicación. Acepta `cev5_068_07`, de otro libreto, por las dos vías de corrección. Liquid conserva los siete permitidos, pero acepta tanto el carné como el otro libreto; este último también obtiene contraste positivo. Las vías visibles son juicios fallables, no demostraciones semánticas.

## Verificación y registro

140 pruebas del workspace pasan. Tras reforzar el vínculo catálogo/revisión, pasan además las 57 pruebas focalizadas del evaluador, formato y compilación. El test remoto manual se ejecutó una sola vez; cuatro tests remotos siguen ignorados en la suite offline normal. Se corrigió un import sin uso en el auditor anterior; no cambia sus resultados ni contratos.

Artefactos completos del diagnóstico: `outputs/iromem/evaluation/role-complete-provider-probe-2026-10-19/`, con solicitudes, preflight, diario, estado del ejecutor e informe independiente. Se conservan huella del ejecutable y fuentes parciales; no se archivan binarios ni se afirma una reconstrucción autónoma de todo el workspace. No hay cambios SQL, secretos, puertos, bases temporales o proveedores admitidos en producto.

Gasto nuevo: **36 operaciones/HTTP, USD 0,000425532**. Acumulado del objetivo: **3488 operaciones, 21 campañas emparejadas completas, USD 0,242374730 conocidos y tres operaciones históricas de importe desconocido**. Un filtro de modelos no es una campaña emparejada completa. No incluye programación/revisión con Codex.

## Próxima hipótesis

El siguiente experimento puede fijar Perplexity —por el patrón concreto observado, sin mezclar respuestas de proveedores— y añadir una pregunta atómica de incompatibilidad a la vía ordinaria. Composición prospectiva: **dato OR (corrección ordinaria AND incompatibilidad de una premisa necesaria) OR contraste explícito**. El contraste conserva su comprobación de vínculo contextual, sin veto global sobre datos o parciales.

La nueva pregunta debe distinguir contradicción de coexistencia: un requisito descrito no agota los demás salvo exclusión expresa. Se mantienen todos los negativos, las correcciones válidas y las etiquetas originales. La comprobación adicional puede equivocarse o añadir coste; no está implementada ni medida en este avance. Requiere protocolo propio, el mismo filtro completo y, solo si pasa, los restantes positivos con recibos reales. Siguen pendientes flujo real completo, campañas conocidas, consistencia y V6 revisada aparte; no se relajan puertas para ampliar.
