# Protocolo: comparación prospectiva de alcance de sujeto V3

Fecha: 2026-10-23. Estado: preregistrado; pendiente de ejecución remota.

## Pregunta

¿La ruta `slot_evidence_v3`, con el validador local actual, conserva respuestas literales cuando una memoria incluye hechos de varias personas y evita seleccionar una memoria cuyo predicado pedido pertenece explícitamente a otra persona, frente al clasificador `baseline` vigente?

## Cohorte congelada

La entrada sin etiquetas es `tests/fixtures/slot-evidence-v3-sentence-scope-prospective-v1.json` (diez casos sintéticos revisados a ciegas por dos lectores; coincidencia 10/10). Para esta comparación se incluyen nueve: siete consultables y dos sin respuesta para la identidad preguntada. Se excluye `sentence-08`: contiene una ubicación pasada de Noor que un lector puede considerar evidencia parcial para una consulta actual, mientras el harness solo distingue selección correcta y abstención; puntuarla como negativo sería engañoso. Los otros nueve conservan exactamente pregunta y resumen. Cada consulta usa un ámbito SQLite aislado con una sola memoria candidata.

Los IDs esperados y prohibidos se guardan aparte de la pregunta y resumen originales en `assets/slot-evidence-v3/prospective-sentence-scope-v1/cases.jsonl`; el corpus se guarda en `corpus.json`. El harness construye el request de runtime exclusivamente de `request.search` y `provider`, omitiendo etiquetas, segmento y racional. Procedencia: `blind_synthetic_adjudication`. SHA-256 de casos: `98282b282661c496b2674cb9c00b156abd8db0d2887662d7f01853467d6b273f`; corpus: `5bc12f6e2e12d364aaf1a1cb9b78b0cf5265bb92692416b6f0263ddd694605aa`.

## Brazos y gates

- `gemma3_4b` + `slot_evidence_v3`, primero tres casos, en orden fijo. Deben ser tres selecciones completas correctas consecutivas, sin degradaciones ni fugas de IDs prohibidos. Si falla cualquiera, se detiene V3 y no se reintenta.
- Solo si el gate anterior pasa, se completa V3 sobre los seis casos restantes.
- `gemma3_4b` + `baseline` se ejecuta sobre los nueve casos como control emparejado por pregunta y memoria. El baseline no se modifica.

Máximo: 18 llamadas HTTP; sin retries. Estimación previa: USD 0,002 como límite operativo del experimento, basada en recibos previos de este mismo modelo/ruta; objetivo de duración 2–4 minutos. Se registrarán uso/coste informados, latencia de clasificador, total del servicio, búsquedas, selecciones, errores estructurales y hashes. El coste informado puede no estar presente en una respuesta; los valores ausentes quedan desconocidos, no se convierten en cero.

## Interpretación

El resultado de cada brazo se informará por separado y emparejado, con positivos, negativos, degradaciones y fugas. Una traza manual/estructural no cuenta como extracción; aquí solo cuenta el recorrido de runtime completo. n=9 es un holdout sintético de desarrollo y no basta por sí solo para adopción poblacional. Cualquier regresión frente a baseline o degradación del gate bloquea V3. No se cambia política predeterminada, prompt, etiquetas ni baseline durante la campaña.
