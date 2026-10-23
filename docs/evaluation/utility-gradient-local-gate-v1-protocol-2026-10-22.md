# Puerta local V1 del candidato de corrección fija

Fecha: 2026-10-22. El replay anterior mejoró TRAIN155→174/176, útiles70→72/72 y ajenos19→2 sin errores nuevos. La aproximación lineal falló y no autoriza predicciones sobre otros conjuntos. [Reporte](utility-gradient-replay-v1-results-2026-10-22.md). Esta evaluación mide la única candidata exacta congelada, sin entrenarla de nuevo.

## Candidata, entradas y constantes

Checkpoint SHA-256 `32d62088e13b4e6b4b5f63aec22839e01e8db5e633fadaaf6b02f01dc4356ad7`, copia del replay coeficiente1 único. No nueva delta, escala, umbral, época, semilla alternativa o selección por resultados. Base Qwen3-Reranker-0.6B y LoRA FP32/runtime/tokenización/instrucción idénticos, todas las fuentes y hashes fijados antes de scores.

88 forward, cero backward/optimizador: primero los24de desarrollo, después los64conocidos, en los órdenes de V5 congelados. Ambos conjuntos fueron observados previamente; son **regresión local**, no validación fresca. Ninguno se usó para generar delta, gradientes o seleccionar filas del diagnóstico. Las entradas del modelo siguen siendo solo query/document; target/role/IDs/razones/scores anteriores quedan fuera del texto. Conservar íntegros documentos y tokens, sin truncar, máximo32768.

Congelar inputs/labels/tokenids y su procedencia desde V5, copias de checkpoint, protocolo/runner/runtime antes de inferencia. Verificar tokenids reconstruidos idénticos y32arrays LoRA FP32, base FP16 y hashes antes/después, mismos archivos fuente/runtime al cierre auditado. Modelo eval, GPU MLX explícita sin fallback, una instancia sin caché entre ejemplos ni compilación que capture pesos. Diario UTC RFC3339 milisegundos y secuencia de cada inicio/respuesta; persistir scores y decisiones de los88casos completos.

## Comparadores y puerta original

Comparadores históricos por caso: reranker sin adaptar y V5 matched antes de la corrección, tomados de las campañas congeladas sobre los mismos textos/modelo/instrucción. No se repiten esos comparadores ahora: informar que no es comparación contemporánea, causal de latencia o contra configuración productiva FTS5/Decisions.

Puerta previa a cualquier reserva, sin rebajar: **24/24desarrollo y64/64conocidos**, cero pérdidas útiles, ajenos o degradaciones y al menos una ganancia sobre reranker sin adaptar. Reconstruir independientemente esas condiciones con el gold fijo. Comparar también contra padre V5, sin sustituir base para aparentar mejora. No usar ranking/top1o cantidad fija de seleccionados. Completar ambos conjuntos aunque falle calidad del primero, salvo fallo técnico; ningún score parcial cambia configuración o descarta casos.

Si hay cualquier error, conservar versión y no abrir reserva. Un resultado inferior exige revisar evidencia sin cambiar labels/resultados. Si pasa completa, solo habilita preparar por separado la reserva original24una vez para esta candidata ya congelada; este runner no abre ni lee esa reserva. V6 tampoco se abre. Un pase en observados no demuestra generalización nueva o adopción: siguen casos nuevos/pools mixtos con revisión ciega, ruta vigente completa, recibos/coste/p50/p95≤1,5×, estabilidad y V6 final.

## Recursos, conservación y límites

CLI temporal propia sin puerto/HTTP/API/servicio. Estimación1–2minutos, watchdog600s, techo88forward/0backward/0optimizador. Sin instalaciones, dependencias/pesos base nuevos, DB, secretos, env persistente, procesos ajenos, Rust/Cargo, rama, commit o push. APIUSD0; coste económico local/Codex desconocido. Registrar handle/PID/script y salida observada o desconocida. Ante pérdida de observación inspeccionar mismo proceso/artefactos, no reiniciar solo por timeout. Fallo técnico conserva intento y no se reintenta en directorio existente.

Artefactos en `outputs/iromem/evaluation/utility-gradient-local-gate-v1-2026-10-22/`. Auditor offline reconstruye métricas/pérdidas/ganancias/roles/puerta/eventos/hashes sin otra inferencia, separando integridad y calidad. Las etiquetas sintéticas y revisión de la misma familia tienen límites de población y no prueban fuentes externas. Objetivo original de cinco pasos intacto y activo.
