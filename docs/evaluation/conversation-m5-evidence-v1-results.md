# M5: resultado de la variante de selección por evidencia

> Corrección de cierre, 2026-10-17: se conservan el protocolo, los datos y las puntuaciones históricas. Las selecciones discordantes con una etiqueta negativa no prueban una afirmación falsa; algunas preservan evidencia que refuta la pregunta. Las conclusiones de mejora de verdad o superioridad derivadas solo de esa abstención quedan retiradas. Véase el [cierre de M5](m5-closeout-2026-10-17.md).

Estado: corrida de desarrollo del 2026-10-17. La [política y el criterio de descarte](conversation-m5-evidence-v1-protocol.md) se fijaron antes de llamar al proveedor. Los [diarios, reportes y comparación emparejada](../../outputs/iromem/evaluation/conversation-m5-evidence-v1-2026-10-17/) conservan respuestas y hashes. El banco tiene etiquetas `agent_synthetic_unreviewed`; los casos que motivaron la variante ya eran conocidos, así que esta corrida no es validación independiente.

Se reconstruyó una SQLite temporal con los mismos 150 recuerdos. Las **100/100 listas de candidatos fueron idénticas** a las del diario Decisions base, incluido contenido y orden. La única diferencia de inferencia fue `selection_policy: evidence_v1`, que cambia las instrucciones de la misma elección tipada; `baseline` sigue siendo el valor predeterminado.

| Métrica | Decisions base | `evidence_v1` |
| --- | ---: | ---: |
| Casos correctos | 96/100 | 99/100 |
| Positivos correctos | 74/75 | 75/75 |
| Negativos vacíos | 22/25 | 24/25 |
| Fallos degradados | 1 | 0 |
| Tiempo total p50 / p95 | 368,8 / 555,6 ms | 350,9 / 425,7 ms |
| Tokens de entrada p50 informados | 278 | 323 |
| Coste informado | US$0,0032522 en 97 casos | US$0,0037181 en 98 casos |

La subida de tres aciertos se compone de **dos correcciones de selección** y de un positivo cuyo fallo anterior fue `transport_error` y que respondió en la corrida nueva. Sobre los 99 casos que terminaron completos en la base, el resultado pasó de **96/99 a 98/99**. No se reintentó la llamada incierta del diario anterior: la nueva variante realizó una llamada distinta registrada en su propio diario. La mediana de tokens de entrada subió 45; los costes totales abarcan distinto número de respuestas reportadas. Los percentiles de dos campañas de red separadas no demuestran una mejora causal de latencia.

`evidence_v1` se abstuvo ante el recuento preliminar de aves y la consulta de licencia; Decisions base había seleccionado esas dos memorias de contexto parcial. En la tercera, ambos seleccionaron la memoria que refuta una adjudicación **en esa reunión**, sin establecer si hubo decisión posterior. La [auditoría retrospectiva ciega](conversation-m5-v2-relation-audit-2026-10-23.md) conserva el gold original y clasifica los tres casos según las cuatro relaciones; por ello, no se presenta la abstención como mejora comprobada ni el caso restante como falso recuerdo. La variante sigue descartada como candidata: su comparación conocida no establece que mejore calidad bajo el contrato de apoyo/refutación/parcialidad, ni permite promoción.

El fallo residual acota la siguiente investigación: una instrucción de evidencia más explícita no garantiza abstención cuando el texto del candidato niega el resultado solicitado. Una nueva hipótesis necesitaría casos y conversaciones de desarrollo nuevos antes de volver a medir, para no adaptar instrucciones repetidamente a los mismos tres errores. La calibración y la reserva actuales no se han abierto, y falta revisión independiente de preguntas naturales y etiquetas.

Verificación del cambio: `cargo test --offline --workspace` pasó (27 pruebas unitarias), `cargo fmt --all --check` y `git diff --check` pasaron, y `validate-conversation-m5-development` validó los cuatro hashes y los 100 casos. La prueba local confirmó que la política omitida sigue siendo `baseline` y que la opción explícita llega al clasificador. No se ejecutó un servicio HTTP ni se cambió la base del usuario.
