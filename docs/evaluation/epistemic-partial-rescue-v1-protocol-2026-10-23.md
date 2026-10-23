# Rescate de estado epistémico parcial V1 — protocolo

Estado: **protocolo congelado antes de la campaña**. Es un experimento de desarrollo sobre V9 ya observado; no es validación confirmatoria. La política es optativa y no cambia `baseline`.

## Hipótesis

Las dos omisiones de Decisions en V9 son parciales explícitos: una estimación frente a una medición real y una hora prevista frente a una llegada registrada. Si `baseline` se abstiene, una segunda elección que pregunte únicamente por el estado explícito de ese mismo atributo podría recuperar esos recuerdos sin rescatar candidatos que solo comparten persona, evento o conversación.

## Mecanismo congelado

`epistemic_partial_rescue_v1` realiza primero la elección `baseline` actual sobre el mismo roster. Si selecciona un ID, devuelve esa decisión sin segunda petición. Si se abstiene, realiza una única elección adicional sobre el mismo query y roster, con una instrucción que exige mismo referente, mismo atributo y un estado provisional/pendiente/no registrado declarado expresamente. No se transforma un estimado en valor confirmado ni se infieren hechos ausentes. El proveedor es el mismo en ambas etapas. Las respuestas siguen siendo juicios de modelo, no prueba factual ni extracción de citas; el servicio valida IDs autorizados y agrega recibos.

La configuración vigente no cambia. El límite de campaña es 16 consultas; techo técnico de 32 peticiones HTTP (primera elección más, como máximo, un rescate por consulta), sin reintentos. Las etiquetas y el answer key no forman parte del payload.

## Banco y puertas previas

- 16 casos sintéticos V9, cuatro por relación, cuatro conversaciones; `cases_sha256=cbd0d2cb95f54b4351532a5e24abc8e004167459ef8611101fdbfa01f56b2581`.
- Revisión previa ciega de una vez por un agente Codex de la misma familia; no es revisión humana ni independencia entre familias.
- DB dedicada V9: `database_sha256=2fa0d0a7e46bff77b2b3b1970bbc1638439235dff47c615ef1c74970e1dd4b30`.
- Control Decisions + `baseline` congelado: 14/16, 2/4 parciales, 4/4 refutaciones y 4/4 abstenciones ajenas. Jev 1.13 no mejoró (12/16); no forma parte de esta candidata.
- El comando es una corrida única con Decisions, `development`, 16 consultas y diario nuevo. El techo orientativo de coste total es USD 0,001; duración 1–2 minutos. Los recibos reales prevalecen. API máximo esperado: 32 peticiones; cero reintentos.

## Puerta de desarrollo

Para pasar esta puerta, el resultado debe ser **16/16 exacto** contra la clave V9 congelada; debe recuperar explícitamente `v9_field_q03` y `v9_transit_q03`, preservar los 14 IDs seleccionados correctamente por el control original, mantener vacíos los cuatro negativos y conservar refutaciones y parciales correctos. Cero degradaciones y recibo de coste completo en las 16 operaciones. Coste total ≤1,5× el control y p95 de servicio ≤1,5× el control. Cualquier selección ajena, pérdida, degradación, falta de recibo o exceso de recurso rechaza la candidata; no se reescribe el prompt ni se repite sobre V9 para escoger otra respuesta.

Un pase solo permite pasar a una **cohorte nueva** disjunta, con crítica ciega anterior a llamadas y comparación emparejada contra Decisions + `baseline`. No valida población ni autoriza adopción. Un fallo conserva el diario, deja `baseline` vigente y exige una hipótesis distinta con casos nuevos.
