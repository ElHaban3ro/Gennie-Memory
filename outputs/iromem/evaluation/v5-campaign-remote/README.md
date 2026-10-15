# Campaña remota v5 de recuerdos semilla

Estado: **ejecutada el 2026-10-15** con autorización del usuario. Se usaron exclusivamente [1.600 consultas sintéticas de desarrollo](../../../../tests/benchmark/v5/development-campaign-1600.jsonl): 1.000 positivas que cubren una vez cada una de las 1.000 formas y 600 negativas. La [instantánea híbrida](../v5-campaign-development/v5-campaign-hybrid-results.jsonl) fijó los mismos candidatos para ambos proveedores y las comparaciones locales. No se abrió calibración ni final.

El [plan congelado](../v5-campaign-development/v5-campaign-openrouter-plan.json) tiene SHA-256 interno b7c8452a82bcfe6e3418686635aeccf1c1af6a3dc6ed5d675476a5c354c21035. Se intentaron 2.800 llamadas, 1.400 por proveedor; 200 casos sin candidatos por proveedor se resolvieron sin llamada. Ocho peticiones a OpenAI devolvieron HTTP 504. Se registraron como fallos terminales y **no se reintentaron**; Jev no registró fallos. Las 3.200 tareas terminaron y no quedó ninguna llamada incierta. Una tentativa previa en el entorno restringido falló por DNS antes de alcanzar OpenRouter; su diario separado no forma parte de estas métricas.

## Resultado emparejado

Una semilla exacta contiene únicamente el ID esperado en un positivo y conserva revisión, hecho, condición y fuente declarada; o queda vacía en un negativo. Los fallos HTTP y las semillas degradadas cuentan como error. Los resultados remotos pasaron por el constructor de solicitudes, el compositor con la base congelada y el evaluador de semillas. Los locales son las campañas MPS sobre las mismas consultas y candidatos. Top‑1 usa Laya como encoder híbrido, aunque no lo llama como selector.

| Selector | Positivos exactos / 1.000 | Negativos vacíos / 600 | Total exacto / 1.600 | Fallos del proveedor | p50 de decisión | Tokens de clasificación informados | Coste informado o reservado |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Top‑1 local | 719 | 200 | 919 | — | 0 | 0 | 0 |
| Laya choice local | 136 | 568 | 704 | — | Incluido en p50 total de 52,72 ms | 248.901 | 0 |
| Laya binario condicionado local, umbral 0,5 | 494 | 200 | 694 | — | Incluido en p50 total de 86,61 ms | 518.588 | 0 |
| Jev 1.13 por OpenRouter | **1.000** | **570** | **1.570** | **0** | 475,91 ms | 717.871 | US$0,03015 |
| OpenAI GPT‑6 Luna Decisions por OpenRouter | 992 | 526 | 1.518 | 8 | 482,07 ms | 378.345 | US$0,03943 |

El coste total informado o reservado por el runner fue **US$0,06959**, bajo el tope de US$1. El coste informado por respuestas completas fue US$0,03015 Jev y US$0,03783 OpenAI; para los ocho 504, sin coste reportado, el runner mantuvo una reserva de US$0,0002 por llamada. Esto no confirma la facturación final de la cuenta. Jev sirvió typesafe/jev-1.13-20260917; OpenAI sirvió openai/gpt-6-luna-decisions-20261006.

La p50 remota mide **solo la decisión del proveedor en llamadas completas**. La p95 fue 535,53 ms para Jev y 760,10 ms para OpenAI; p99, 699,69 y 1.160,80 ms. La composición local posterior tuvo p50 3,05 y 3,12 ms, respectivamente. La búsqueda híbrida congelada tuvo p50 20,04 ms con top‑1 local. Estas cifras proceden de recorridos distintos: no se deben sumar percentiles y presentarlos como latencia integrada observada. Los casos sin candidatos no realizan llamada, y el arranque/carga del índice se midió aparte en el recorrido local.

| Segmento negativo | Casos | Jev vacíos | OpenAI vacíos |
| --- | ---: | ---: | ---: |
| Entidad ausente | 100 | 100 | 100 |
| No consultar recuerdos explícitamente | 100 | 100 | 99 (1 HTTP 504) |
| Modalidad desconocida | 100 | 100 | 86 |
| Modalidad sin decidir | 100 | 100 | 63 |
| Ámbito ajeno | 100 | 100 | 100 |
| Tarea ajena | 100 | 70 | 78 |

Jev acertó los 1.000 positivos y conservó hecho, condición y fuente declarada en cada semilla positiva. OpenAI acertó 992: siete positivos fallaron por HTTP 504 y uno recibió «none» pese a tener el recuerdo correcto entre candidatos. Las 992 semillas positivas correctas conservaron las tres propiedades. No hubo IDs prohibidos. La comparación emparejada da 1.505 aciertos de ambos, 65 solo de Jev, 13 solo de OpenAI y 17 errores de ambos. Jev supera a OpenAI en el **total de esta muestra sintética de desarrollo**, aunque OpenAI acierta ocho casos de tarea ajena que Jev falla. Los errores de Jev se concentran en tarea ajena; su comportamiento no está validado en preguntas humanas.

## Evidencia y reproducción

- [Resumen validado](summary.json): huellas de entradas, métricas, segmentos y tablas emparejadas contra los tres selectores locales.
- [Decisiones por caso](decision-report.json): estados, IDs presentados, elección, probabilidades, modelo servido, tokens, latencia y coste reportado. No contiene la clave ni los textos de consulta; el [plan](../v5-campaign-development/v5-campaign-openrouter-plan.json) contiene estado y candidatos sintéticos.
- Semillas completas por caso: [Jev](jev-seed-results.jsonl) y [OpenAI](openai-seed-results.jsonl); [evaluación agregada Jev](jev-seed-evaluation.json) y [OpenAI](openai-seed-evaluation.json).
- El diario reanudable original permanece en .local/pilots/v5-campaign-openrouter-network-journal.jsonl (ignorado por Git), SHA-256 7733c60e9462dfcb99f0d477b3e6b3dcbcbe7b19f4f303e4abd6bd732a008777. Registró 2.800 eventos de inicio, 2.800 de fin y 400 casos vacíos. La [guía de ejecución](../../../../docs/evaluation/remote-v5-runbook.md) contiene los comandos de exportación y composición.
- [El resumidor](../../../../scripts/summarize_remote_campaign.py) volvió a validar huella del plan, 3.200 pares caso/modelo, candidatos, estados, elecciones, composición, etiquetas y ausencia de IDs prohibidos. Las huellas de los informes publicados están en summary.json.

El proveedor recibió solo el estado (pregunta y ámbito sintéticos) y un mapa de candidatos con sus síntesis más la opción «none»; las etiquetas esperadas quedaron en el plan local y **no** entraron en el payload de la API. Preguntas, recuerdos, ámbitos y fuentes son sintéticos. Las etiquetas derivan de reglas del generador sin revisión humana. La muestra de desarrollo comparte familias de plantillas con calibración; no prueba generalización, una política final ni ahorro de tiempo del agente Codex. La evaluación final sigue reservada.
