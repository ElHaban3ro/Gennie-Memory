# M5: revisión adversarial entre agentes de cinco etiquetas de conversación

Estado: revisión **entre agentes**, 2026-10-16. No es etiquetado humano independiente ni una nueva medición de iROMEM. No modifica las etiquetas originales, la ruta de producto ni los resultados históricos del [informe de desarrollo](conversation-m5-development-2026-10-16.md).

## Procedimiento y hallazgo

Dos subagentes leyeron por separado y solo en modo lectura el [paquete ciego](conversation-m5-style-audit-blind.md): las cinco preguntas originales y los treinta resúmenes de cada conversación, sin etiquetas esperadas, candidatos ordenados ni respuestas del sistema. Uno propuso los IDs admisibles; el otro intentó refutar que hubiera una única respuesta. Después intercambiaron objeciones en una segunda ronda. Un tercer subagente, con acceso al informe de desarrollo y al roadmap, atacó la inferencia experimental y propuso una prueba de falsación. Ninguno modificó archivos ni llamó proveedores.

Los dos revisores ciegos seleccionaron de forma independiente **dos recuerdos explícitos por pregunta**: la estructura pedida para responder y el idioma pedido para explicar el resultado.

| Conversación | Estructura | Idioma |
| --- | --- | --- |
| 021 | `cm5_2f780592ca1a2f9022c5` | `cm5_ed945be6748934034fa8` |
| 022 | `cm5_faef6c8f12040c5c3ef4` | `cm5_ea78ee73bc1944e14160` |
| 023 | `cm5_c295aacdc2c1426a0665` | `cm5_b608742cdee8b6bfb409` |
| 024 | `cm5_5fdfd02b5ebda685a318` | `cm5_9a32c53da00deec4ab9c` |
| 025 | `cm5_f382efbbafdc84b01ddc` | `cm5_31e39b3a3edda5160090` |

La etiqueta del fixture acepta solo el ID de estructura. La pregunta «¿Qué formato o estilo de respuesta se pidió?» no excluye el idioma. Los revisores también señalaron restricciones de presentación en 023 y 024 como casos limítrofes de menor confianza; no las equipararon a una petición explícita de formato. No encontraron un caso convincente sin respuesta.

En la segunda ronda ambos retiraron la idea de usar «¿Qué estructura de respuesta se pidió?» como corrección retroactiva. Esa formulación deja un único ID estructural, pero **cambia el dato solicitado** y da una pista léxica; debe tratarse como un ítem nuevo. La crítica metodológica añadió que la matriz 2×2 usó cuatro preguntas elegidas tras fallar, repetidas tres veces, solo tres candidatos y un puesto primero forzado con texto indexado artificial. Muestra sensibilidad de ese montaje al orden y a la redacción; no ofrece una política capaz de reconocer el ID correcto en la ruta real de veinte candidatos. Los negativos de tipografía ausente no cubren distractores semánticamente cercanos, ámbito ajeno ni revisión caducada.

## Decisión provisional y falsación

La coincidencia de agentes es evidencia útil para **priorizar una auditoría**, no una validación independiente: comparten familia de modelos y pueden compartir sesgos de lectura. Mantener los resultados 70/75 y 0/12 como mediciones históricas bajo sus etiquetas originales, pero no usarlos solos para concluir que el clasificador falla en una pregunta de respuesta única. No promover `natural`, la instrucción experimental, la pregunta aclarada ni una regla que favorezca el primer candidato. El evaluador actual compara `selected == expected_ids`; antes de puntuar preguntas con alternativas admisibles, su contrato de etiquetas debe distinguir «cualquiera de estos IDs» de «seleccionar todos».

Prueba prospectiva mínima, para preparar **antes** de nuevas llamadas remotas:

1. Obtener una revisión humana de las cinco preguntas originales, eligiendo todos los IDs razonables sin mostrar el resultado de iROMEM. Registrar desacuerdos y resolver por adelantado si se evalúa una respuesta única, varias alternativas o una solicitud de aclaración. Un usuario informado del diagnóstico puede revisar, pero una segunda persona sin esa información aporta mayor independencia.
2. Crear al menos cinco conversaciones **nuevas de desarrollo**, separadas por sujeto, con preguntas originales y específicas, controles negativos difíciles y otras categorías centinela. Fijar preguntas, memorias, etiquetas admisibles, hashes, modelo, métricas y criterio de descarte antes de ejecutar. No abrir calibración ni reserva para elegir la variante.
3. Medir primero con el mismo conjunto de candidatos y resúmenes congelados si la redacción o el orden cambian la selección; incluir un orden placebo que ponga un distractor plausible primero. Después comparar la ruta vigente contra **una sola** política realizable sin conocer el ID esperado, con corpus intacto y hasta veinte candidatos. Medir por conversación cobertura, semilla correcta, abstenciones, falsos recuerdos, cruces de ámbito, revisiones caducadas, latencia y coste.
4. Descartar la variante si la ventaja depende del orden impuesto por oráculo, desaparece en la ruta completa, no mejora los aciertos netos por conversación o introduce un falso recuerdo nuevo. Cinco conversaciones pueden revelar fallos, no acreditar una tasa poblacional del 95 %.

La revisión humana y los sujetos nuevos son límites de evidencia, no autorización para reescribir el resultado ya observado ni para repetir variantes hasta obtener una cifra favorable.
