# 0002 — Banco sintético y pilotos comparables

Fecha: 2026-10-15. Estado: aceptado como infraestructura de evaluación, sujeto a revisión después de la campaña de calibración.

Se versiona un corpus determinista de 10.800 consultas para distinguir recuperación de positivos, rechazo de ausencias y filtración entre ámbitos. Las etiquetas proceden de reglas del generador y se separan por entidad en desarrollo, calibración y final. `search-batch` mantiene una conexión SQLite por proceso y registra la latencia del servicio Rust por consulta. Los runners de Laya, coseno y RRF permanecen scripts de evaluación identificados; no se presentan como semilla integrada.

Se eligió medir primero una muestra pequeña para detectar errores de contrato, costes y falsas mejoras antes de la campaña grande. El piloto muestra que las dos variantes de clasificación Laya probadas no mejoraron FTS5 en ocho casos y que coseno/RRF cambian el ranking. La elección de umbral y proveedor se aplaza a calibración, no se decide con esa muestra. La caché de Laya se usa en lectura y los reportes quedan fuera de Git salvo el resumen verificable en [piloto](../evaluation/pilot-2026-10-15.md).

Se fijó Jev 1.13 para el primer sondeo remoto porque Jev 1.14 devolvió HTTP 504; GPT-6 Luna Decisions se probó mediante OpenRouter. La clave `GENNIE_MEMORY_OPENROUTER_API_KEY` no entra en fixtures, informes ni código. Las versiones y precios se deben volver a verificar antes de una corrida extensa. Ver [contrato](../evaluation/contract-v1.md) y [roadmap](../../ROADMAP.md).
