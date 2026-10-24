# Conversaciones sintéticas M5 v1: desarrollo

Este paquete **solo** contiene la primera partición de desarrollo del plan M5. Son cinco conversaciones ficticias nuevas (`021`–`025`), 150 recuerdos (30 por conversación) y 100 preguntas: quince positivas y cinco negativas por conversación. Las consultas incluyen un ámbito `conversation` explícito; no evalúan resolución de contexto ni permisos reales.

Los hechos, preguntas y etiquetas se diseñaron para este sondeo y se escribieron en archivos congelados. `label_source: deterministic_fixture_rule` significa que el ID esperado procede de la asociación determinista entre la pregunta y el recuerdo escrito para el caso; **no** significa revisión humana. Las cinco conversaciones repiten quince categorías controladas, con distintos valores y contextos. Esta repetición ayuda a localizar fallos por categoría, pero no constituye diversidad de preguntas naturales. No usar esta partición para estimar una tasa final ni mezclarla con la cohorte 018–020.

- `memories.json`: entrada de `gennie-memory load-fixtures`, con fuente `synthetic://` declarada, no verificada externamente.
- `development.jsonl`: 75 positivos y 25 negativos para búsqueda y campaña Decisions.
- `development-probe.jsonl`: seis reformulaciones posteriores a ver los errores de desarrollo; cinco preguntas de estilo de respuesta y una de supuesto abandonado. Es un sondeo exploratorio de recorrido completo, **no** una partición reservada ni una lista fija de candidatos.
- `manifest.json`: tamaños, procedencia y SHA-256 de los tres archivos anteriores.

Los negativos preguntan por hechos que no figuran como respuesta aplicable: contraseña, presupuesto final aprobado, fecha de despliegue, cliente firmante o modelo de teléfono personal. Una lista FTS5 no vacía en esos casos es normal: el clasificador debe dejar la semilla vacía. La pregunta de presupuesto tiene una estimación preliminar como distractor; aprobarla como presupuesto final sería un falso recuerdo.

El [informe de desarrollo](../../../docs/evaluation/conversation-m5-development-2026-10-16.md) enlaza diarios, resultados por caso y límites. Las particiones de calibración y prueba nueva reservada siguen sin crearse ni ejecutarse; primero deben diseñarse y revisarse sus etiquetas sin mirar salidas del modelo.
