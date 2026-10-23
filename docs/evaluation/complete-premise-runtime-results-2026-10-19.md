# Premisa incompatible V1: replay y flujo real

La variante optativa está implementada y verificada estructuralmente, pero **no se promueve ni se amplía a campañas completas**: el replay congelado pasa y el flujo real V5 añade un ID no permitido. V4 pasa. `baseline` permanece predeterminado y V6 sigue sin llamadas.

## Qué cambió

`per_memory_complete_premise_v1` usa propuesta de tipo Jev V2 sobre siete candidatos y verifica cada positivo con Perplexity Decider V1.2, revisión servida fijada. Guarda cuatro juicios reales: dato/parte, corrección ordinaria, incompatibilidad de una premisa necesaria y contraste explícito. Composición: **dato OR (corrección ordinaria AND incompatibilidad) OR contraste**. No hay veto global sobre parciales ni contraste, ni selección sin comprobación por posición o tipo propuesto.

Retrieval posee tipos y validación por política/verificador/revisión; bootstrap transporta comprobaciones aisladas concurrentes. Un recibo por petición, con las cuatro respuestas. El servicio y el auditor JSON independiente comprueban todos los campos, IDs, revisiones, agregación y consumo. Cualquier fallo de una comprobación degrada la operación completa y conserva una vez sus costes conocidos. No añade dependencias, secretos, conexiones SQL o puertos. [Protocolo de flujo real](complete-premise-runtime-protocol-2026-10-19.md).

## Diagnóstico congelado

El filtro pasó **12/12**; se ejecutaron los 31 restantes, **43 comprobaciones en total**, sin repetir los primeros doce. Los trece casos V5 reconstruidos preservan evidencia de base/scopedV1 sin IDs no permitidos y las cinco regresiones V4 dan los sets esperados. Coste nuevo de verificaciones: **USD 0,000525560**. Con propuestas históricas reutilizadas, los costes estimados son **1,275× V5 y 1,354× V4**, bajo el límite. Son estimaciones de un flujo reconstruido, no un `recall` nuevo ni prueba de completitud semántica.

El preflight del diagnóstico omitió la entrada separada del archivo de pregunta atómica. El primer intento de auditoría rechazó esa ausencia. Se conservaron el preflight y las solicitudes originales: su huella global, guardada antes de red, vincula el texto completo de la cuarta pregunta en cada payload. El auditor verifica las 43 coincidencias exactas y conserva `original_direct_fixture_preflight_passed:false`; no se presenta el vínculo directo original como aprobado. El generador se corrigió y la prueba de vinculación rechaza cualquiera de las 43 preguntas alterada o ausente. No se repitieron llamadas ni se modificó el preflight histórico. Los pilotos reales siguientes sí incluyen la huella directa del fixture.

## Medición de la CLI completa

Nuevas propuestas y verificaciones, mismos casos/corpus/gold. Directorios `per-memory-complete-premise-v1-{pilot,v4-targeted}-2026-10-19` conservan preflight, diario, auditoría ordinaria, informe de política y resumen CLI.

| Grupo | Pérdidas útiles de base/scopedV1 | IDs no permitidos | Coste completo USD | Coste/base histórica | HTTP | CLI p50/p95 | Puerta |
| --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| 13 V5 | 0 | 1 | 0,000943720 | 1,292× | 39 | 943 / 1478 ms | Falla calidad |
| 5 V4 | 0 | 0 | 0,000473504 | 1,354× | 23 | 1610 / 1754 ms | Pasa diagnóstico |

Los costes relativos usan subconjuntos históricos; todavía no son campañas emparejadas completas contemporáneas. Los porcentajes globales del producto no se actualizan a partir de estos casos seleccionados.

Dentro de los 91 candidatos V5 clasificados, se seleccionan 16 IDs permitidos y uno no permitido, se omite otro permitido y se rechazan correctamente 73. V4 selecciona los cinco permitidos y rechaza los otros 30. Cero pérdidas frente a base/scopedV1 no significa haber seleccionado todo el gold; tampoco cubre recuerdos fuera del prefijo.

