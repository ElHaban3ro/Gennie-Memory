# 2. Peticiones y acumulación

Estado: implementado en `bootstrap::openrouter::token_budget`, `retrieval::product` y las entradas acotadas de `Classifier`.

El transporte cuenta los mismos bytes JSON que asigna al cuerpo HTTP, después de serializar y antes de POST. El plazo se vuelve a comprobar tras el conteo para incluir su inicialización. Los hashes/conteos registran admisión, no envío o facturación. Ningún rechazo produce un recibo nuevo. La revisión de equivalencias recibe solo el remanente; si no cabe, conserva selección y recibo principales y se declara omitida con ejecución degradada.

Pruebas nativas: `product_providers_and_policies_guard_full_payload_and_second_pass` cubre Decisions, Jev 1.13 y Jev 1.14 con baseline, per-memory v1/v2 y revisión. `rejected_token_budget_opens_no_connection_and_can_be_reused_at_exact_limit` confirma ausencia de conexión a un listener propio al rechazar; comprueba la frontera exacta. `evaluation_capture_records_only_hashes_and_transport_metadata` compara hash y conteo del cuerpo recibido por un mock HTTP con la contabilidad, además de preservar el recibo independiente y excluir la credencial de los artefactos.

Las pruebas de producto usan un adaptador sintético para probar flujo y fronteras, sin presentarlo como protocolo de proveedor. Los mocks nativos prueban los bytes reales. Una petición rechazada sigue contando como entrada observada en la traza, pero no se descuenta del remanente.
