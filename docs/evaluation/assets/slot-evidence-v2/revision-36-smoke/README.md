# Revisión 36 — cita conjunta fallida

Caso nuevo, críticos ciegos: respuesta directa; FTS recuperó 1/1 en top-1. Con el esquema de `anchor_quote` conjunto, una llamada HTTP200/Parasail a Gemma3 4B falló con `slot_evidence_v2_anchor_phrase_not_exact`; ninguna selección. Recibo:645/202 tokens, USD0,00003590; clasificador2.693,110ms, total2.694,245ms. No hay raw; no conocemos la cita propuesta. Se detuvo sin reintentos. Hashes pre-llamada y captura en `manifest-preflight.json`/`manifest.json`.

La revisión local derivaba el tipo solo de un prefijo verificado y exigía nombre e ID dentro de la misma frase. El modelo no satisfizo ese contrato en este caso. No se relajó la aceptación. No es mejora semántica; r31–r36 siguen sin pasar smoke válido.
