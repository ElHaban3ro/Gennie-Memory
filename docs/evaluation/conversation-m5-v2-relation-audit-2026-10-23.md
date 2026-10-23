# Auditoría retrospectiva de relaciones en M5 v2 — 2026-10-23

## Alcance

Se revisaron a ciegas los tres casos que el informe M5 v2 había descrito como falsos recuerdos seleccionados por Decisions. Dos revisores Codex recibieron solo cada pregunta y las memorias pertinentes: no recibieron etiquetas esperadas, resultados del clasificador ni el dictamen del otro. Coincidieron en los tres casos. Son revisiones separadas de la misma familia de modelos, no una revisión humana ni una nueva estimación del rendimiento.

Las etiquetas, manifiestos, diarios y métricas originales se conservan sin modificación. Este anexo aplica la distinción entre respuesta, refutación y contexto parcial sin reescribir el gold observado. No hubo llamadas API (USD 0); el uso/coste de Codex no está disponible en el recibo de estas revisiones.

## Dictámenes ciegos

| Caso | Memoria que eligió Decisions base | Relación retrospectiva | Qué permite decir | Qué no permite decir |
|---|---|---|---|---|
| `cm5v2q_ee0d52649ceeffc138e1` | `cm5v2_da7230b8c12599e52561`: «…dos aves migratorias en un conteo preliminar» | `pending_or_partial` | La única cifra disponible pertenece a un conteo preliminar. | No establece el resultado del informe final ni que ese informe carezca de otra cifra. |
| `cm5v2q_83eac9f1cc94898beab7` | `cm5v2_3ca079a84f414caa9ba8`: «…solicitó información sobre una licencia de emisión comercial» | `pending_or_partial` | La conversación documenta una consulta sobre la licencia comercial; es contexto para distinguirla de la licencia CC BY de la música. | No establece que se haya concedido una licencia comercial ni cuál sería. |
| `cm5v2q_c8df100909c8daf7fbc0` | `cm5v2_2155cb370088eaa59682`: «…se compararon dos proveedores…, sin seleccionar uno en esa reunión» | `refutes_premise`, con alcance temporal acotado | La memoria refuta que hubiera un proveedor seleccionado **en esa reunión**. | No permite concluir que nunca se eligió proveedor o que no hubo decisión posterior. |

En los tres pares la memoria seleccionada es pertinente bajo el contrato de cuatro relaciones; ninguna permite responder como dato final sin conservar su estado o límite. La selección de la tercera memoria es evidencia correctiva útil, no prueba de quién ganó.

## Consecuencia para la comparación histórica

El informe original midió 96/100 para Decisions base y 99/100 para `evidence_v1` contra etiquetas binarias que asignaban `expected_ids: []` a estos negativos. Esa puntuación se preserva como concordancia con el gold original, que figura como no revisado. La lectura retrospectiva no cambia el denominador ni autoriza a recalcular el resultado global con solo tres casos.

En los dos primeros casos, `evidence_v1` devolvió `none`; por tanto, no se puede presentar su subida histórica como una mejora de conservación de evidencia parcial. En el tercero, `evidence_v1` conservó el recuerdo que refuta la adjudicación en esa reunión. La comparación conocida no demuestra que la variante mejore la calidad según el contrato actualizado; tampoco demuestra el comportamiento en otras consultas.

## Siguiente trabajo

No ajustar otra instrucción con estos tres casos. La siguiente evaluación de calidad debe usar casos nuevos redactados con las cuatro relaciones antes de puntuar, revisión ciega de fuente/pregunta y comparación con Decisions vigente. Las mejoras locales deben conservar apoyo, refutación y contexto parcial; una abstención solo cuenta como correcta cuando la evidencia realmente es irrelevante o no existe dentro del alcance declarado.
