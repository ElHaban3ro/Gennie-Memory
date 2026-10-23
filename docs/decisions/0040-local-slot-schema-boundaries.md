# 0040 — Restringir el esquema compacto V3 desde señales locales de la consulta

## Contexto

Dos smokes compactos de un caso con pregunta y memoria sintéticas fallaron antes de seleccionar: primero el modelo emitió dos slots para una dimensión; después, al fijar la cantidad/IDs de slots, inventó una segunda ancla cuyo ID era el valor Northvale presente solo en el recuerdo. Una fixture alternativa “where does … work?” recibió desacuerdo ciego sobre si pedía ciudad o lugar de trabajo y se excluyó antes de la llamada remota.

## Decisión experimental

El esquema compacto V3 deriva de la consulta cantidad e IDs de slots, cardinalidad de anclas a partir de IDs literales con forma acotada, una cantidad de premisas para patrones explícitos de comunicación entre dos IDs, y los IDs permitidos de destino (`slot_n`, `premise_n` o vacío). Las citas de slot continúan siendo slices locales de la consulta. La ruta falla cerrada cuando la salida no satisface el esquema o la validación estructural.

La implementación conserva su alcance experimental. No cambia baseline ni las rutas anteriores. El reconocimiento de ID se limita a tokens ASCII con mayúscula, guion y dígito; el reconocimiento de premisa relacional es una heurística acotada en inglés/español. Ausencia de un patrón reconocido no significa que la consulta carezca semánticamente de premisas.

## Resultado y motivo

La revisión nueva obtuvo 0/1 selecciones V3 frente a 1/1 recuerdo seleccionado por la configuración vigente Decisions. La salida V3 encontró una cita literal relevante, pero la clasificó `unrelated` y la enlazó al slot. El rechazo es correcto; cambiar el binding o la relación localmente para forzar una selección no está justificado. Corrijo el reporte inicial: su primer control era Gemma + baseline (Parasail), no el proveedor predeterminado. El control real usa GPT-6 Luna Decisions (OpenAI). La comparación sintética de un caso enfrenta configuraciones completas —modelo, proveedor y política—; no atribuye causalidad a un solo componente ni mide calidad poblacional.

Esta revisión no supera calidad y no se adopta. No continuar con ajustes de formato campo por campo: una iteración futura debe cambiar la política/representación de forma completa, congelar contrato y preguntas nuevas con crítica ciega antes de gastar llamadas, y medirla frente al baseline con un control comparable.

## Evidencia y revisión

El detalle, costes, latencias, hashes, capturas y pruebas está en [resultados V3 de slots locales](../evaluation/slot-evidence-v3-local-query-slots-results-2026-10-23.md). Reabrir esta decisión solo con fixture independiente, crítica ciega, contrato de control adecuado y una señal de calidad que no requiera reparar falsedades por postproceso.
