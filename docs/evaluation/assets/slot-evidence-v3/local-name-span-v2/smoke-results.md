# Smoke V2: separación explícita de cita de memoria y pregunta

Fecha: 2026-10-23. El caso es sintético y distinto al de V1. Dos lectores ciegos coincidieron (2/2): la respuesta es Port Arlen y Bellford pertenece a Tomas Rell T-28, no a Inez Vale. El manifiesto y hashes congelados están en `manifest-preflight.json`.

## Resultado

La primera llamada se detuvo antes de puntuación, según la regla de parada. El transporte tuvo HTTP 200 y recibo completo de DeepInfra para `google/gemma-3-4b-it`: 1.070 tokens de entrada, 185 de salida, USD 0,000072, 2.769 ms de transporte y 2.786 ms de clasificación. La respuesta capturada fue completa; los dos JSONL se guardaron con permisos `0600`.

La ancla y la cita de proposición superaron esta etapa: el modelo devolvió `Marine biologist Inez Vale M-63` y citó literalmente la oración fuente. La ejecución falló en el slot con `slot_evidence_v3_slot_quote_not_literal`: para la pregunta “Which city …”, el modelo emitió `from Seagrass Research Pier`, texto tomado del recuerdo, como cita del slot que debe proceder de la pregunta. No hubo selección; no se hicieron las otras dos invocaciones y no se reintentó.

La iteración del prompt de V1 sí corrigió la clase anterior de error en la cita de proposición en este caso, pero apareció una confusión distinta en la cita del slot. Por tanto no se puede atribuir una mejora global ni puntuable al selector. La nueva revisión no supera la puerta.

## Coste

- Esta llamada fallida: USD 0,000072; 1.255 tokens; 2.786 ms de clasificación.
- Dos llamadas restantes: no realizadas.
- Acumulado de la revisión V2: USD 0,000072, 1 llamada.
- Coste Codex: no disponible.

No se ejecutó comparación baseline ni cohorte. No mezclar con el historial V3 anterior.
