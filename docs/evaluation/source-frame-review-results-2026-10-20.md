# Auditoría retrospectiva: extraer mejor no basta

Fecha UTC:2026-10-20. La nueva auditoría localiza errores posteriores de atribución aun cuando las representaciones conservaban la distinción pertinente. No demuestra una mejora de recuperación ni habilita adopción. También descarta antes de gastar otra tanda una composición que solo vetaría roles congelados. El objetivo sigue activo en el paso3; default y V6 permanecen intactos.

## Revisión sin selecciones

Un crítico nuevo recibió únicamente125fuentes/frames decodificados —20preguntas y105memorias— con IDs opacos. No recibió etiquetas, respuestas del comparador, reportes ni mapa privado. Completó todos antes del cruce. Es un agente de la misma familia de modelo, no una revisión humana ni independencia estadística. Sus valoraciones de fidelidad son juicios semánticos falibles.

Resultado de esa revisión:69frames fieles,48defectuosos y8inciertos. Dimensiones marcadas defectuosas: sujeto12, atributo18, alcance/condiciones20, polaridad1 y estado6; pueden coincidir varias en un frame. No se equipara un array de palabras no contiguas a una cita ni se exige copiar todos los conectores.

Se añadió `audit-source-frame-review` a `iromem-eval`, fuera del producto. Revalida el informe padre desde sus fuentes, el roster completo, hashes, mapa,125fuentes/frames exactos, enums, coherencia de dimensiones, explicaciones y citas literales de fuente. Después recompone los128pares originales. La validación estructural no certifica los juicios de fidelidad. No transforma un frame incierto en uno fiel.

## Cruce completo con resultados históricos

| Frames del par, según revisión | Pares candidatos | Coinciden con gold histórico | Desacuerdos históricos |
| --- | ---: | ---: | ---: |
| Ambos fieles | 41 | 36 | 5 |
| Al menos uno defectuoso | 83 | 75 | 8 |
| Incierto, sin defecto declarado | 2 | 1 | 1 |

Los2controles se conservan aparte: ambos coinciden con sus etiquetas aunque presentan algún frame defectuoso. Los14desacuerdos históricos originales no se sustituyen ni se reclasifican como resultado nuevo. Tampoco se deduce causalidad de la asociación entre frame y selección: el matcher recibió además los textos originales y pudo atenderlos, ignorarlos o contradecirlos.

Entre los5desacuerdos con ambos frames considerados fieles hay4aceptaciones y1omisión según el gold histórico. **No son cinco errores actuales inequívocos.** La omisión de la muestra de Índigo preparada para frote húmedo corresponde a una etiqueta posteriormente distinta en la versión de desarrollo; no se usa como contraejemplo causal actual. El tramo Sarga/Linde conserva posible ambigüedad. El vínculo de «ese libreto» en Neblina merece cautela al valorar suficiencia de su frame.

Dos casos bastan para refutar «arreglar únicamente la extracción resuelve todos los fallos observados», condicionada a esta revisión:

- Pregunta por la contraseña del buzón **Ribera**. La fuente da la contraseña de **Salitre**. El frame de pregunta conserva Ribera y el de memoria Salitre y su valor. El matcher declara `supports_requested_part`; la distinción pertinente estaba disponible y se transfirió un dato de otro buzón.
- Pregunta por los días de conservación de crudos de **Cendal/M4**. La fuente afirma que la solicitud de **Lyra** es para procesados, no crudos. Sus frames conservan los referentes distintos. El matcher declara `refutes_premise`, aunque esa negación de Lyra no refuta la política de Cendal/M4.

El crítico de mecanismos confirmó ambos ejemplos después del cruce. No se afirma que toda representación alternativa sea inútil ni cuánto mejoraría corregir los48frames; no se hizo una intervención causal con extracción corregida. Se descarta repetir más campos como si todavía no se hubiera ensayado esa arquitectura.

## Composición descartada sin llamadas

Se consideró ejecutar Perplexity/LocalV5 y Microsoft/LocalV5 en paralelo, conservando el rol de Perplexity si Microsoft lo juzga útil. La paralelización sí retiraría una dependencia de transporte, pero ese veto binario no puede reparar un rol útil equivocado.

En la respuesta congelada de Laurel, `cev5_073_17` es un aporte parcial A y Perplexity lo llamó B. Cualquier verificador binario solo puede dejar B o producir none: ambos resultados fallan el contrato. El diagnóstico enumera esas posibilidades y demuestra al menos un error inevitable para **cualquier** veto con ese rol primario congelado. No presupone cómo respondería Microsoft ni vuelve a llamar esperando otra salida favorable. Un64/64retrospectivo sobre el mínimo no probaría complementariedad porque Perplexity ya acertaba todo allí.

## Consecuencia para el siguiente mecanismo

La próxima variante debe justificar cómo adjudica la relación y corrige un rol útil, además de rechazar referentes ajenos. Un veto binario o mejorar solo la extracción no satisface ese requisito. Debe conservar partes/refutaciones y medir el coste completo de cualquier preparación o adjudicación. La literalidad de nombres tampoco queda validada automáticamente: sus límites ya se observaron antes. No se baja la puerta de calidad ni se abreV6para elegir un mecanismo.

## Verificación y coste

254pruebas del workspace pasan con y sin HTTP/2; formato y compilación pasan. Tres pruebas nuevas detectan roster incompleto/duplicado/ajeno, citas inventadas y pérdida de incertidumbre. El CLI reconstruye el padre sin cambios y el nuevo informe se reproduce exactamente. La primera versión del informe y la versión con recuento adicional de dimensiones coinciden en todos los campos compartidos; ambas se conservan.

Coste incremental: **0HTTP/0USD de proveedor**. No se vuelven a cobrar recibos reutilizados; Codex de programación/revisión queda fuera. Acumulado sin cambios:5.606operaciones,23campañas completas, **USD0,569447228 conocidos +4 operaciones históricas de importe desconocido**. No hay política runtime, tabla, conexiónSQLite, dependencia, servicio, credencial o proceso del usuario modificado. Las métricas/gráficas del README permanecen en su ruta vigente.

[Protocolo](source-frame-review-v1-protocol-2026-10-20.md). Artefactos en `outputs/iromem/evaluation/source-frame-review-v1-2026-10-20`: entrada ciega, respuesta completa, mapa privado, unión de todos los pares, informe reproducible, revisión crítica y prueba de imposibilidad. Cierre de huellas/contabilidad: `quality-goal-2026-10-19/snapshots/040`.
