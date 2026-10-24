# 0026 — Referencias con evidencia distribuida, solo evaluación

Estado: prototipo local no adoptado. UTC 2026-10-21.

El catálogo anterior conservó referencias, pero rechazó un propietario citado en otra oración y no equiparó sensor/sensores. Se añade un adaptador versionado en gennie-memory-eval que separa mención y evidencia del vínculo, conserva aplicabilidad/contexto, y admite una vinculación explícita a clases de un registro fijo. El traductor antiguo mantiene su callback original y sus resultados se reproducen.

Se descartaron recorte de cabezas, aliases derivados de gold y ampliación artificial de la frase nominal. Un registro de definiciones no demuestra equivalencia semántica: la matriz conserva dos falsos positivos que bloquean adopción. No hay extractor, cambios de producto, dependencias o recursos runtime nuevos. Coste de proveedor de este prototipo: cero; recursos del mecanismo automático aún sin medir.

Revisión: comprobar correspondencia cita/clase/intervalo y conservación de calificadores y de cada solicitud antes de una nueva versión. Requiere conocidos, baseline, recursos/estabilidad y cohorte nueva ciega antes de promoción. [Protocolo](../evaluation/distributed-reference-v1-protocol-2026-10-21.md) y [resultados](../evaluation/distributed-reference-v1-results-2026-10-21.md).
