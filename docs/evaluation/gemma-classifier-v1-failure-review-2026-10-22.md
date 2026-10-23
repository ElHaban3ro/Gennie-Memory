# Gemma clasificador V1: revisión breve del fallo observado

Fecha: 2026-10-22. Revisión estática de `inputs.jsonl`, `requests.jsonl`, los cuerpos originales de `journal.jsonl` y `report.json` en `outputs/iromem/evaluation/contribution-gemma-minimal-v1-2026-10-22/`. Sin nuevas llamadas, código o pruebas.

**Hay fallo semántico observado antes del429, independiente del fallo de transporte.** Las dos primeras respuestas son técnicamente válidas: modelo `google/gemma-3-4b-it`, proveedor Parasail, finalización `stop`, JSON y cardinalidad correctos, recibos completos y `validation_error:null`. Sus ocho juicios contienen tres discrepancias frente al gold congelado: una selección ajena y dos pérdidas. La tercera petición terminó HTTP429sin recibo; se detuvo sin reintentar. No atribuir los errores anteriores al429ni calcular una tasa global sobre las16preguntas.

| Caso y fuente | Gold / salida real | Evidencia del desacuerdo |
| --- | --- | --- |
| `cv3q_01`, `cv3_01_01`: «El teléfono de contacto del pedido O7 de Darsa no consta en el registro.» | A / none | Se pregunta por ese teléfono, pedido y titular. La instrucción A incluye expresamente estado desconocido/no registrado aunque falte el valor. La fuente aporta ese estado documental, sin afirmar inexistencia del teléfono. Se perdió evidencia solicitada. |
| `cv3q_01`, `cv3_01_03`: «El pedido O7 de Darsa se recoge en la ventanilla este.» | none / A | Mismo pedido, atributo distinto: lugar de recogida frente a teléfono. La instrucción preserva sujeto y atributo y excluye otro atributo. Elegir esta fuente como A transfiere pertinencia temática a aporte solicitado. No hay valor ni estado de teléfono en ese texto. |
| `cv3q_02`, `cv3_02_01`: «En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.» | B / none | La pregunta presupone un tutor que deba acompañar a Ina para matricularse. Bajo la lectura declarada del banco, la eliminación del requisito y suficiencia de la credencial refutan esa necesidad. Se perdió la refutación anotada; las cuatro respuestas del caso fueron none. |

Los otros cinco juicios válidos fueron none correctos. En particular, el modelo sí excluyó el teléfono de otro pedido y el requisito de presentar credencial por sí solo. Esto acota el síntoma; no demuestra que aplique establemente identidad o compatibilidad de requisitos. En el primer caso produjo exactamente `[none,none,none,A]`; en el segundo `[none,none,none,none]`. No hubo truncamiento, citas generadas o remapeo de IDs que explique esas elecciones: el payload conserva los cuatro textos en el mismo orden que gold y cuerpo.

## Ambigüedad y límites causales

Los dos errores del teléfono bastan para incumplir la puerta de calidad de esta versión. No encuentro una contradicción concreta entre su rúbrica y las fuentes: la regla A permite explícitamente estados desconocidos y el lugar de recogida es otro atributo. Una objeción al caso Faro no rescataría ese fallo independiente.

Faro tiene cautelas lingüísticas documentadas en otros experimentos: [control/finalidad y scope normativo](documentary-v3-faro-control-contract-critique-2026-10-22.md) y [construcciones nominales del contrato documental](documentary-v3-coherent116-contract-adjudication-2026-10-22.md). Esas discusiones distinguen representar toda lectura, resolver al agente de «matricularse», identificar autoridad normativa y conservar «obligación de tutor». No constituyen una nueva adjudicación de este gold ni demuestran que `none` sea correcto en esta tarea. Aquí fuente y pregunta íntegros se clasifican bajo la lectura de requisito anotada antes de la llamada; no se exige compilar un frame documental ni resolver todos esos campos. Mantener B como esperado histórico y registrar cualquier objeción por separado, sin cambiar el resultado. Tampoco presentar esta revisión como prueba de verdad universal de una lectura única del español.

Es compatible con errores de asociación de atributo y conservación de estado/premisa. **No identifica por qué ocurrieron**: capacidad del modelo, comprensión del español, seguimiento de instrucciones, formato o efectos de contexto son hipótesis no aisladas. No hay razonamiento observable que permita elegir entre ellas; ambas respuestas declaran cero tokens de razonamiento y no incluyen explicación. El fingerprint de servicio tampoco acredita pesos inmutables. El ensayo no demuestra incapacidad general de modelos4B, de otro proveedor o de todo recuperador textual.

## Decisión y contabilidad

La versión no se amplía: fallo de calidad ya observado, mínimo incompleto y recursos no acreditados. Son3HTTP iniciados/terminados:2respuestas válidas,1fallo técnico y13preguntas no ejecutadas. El caso03no es ausencia correcta ni otro error semántico evaluable. Coste conocidoUSD0,000033370más1operación de importe desconocido por429; no declarar el coste total ni el ratio de recursos completo. Los tiempos de dos respuestas no permiten inferir p95 del recorrido o estabilidad.

El reporte `screen_passed:false` es coherente con estos hechos. No se modifica gold, históricos, la candidata documental64/116FAILED, all116, baseline o la reserva. Esta revisión confirma el descarte de la versión observada; no propone otra campaña ni convierte una corrida incompleta en resultado global64juicios.
