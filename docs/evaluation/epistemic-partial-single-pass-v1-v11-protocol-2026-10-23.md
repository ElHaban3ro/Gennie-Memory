# Protocolo de desarrollo V11 para single-pass V1

Estado: congelado antes de llamadas remotas. V11 es una cohorte sintética nueva, disjunta de V9/V10, con cuatro relaciones y cuatro consultas por relación. La política y su instrucción ya están congeladas en el código; no se modifican durante este contraste.

## Revisión y hashes

El subagente recibió solo `assets/quality-goal-dev-v11/blind-review.jsonl`, SHA-256 `cab084dfc60aa1d58e61a617f890ded749df66682354193e680f9dd5556aabf6`. El dictamen final confirma cuatro apoyos, refutaciones, parciales y negativos near-miss sin ambigüedad material; [detalle](assets/quality-goal-dev-v11/blind-adjudication.md). No hubo proveedor ni inferencia durante la revisión. Los hashes de corpus, preguntas, clave y campaña están en [`assets/quality-goal-dev-v11/hashes.json`](assets/quality-goal-dev-v11/hashes.json).

## Hipótesis

`epistemic_partial_single_pass_v1` podría rescatar información provisional sobre el mismo atributo sin confundirla con un resultado confirmado ni aceptar datos de otro atributo del mismo objeto. Usa una sola llamada con Decisions y una instrucción congelada. V10 empató con baseline en el techo de 16/16, por eso V11 dirige la prueba a parciales realistas y negativos near-miss. No se retoca el prompt con V10 ni V11.

## Gate local y ejecución condicional

El preflight de búsqueda debe recuperar los 12 IDs útiles antes del proveedor; esta operación no crea evidencia. Después ejecutar Decisions `baseline` en 16 casos. Si baseline logra 16/16, no se ejecuta candidata: no hay margen de mejora observable y el resultado es no-go sin gasto adicional. Si baseline deja algún fallo, correr exactamente 16 casos de `epistemic_partial_single_pass_v1`, con mismo roster y DB. Etiquetas y clave nunca entran en los mensajes. Máximo 32 HTTP contando ambos brazos, cero reintentos, presupuesto combinado ≤USD 0,003.

La candidata solo evidencia mejora si corrige al menos un fallo de baseline y alcanza 16/16: cuatro apoyos, refutaciones y parciales correctos; cuatro negativos vacíos; cero IDs baseline correctos perdidos, cero ajenos, cero degradaciones y recibos completos. Coste y p95 de servicio deben ser ≤1,5× baseline. Si empata, introduce cualquier daño, excede recursos o falta recibo, se rechaza sin otra ejecución en V11. Aun pasando, esto solo permitiría preparar confirmación independiente; no autoriza adopción por sí solo.
