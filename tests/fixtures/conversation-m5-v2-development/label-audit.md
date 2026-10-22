# Auditoría interna de formulación, fuera de puntuación

Estado: una sola autoría de agente, sin revisión humana ni ejecución del modelo para corregir etiquetas. Cada positivo puntuado se redactó para referirse a un solo hecho del ámbito. Los casos siguientes se **excluyeron** del JSONL puntuado porque admitirían varias respuestas o una inferencia que el corpus no respalda:

| Ámbito | Pregunta excluida | Motivo |
| --- | --- | --- |
| 026, herbario | «¿Cómo se prepararon las muestras?» | Podría contestarse con el sobre del musgo, el secado de hojas o el cartón de prensado. |
| 027, radio | «¿Qué formato tendrá el episodio?» | Podría aludir a duración, canal de publicación o códec del archivo maestro. |
| 028, cocina | «¿Dónde se guardarán los alimentos?» | El refrigerador, la despensa y el depósito aparecen en recuerdos distintos. |
| 029, mapas | «¿Cómo se presentarán los mapas?» | El marco, la iluminación, las reproducciones y la audioguía son respuestas plausibles. |
| 030, aves | «¿Cómo se registrarán las observaciones?» | La hoja de avistamientos, el archivo local, las fotos y las fichas ofrecen respuestas parciales. |

Las negativas no prueban ausencia en el mundo: solo que el dato pedido no está documentado por estos 30 recuerdos sintéticos del ámbito. La autoría y esta revisión son del mismo agente; una segunda lectura crítica podría descubrir ambigüedades adicionales. Cualquier corrección posterior debe quedar versionada y no transformarse en una mejora retrospectiva de una medición ya vista.

## Correcciones antes de medir

- La pregunta positiva `cm5v2q_5a85ede9c7777afab0b7` (radio, `quote_cut`) pedía identificar una frase literal, pero el recuerdo solo menciona su tema. Ahora pregunta «¿Sobre qué tema trataba la frase que se acordó conservar íntegra?»; el ID esperado y el hecho no cambiaron.
- Dos negativos por ámbito se acercaron a recuerdos existentes. En cinco pares de recuerdos de contexto se añadió información provisional o de solicitud, sin afirmar precio final, fecha confirmada, contrato, licencia concedida, compra adjudicada, póliza firmada, venta, conteo final o permiso otorgado. Las preguntas negativas distinguen expresamente ese estado del dato ausente.
- Antes de ejecutar búsqueda o clasificador se barajaron recuerdos y casos según el método y las semillas del README. Esta auditoría sigue siendo interna del mismo agente y no resuelve el riesgo de ambigüedades inadvertidas.
