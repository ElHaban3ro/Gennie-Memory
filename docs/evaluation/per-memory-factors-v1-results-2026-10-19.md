# Factores por recuerdo: corrección conseguida, dos pérdidas y coste excesivo

Estado2026-10-19: desarrollo sintético conocido, sin promoción. [Protocolo fijado antes de llamadas](per-memory-factors-v1-protocol-2026-10-19.md).

`per_memory_factors_v1` pide tres choices independientes por cada top8recuerdo en una petición Jev1.13. El modelo juzga contexto/referente, dato solicitado y corrección de premisa. Retrieval combina contexto&&(dato||corrección), preserva todos los positivos y revalida ID/revisión/cardinalidad/agregación. La traza factor_checks expone esos juicios; son fallables y no constituyen prueba de verdad. Un parser inválido falla atómicamente y conserva consumo conocido.102pruebas workspace pasan; no hay cambios SQL, dependencias, secretos, servidores o defaults.

## Resultado medido

Diez pares conocidos/20operaciones, primero2para formato y18restantes en el mismo diario, sin repetir. Todas completaron con recibos; auditoría ordinaria y composición JSON válidas. Se mantuvieron consultas, gold y candidatos de cada par.

| Medición | ControlV5 | Tres factores |
| --- | ---: | ---: |
| Operaciones |10|10|
| IDs no permitidos seleccionados |1|0|
| IDs útiles del control perdidos por candidata |—|2|
| Degradaciones |0|0|
| Coste reportado USD |0,000566076|0,001053402|
| Mediana del runner ms |353,60|374,03|
| p95 del runner ms |1556,21|521,15|

La candidata eliminó el carné `cev5_071_03`, conservando la negación de avalista correcta `cev5_071_08`. No ganó nuevos IDs útiles. Perdió dos ya seleccionados por el control:

- `cev5q_073_03`: pregunta por acometida de red; `cev5_073_06` niega uso de red y aporta depósito de lluvia. El modelo reconoció dato y corrección, pero marcó contexto falso y vetó el recuerdo.
- `cev5q_070_05`: pregunta por grados seco y húmedo; `cev5_070_10` aporta seco4y declara húmedo pendiente. El modelo reconoció dato solicitado, pero marcó contexto falso y lo vetó.

Ambas pérdidas afectan evidencia directa, no los fragmentos de preparación discutidos en la auditoría anterior. No se retiraron etiquetas para aprobar. **Calidad falla** por esas pérdidas. Coste1,8609×>1,5×: **recursos fallan**. La cola menor de esta muestra pequeña no compensa pérdidas/coste ni demuestra latencia poblacional. Esto no es contraste completo contra Decisions/baseline ni respuesta final de Codex.

## Diagnóstico retrospectivo separado

Se recombinaron los factores ya observados usando dato||corrección sin el veto de contexto: sobre estas mismas diez consultas, recuperaría ambos IDs útiles y mantendría la eliminación del carné, sin nuevos IDs no permitidos. Es una ablación posterior a observar resultados: **cero llamadas nuevas, sin cambio al reporte original, sin validación nueva ni ahorro medido**. No demuestra que omitir identidad sea seguro fuera del piloto.

La causa observada es un veto de contexto equivocado, no ausencia del dato en la búsqueda. Una próxima versión deberá comprobar el referente dentro de cada vía de dato/corrección, preservando premisas corregibles y partes incompletas. Reducir la duplicación de preguntas es una hipótesis de coste que requiere medición. No fijar identidad como verdadera ni presentar la recomposición retrospectiva como política ya probada.

[Reporte Rust](../../outputs/iromem/evaluation/per-memory-factors-v1-pilot-2026-10-19/report.json), [ablación diagnóstica](../../outputs/iromem/evaluation/per-memory-factors-v1-pilot-2026-10-19/retrospective-context-ablation.json), [auditoría](../../outputs/iromem/evaluation/per-memory-factors-v1-pilot-2026-10-19/audit.json).

Coste nuevoUSD0,001619478, recibos completos. Acumulado2605operaciones/17campañas emparejadas, USD0,183412894 conocidos más dos operaciones de coste desconocido. Excluye Codex. Los240juicios de factores de la candidata están agrupados en diez consultas y no se presentan como240observaciones independientes. DB temporal propia eliminada, dashboard intacto, V6congelada sin llamadas. No se amplía esta versión; el objetivo sigue activo y faltan desarrollo aprobado, validación nueva y adopción con mejora demostrada.
