# Vía alternativa tras rechazo: resultados

La vía complementaria recupera las dos refutaciones V5, pero añade un ID no permitido V4 y excede el presupuesto estimado. **No se implementa en producto ni se amplía a V6.** El objetivo continúa activo.

## Mecanismo y comprobaciones

Se ejecutaron las tareas complementarias de **todos los 25 rechazos Decisions** entre las 43 propuestas históricas del diagnóstico anterior. La elegibilidad no usa gold. Los positivos iniciales no se volvieron a preguntar; no se elige proveedor o respuesta por recuerdo. Consulta, resumen, modelo e instrucciones completas están vinculados a las fuentes congeladas. Cada resultado final es primer juicio OR tarea complementaria.

El test manual conserva solicitudes/preflight antes de red, huella de ejecutable, started/respuesta y recibos. El informe Rust independiente reconstruye el diagnóstico anterior desde sus fuentes y exige el roster entero, template complementario exacto, IDs/revisiones/textos y OR. Un fallo técnico bloquea el caso y no se interpreta como abstención fiable. Los artefactos se conservan en `outputs/iromem/evaluation/role-opposite-task-probe-2026-10-19/`. [Protocolo](opposite-task-probe-protocol-2026-10-19.md).

138 pruebas del workspace pasan; los tres tests remotos manuales se omiten normalmente. El nuevo test manual se ejecutó una vez con 25 respuestas completas. Compilación del evaluador y formato pasan. No hay cambios de modelo admitido por producto, SQL, secretos, puertos ni bases temporales nuevas.

## Métricas

| Grupo reconstruido | Evidencia útil previa perdida | IDs no permitidos | Nuevas comprobaciones USD | Flujo estimado/base histórica | Calidad |
| --- | ---: | ---: | ---: | ---: | --- |
| 13 V5 | 0 | 0 | 0,000412400 | 2,545× | Pasa preservación |
| 5 V4 | 0 | 1 | 0,000481800 | 3,780× | Falla |

Los dos rescates V5 son `cev5_073_06` —Laurel no usa agua de la red— y `cev5_066_06` —en Ribera no se capturan aves—. Ambos estaban propuestos como datos y antes carecían de una comprobación de corrección. La nueva tarea devuelve evidencia útil para esas refutaciones; no se completa la información ausente.

El ID nuevo no permitido por el gold es `cev4_062_17`: ante la pregunta por el número total de músicos confirmados, el recuerdo dice que no habrá solista invitado y que su pasaje se repartió entre violines. La primera comprobación lo rechazó; la tarea de corrección lo aceptó. La pregunta no presupone un solista invitado: esa ausencia no refuta una premisa requerida por la pregunta. La posible utilidad parcial de una ausencia es una cuestión distinta de esa corrección; aquí se conserva la etiqueta original, sin usar la falta de un total como motivo general para rechazar parciales ni modificar el gold para pasar.

La nueva evidencia confirma el problema de la ruta exclusiva, pero refuta la suficiencia del rescate OR: recuperar refutaciones no basta si una negación de otro atributo también se acepta. No se compensa ese error con los dos rescates ni se retira el control del banco.

Los importes del flujo estimado reutilizan propuestas y primeras verificaciones históricas. V5 suma USD 0,000628320 de propuesta + USD 0,000819000 de primera verificación + USD 0,000412400 nuevo; V4 suma USD 0,000251244 + USD 0,000588900 + USD 0,000481800. No son cargos nuevos de las primeras etapas ni una medición end-to-end contemporánea. El límite ≤1,5× se conserva y ambos grupos lo exceden.

## Coste y dirección siguiente

Gasto remoto nuevo completo: **25 operaciones/HTTP, USD 0,000894200**. Acumulado del objetivo: **3452 operaciones, 21 campañas emparejadas completas, USD 0,241949198 conocidos y tres operaciones históricas de importe desconocido**. Las nuevas comprobaciones de etapa no incrementan las campañas emparejadas completas. No incluye programación/revisión con Codex.

La siguiente investigación debe combinar precisión sobre premisas con un coste viable, sin seguir suponiendo que cambiar formato abarata. La consulta del [catálogo oficial de OpenRouter](https://openrouter.ai/api/v1/models?output_modalities=decisions), realizada en este avance, muestra otros modelos compatibles con decisiones tipadas. Entre ellos están Perplexity Decider V1.2, Respan Span-02 y Liquid d1-ultra, con IDs versionados y precios de entrada publicados. No se han ejecutado ni se les atribuye rendimiento.

Hipótesis pendiente: diagnóstico acotado de proveedores distintos sobre todos los tipos de fallo conocidos, con las tres preguntas completas —dato, corrección ordinaria y contraste— para un solo recuerdo por petición, sin que el tipo propuesto elimine una vía. El catálogo no demuestra precisión, ahorro real o disponibilidad efectiva de una petición: deben medirse antes de introducir otro proveedor en producto. Fijar modelos, límites, roster y criterios antes de gastar; conservar los negativos. Siguen pendientes campañas completas conocidas, consistencia y V6 reservada/revisada aparte.
