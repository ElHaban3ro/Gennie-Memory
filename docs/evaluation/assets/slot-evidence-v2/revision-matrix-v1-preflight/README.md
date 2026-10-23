# Preflight aislado de la matriz de revisiones

Este directorio separa el corpus y las solicitudes sin gold de `annotations.jsonl`. No pasar anotaciones ni respuestas esperadas al clasificador. Los 24 JSON de `requests/` forman los brazos G0 (Gemma baseline), G2 (Gemma Slot Evidence V2) y D0 (Decisions baseline), con consulta, scope y roster idénticos por caso. Solo ejecutar después de tres smokes consecutivos válidos según el protocolo V2.

`search-cases.jsonl` se usa solo con `iromem search-batch` para confirmar cobertura local antes de cualquier llamada remota.
