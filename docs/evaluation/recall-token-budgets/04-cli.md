# 4. Recorrido CLI y coste local

Estado: implementado y medido con binario debug de features predeterminadas sobre fixture sintético, DB temporal y clave ficticia. Un límite de un token impide enviar al proveedor. No usa base personal ni inferencia pagada.

`recall_token_budget_is_enforced_by_cli_and_each_jsonl_operation` carga fixtures y ejecuta `recall-v1` y dos operaciones `recall_v1` en la misma sesión. Comprueba rechazo local, hash/conteo idéntico, presupuesto nuevo por operación y ausencia de recibo/coste. El directorio temporal incorpora secuencia atómica para evitar colisión entre pruebas paralelas.

[Mediciones crudas](cli-measurements.json): dos procesos nuevos, 524,64 y 675,86 ms de operación; once operaciones en una sesión, primera 649,78 ms y diez siguientes entre 1,21 y 1,87 ms. Cuerpo nativo de 191 tokens de referencia. La primera operación incluye carga del contador; la reutilización de sesión conserva el vocabulario en memoria.

Es una medición pequeña de admisión rechazada, con búsqueda y orquestación; no aísla tiempo del tokenizer ni mide selección remota, recall exitoso, p95 estable o mejora semántica. Ejecución concurrente con pruebas offline puede afectar los tiempos. La inicialización añade coste de CPU/memoria y el nuevo vocabulario aumenta el binario; no se midió RSS ni tamaño diferencial. FTS5 y E5 conservan su estado previo.
