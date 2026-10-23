# Protocolo de confirmación V14 para single-pass V1

Estado: congelado antes de inferencia. Esta es la única confirmación posterior al resultado exploratorio V11. V12 y V13 se descartan antes de llamadas por defectos de etiquetas/pregunta documentados; no se harán más cohortes dentro de este objetivo si V14 no confirma la mejora.

## Integridad y revisión

V14 contiene 16 casos sintéticos nuevos en cuatro ámbitos: cuatro apoyos, cuatro refutaciones, cuatro hechos parciales/provisionales y cuatro consultas sin respuesta. Sus hashes se fijan en [`assets/quality-goal-confirm-v14/hashes.json`](assets/quality-goal-confirm-v14/hashes.json). Un subagente revisó a ciegas solo `blind-review.jsonl`, SHA-256 `e42d2bb131dc5e1c6027420c64c6be25a96ccba5627cdf2890e3aa5c0136fc67`, y declaró aptas las 16 relaciones; véase [adjudicación](assets/quality-goal-confirm-v14/blind-adjudication.md). La revisión no constituye validación humana ni independencia de familia de modelos.

Antes de las llamadas, un intento de campaña se rechazó localmente por el literal de procedencia `independent_blind_synthetic_adjudication`. Se normalizó a `blind_synthetic_adjudication`; el único cambio fue ese metadato contractual en consultas/casos, se recalcularon hashes y se repitió la búsqueda. La pregunta, corpus, clave y archivo ciego no cambiaron. El intento rechazado no generó peticiones HTTP.

El preflight local debe recuperar los 12 IDs útiles en top-5/top-20 y conservar candidatos para las cuatro negativas. DB, binario, roster y hash de casos se registran en los informes. La clave y las etiquetas no se envían al proveedor.

## Gate emparejado, congelado

Ejecutar primero Decisions con `baseline` sobre los 16 casos, un intento por caso, sin reintentos. Si baseline obtiene 16/16, detener: la candidata no se llama porque no hay mejora observable y queda rechazada para adopción. Si baseline falla algún caso, ejecutar exactamente 16 casos con la política congelada `epistemic_partial_single_pass_v1`, misma DB, binario, proveedor y orden. Máximo 32 peticiones HTTP para la tanda, presupuesto combinado ≤USD 0,003. Registrar recibos por caso; una llamada incierta no se repite.

La candidata solo confirma si logra 16/16, corrige al menos un fallo de baseline, conserva todos los IDs correctos de baseline, produce cuatro negativos vacíos, añade cero IDs ajenos/degradaciones, tiene recibos completos y su coste informado y p95 de servicio son cada uno ≤1,5× baseline. Cualquier empate, regresión, error técnico, recibo ausente o límite excedido rechaza la candidata. No se retocan prompt, datos, política ni gold después de las llamadas, y no se abre otra variante en este objetivo.

La confirmación es sintética y acotada; no establece tasa de producción ni verdad externa. Aun si pasa, la promoción se limita a habilitar la política ya implementada como opción explícita del runtime; no altera el proveedor ni la política predeterminados.
