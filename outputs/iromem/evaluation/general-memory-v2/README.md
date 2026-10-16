# Recuerdos generales v2 — artefactos de evaluación sintética

El [informe](../../../../docs/evaluation/general-memory-v2-results.md) describe el método, los resultados y sus límites. Esta carpeta contiene el manifiesto de la cohorte, los informes de formulaciones de desarrollo, los informes emparejados de proveedores para calibración, las decisiones fusionadas de la cascada y los resúmenes de semillas. No incluye credenciales ni recuerdos reales.

Los vectores híbridos, la base SQLite, las instantáneas de consultas y búsqueda, los diarios de llamadas y las solicitudes/resultados completos de semillas permanecen en `.local/pilots/general-memory-v2/`, ignorado por Git. `scripts/prepare_general_memory_followup.py` reconstruye la cohorte desde las instantáneas v1; `manifest.json` registra sus hashes y sujetos seleccionados. Las llamadas a proveedores pueden auditarse por ID de caso en los informes de esta carpeta.

`calibration-cascade-decisions.json` es un informe derivado que registra qué proveedor decidió cada caso después de aplicar la regla fija de segunda opinión. Lo consume `build_seed_requests.py`; generarlo no hizo llamadas adicionales al proveedor.
