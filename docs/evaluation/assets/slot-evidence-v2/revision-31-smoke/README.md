# Revisión 31: smoke de failover de proveedor

Nueva solicitud sintética; no reintenta la request de r30. El único cambio de configuración frente a r30 es el failover habilitado en las rutas Gemma experimentales, con `require_parameters=true`; Decisions predeterminado no cambia. El request JSON no lleva etiquetas. No enviar `case.jsonl` al clasificador.

Protocolo: misma solicitud/corpus en tres operaciones consecutivas; salida tipada, roster válido, `served_model`, `served_provider`, tokens y recibo de coste obligatorios. Detener al primer fallo sin repetir. Los tres directorios de resultado empiezan vacíos de llamadas.

Estimación máxima conservadora documentada: USD 0,004 para tres operaciones con hasta dos providers cada una; coste real se toma de recibos y una operación sin recibo queda como coste desconocido.