La latencia CLI también requiere trabajo: estas medianas son mayores que las de los subconjuntos históricos (410 ms V5 y 385 ms V4). No se declara una puerta global de latencia emparejada medida, pero el coste bajo por token no hace rápida una secuencia de propuesta y comprobaciones. Una futura variante debe resolver y medir ambas dimensiones; no se abrirán campañas nuevas solo por el coste.

El ID nuevo es `cev5_068_01`: «La digitalización del libreto Luciérnaga se hará en TIFF de 600 puntos por pulgada». La pregunta pide si todo está digitalizado y listo para publicar. La propuesta nueva lo incluyó; no estaba entre los 43 positivos congelados. Perplexity acepta **dato/parte**, con las vías de corrección y contraste negativas. Por tanto la nueva condición sobre correcciones no podía rechazarlo: es un fallo de la vía de datos, que admite un formato previsto como evidencia sobre avance/terminación. No prueba ni digitalización completada ni ausencia de ejecución previa.

El ejemplo distingue un parámetro de trabajo de una parte pedida, una condición de publicación o un paso previsto que compruebe el resultado. La etiqueta original se conserva y se informa el nuevo error; no se elimina para pasar. Que el replay pasara no cubría nuevos positivos del proponente, como demuestra esta ejecución. Ninguna de estas métricas autoriza abrir V6.

## Verificación y coste acumulado

146 pruebas del workspace pasan, junto con formato y compilación. Cubren rescate pese a tipo propuesto incorrecto, rechazo de corrección compatible, contraste/dato sin incompatibilidad, cuatro respuestas obligatorias, revisión servida fijada, mezcla de esquemas, intersección y recibos, y fallo atómico con costes conocidos. Las trazas históricas omiten el nuevo campo opcional y conservan su formato.

Se guardan fuentes parciales y huellas de binarios, sin archivar ejecutables ni afirmar una reconstrucción autónoma completa. Las bases temporales propias se eliminan al terminar; no se toca el dashboard ni se detienen procesos ajenos.

Este avance añade **61 operaciones, 105 HTTP y USD 0,001942784**: 43 comprobaciones congeladas y 18 operaciones CLI nuevas. Acumulado del objetivo: **3549 operaciones, 21 campañas emparejadas completas, USD 0,244317514 conocidos y tres operaciones históricas de importe desconocido**. No incluye programación/revisión con Codex; ninguno de estos diagnósticos incrementa las campañas emparejadas completas.

## Próxima prueba

Antes de otro piloto, probar el verificador contra **todos los candidatos del prefijo de siete en los dieciocho casos**, no solo una lista histórica de positivos. Así la elegibilidad no oculta el nuevo formato ni otros candidatos que pueda proponer Jev.

La hipótesis semántica pendiente es una pregunta separada sobre la dimensión solicitada, aplicada solo a la vía de datos: dato AND dimensión pedida, OR corrección ordinaria AND incompatibilidad, OR contraste. Debe distinguir estado/avance, partes, límites, condiciones y comprobaciones previstas pertinentes de formato, método u otro atributo asociado, sin exigir el total completo ni convertir planes en ejecución. Debe preservar los parciales y las correcciones actuales.

Después, si ese contraste sobre todos los candidatos pasa, investigar una sola petición de veredictos sobre el prefijo completo, sin propuesta secuencial. Compartir consulta/reglas y mantener cada resumen en sus propias preguntas podría reducir etapas; los fallos previos de lotes impiden asumir equivalencia semántica o ahorro. Comparar contra las comprobaciones aisladas congeladas antes de incorporar. Ambas hipótesis están pendientes, sin mejoras ni ahorro demostrados; requieren protocolo y roster completo antes de gastar. Todas las puertas restantes —flujo real, campañas conocidas, consistencia y V6— siguen pendientes.
