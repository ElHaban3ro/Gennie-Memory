# Extracción documental V3: piloto nativo con OpenAI

UTC 2026-10-20. Se conserva íntegra la evidencia negativa de [V1 y V2](documentary-extraction-results-2026-10-20.md). El objetivo anterior produjo progreso verificable: implementó extracción, reveló rechazo del esquema y defectos semánticos, y preservó recibos y críticas. No pasó calidad ni autoriza abrir V6.

## Cambio y roster fijados antes de red

V3 vuelve al JSON Schema nativo V1. Conserva exactamente mensajes, textos, ontología, esquema, temperatura, máximo de salida y reglas. Solo el payload cambia `model` a `openai/gpt-4.1-mini` y la ruta a `openai` exclusiva, sin fallback. Se acepta únicamente proveedor servido `OpenAI` y modelo servido `openai/gpt-4.1-mini` o `openai/gpt-4.1-mini-2025-04-14`. Si el recibo usa el alias, no se atribuye verificación independiente de una revisión fechada. No se presentan comparaciones entre familias como cambio aislado de una instrucción.

Piloto de ocho peticiones: ordenar por las claves ya congeladas, elegir índices cero-based de fuentes **0, 27, 55, 82** y preguntas **0, 10, 21, 32**, intercalados en ese orden. Son los IDs originales `op_000, op_001, op_054, op_021, op_088, op_043, op_115, op_065`. La selección usa posiciones, no etiquetas, juicios manuales ni salidas del modelo. No se renumeran recibos históricos. El piloto no representa estadísticamente los 100 pares ni sustituye el banco difícil.

Preparación, directorio, fuentes ejecutadas y diario nuevos. Catálogo público observado antes de llamar: endpoint OpenAI estándar, soporte de `structured_outputs` y `response_format`, USD 0,40/M tokens de entrada y USD 1,60/M de salida. Se calcula estimación desde los ocho payloads y se anuncia alcance, coste y duración. Máximo ocho HTTP, cuatro concurrentes, sin reintentos; un error fatal detiene batches futuros, conservando las respuestas concurrentes completas.

La preparación y el runner reconstruyen el roster/textos/ontología/payloads desde el banco congelado, verifican hashes de banco, pase manual, fuente ejecutada y preparación, y rechazan perfiles desconocidos. Interpretar la pregunta sigue siendo independiente de cualquier recuerdo; extraer la fuente sigue siendo independiente de cualquier pregunta. Los frames manuales solo sirven al control y las ablaciones locales, nunca al payload ni a una reparación de la respuesta.

## Puertas

Se exige **8/8 salidas completas válidas** por esquema y contrato de dominio y **8/8 fieles** en crítica ciega separada de todos los elementos, sin gold, frames manuales ni resultados de ablaciones. Unsupported, omisión, incertidumbre de fidelidad y falta de salida no pasan. Una cita correcta no demuestra que el hecho sea fiel. Identificadores distintos pueden ser semánticamente equivalentes; perder referente, atributo, condición, negación o alcance es defecto. No se alteran anotaciones ni se añaden aliases extraídos del gold para hacer pasar la candidata.

`report-documentary-extraction` sigue informando los 100 pares y sus degradaciones en cuatro ablaciones: las operaciones fuera de este piloto son explícitamente no intentadas. Ningún porcentaje sobre ese subconjunto pasa automáticamente la puerta completa. Para ampliar a 116 se requiere el pase de capacidad y fidelidad de los ocho, sin reparar salidas. Si falla, conservar resultados y diagnosticar causas antes de otra versión explícita.

Coste informado una sola vez por respuesta, separando fuente y pregunta; importe ausente es desconocido. Latencia de petición y duración del runner no son latencia de recall/producto. Aun si el piloto pasa, faltan banco completo, 126 difíciles, comparación con baseline, recursos y estabilidad; después, cohorte nueva con revisión ciega. No hay adopción, cambio de runtime, nueva dependencia, secreto, servicio, SQL ni apertura de V6 en este piloto.
