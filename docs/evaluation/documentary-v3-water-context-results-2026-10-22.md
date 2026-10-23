# Contexto de suministro: mejora del banco completo — 2026-10-22

El comparador experimental mejora **24/39→27/39 evidencias útiles (61,5 %→69,2 %)** sobre los mismos 100 pares congelados. Conserva todas las selecciones anteriores y mantiene **cero selecciones ajenas de 60**. Quedan 12 omisiones útiles; una pregunta ambigua continúa separada. No supera la puerta completa 39/39 y no se adopta en producto.

| Medición | Antes | Después |
| --- | --- | --- |
| Evidencias útiles seleccionadas | 24/39 | 27/39 |
| Selecciones ajenas | 0/60 | 0/60 |
| Pérdidas anteriores | — | 0 |
| Respuestas completas | 0 | 0 |
| Selecciones solo condicionales | 1 | 1 |
| Pruebas workspace | — | 726 aprobadas; cero fallos; 24 ignoradas |
| Informes históricos reproducidos | — | 226/226 idénticos |
| Controles críticos principales | — | 49/49 |
| Controles adicionales de alias en rama | — | 2/2 |

## Qué cambió

Las ganancias corresponden a Nardo, Azucena y Olmo: `cv3q_05/cv3_05_00`, `cv3q_07/cv3_07_00` y `pcv1_08_01/claim_01`. La fuente aporta un depósito que alimenta el destino, un suministro inmediato o una relación aguas arriba de ese depósito. El comparador anterior exigía resolver identidad anónima, uso para riego y contexto de la cadena antes de conservar información parcial. No había que afirmar una tubería final ni dar por verdadero el propósito ausente.

La ruta nueva usa únicamente los enlaces y perfiles declarados en el vocabulario. Comprueba el destino nombrado y sus restricciones, vincula los operandos dentro de su propia afirmación y conserva las proposiciones, ámbitos, calificadores y premisas originales. Una corroboración aguas arriba exige los bindings exactos declarados y su propio contexto compatible; no se convierte en origen del destino final.

Los términos y proposiciones de respuesta y los bindings de salida permanecen vacíos. Se mantienen pendientes identidad, dato solicitado, origen y, cuando corresponde, propósito, premisa y fundamento. El filtro posterior de patrones de respuesta reconoce esta ruta exclusivamente como contexto parcial con esos campos vacíos y sin inferencia de origen; las respuestas conservan sus comprobaciones habituales.

No hubo nueva extracción ni sustitución de frames: los textos, raws, vocabularios, etiquetas y regla de selección en todas las lecturas son los mismos. La mejora del piloto de retención 0→3/3 sigue siendo un resultado separado y no se suma a 27/39.

## Fallos detectados y conservados

La prueba inicial con stub reproduce las tres omisiones. La primera implementación generaba el contexto, pero el filtro posterior lo descartaba por no proporcionar una tubería: ese fallo queda preservado antes de la integración.

La crítica independiente encontró ocho resultados incorrectos en versiones intermedias: tres contradicciones locales no bloqueadas (suministro, alimentación y corroboración), dos perfiles insuficientes aceptados, un positivo rechazado al permutar bindings y dos alias cuyo nombre o restricciones procedían de una rama. Todos tienen entradas válidas y ejecuciones previas conservadas. La versión final comprueba contradicciones locales compatibles, perfiles completos, bindings como conjunto y ámbitos de todos los alias. No se eliminó ningún contraejemplo ni se modificaron oráculos después de observar el nuevo resultado.

Los 49 controles principales pasan; sus 11 testigos nuevos conservan exactamente el payload fuente y las deudas, con respuestas vacías. Los dos controles separados de alias en rama también pasan. Son casos conocidos y mutaciones estructurales de la misma familia Codex: no demuestran extracción fiel ni generalización. La exploración adicional de binders queda fuera del denominador principal; no se atribuye cobertura ejecutada a toda combinación posible.

El caso ambiguo `pcv1_08_01/claim_00` ahora conserva el proveedor inmediato como contexto. Sigue excluido de útiles y ajenos; no se cambia su etiqueta. La auditoría retrospectiva comprueba que no responde una tubería ni refuta el posible origen aguas arriba. Es una revisión del payload, no una nueva anotación ciega de utilidad.

## Tiempos, coste y alcance

En esta comparación emparejada, p50 CLI fue **21,699→21,958 ms** y p95 **44,631→42,287 ms**. Hubo verificaciones locales concurrentes. Son observaciones descriptivas del comparador, sin acreditar mejora de recursos ni latencia de producto; excluyen extracción, crítica, red y entrega a Codex.

**Cero llamadas externas nuevas; USD 0 externos nuevos.** Coste y tokens facturables Codex desconocidos. Acumulado externo conservado: 6.129 operaciones, 23 campañas, USD 4,565416508 conocidos y 20 operaciones con importe desconocido, sin Codex.

La implementación solo pertenece a evaluación. Default de producto, README, dashboard, datos y credenciales permanecen sin cambios. No se abrió V6. La validación local completa aún falla; comparación contemporánea con producto, recursos completos, estabilidad y casos nuevos con revisión ciega siguen pendientes antes de adoptar.

## Próximo mecanismo preparado

El caso Duna necesita declarar explícitamente relevancia `admits→property`, sin equiparar `property` con `quantity`. La propuesta V2 pasó crítica separada tras corregir dos carencias: preservar las restricciones de `described_ref` y comprobar el dominio de todos los miembros y alias. Sus 24 familias de controles aún son propuestas, no fixtures ejecutados. No hay implementación ni ganancia atribuida a ese contrato. La siguiente acción es concretarlos y congelarlos antes del código.

## Evidencia reproducible

Bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`:

- `water-operand-context-v1/root-baseline/`: reproducción previa y binario inicial.
- `water-operand-context-v1/implementation/`: contrato, addendum y fuentes fallidas preservadas.
- `water-operand-context-v1/critical-review/critical-review-v1.md`: controles congelados, fallos y revisión final.
- `water-operand-context-v1/known100-comparison-v2/report.json`: comparación completa con el binario final; la primera ejecución queda en `known100-comparison/`.
- `water-operand-context-v1/historical-compatibility-v2/report.json`: 226 informes idénticos.
- `water-operand-context-v1/root-verification/`: fuentes, fixtures y logs finales.
- `water-operand-context-v1/ambiguous-row-retrospective-audit.json`: caso ambiguo sin reetiquetar.
- `admission-property-contract-v1/candidate-v2/` y `critical-review/review-v2.md`: siguiente contrato, sin implementar.

Binario final SHA-256: `74ce25d59053dd3223e7ba57951dc37579a3129369101b88dbc953c39397e51f`.
