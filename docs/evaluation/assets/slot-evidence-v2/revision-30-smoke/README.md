# Revisión 30: entradas preparadas para smoke

Caso sintético nuevo de identidad/slot para tres llamadas consecutivas con la misma revisión del prompt V2. Es distinto al caso r29; no es un reintento. Se crearon carpetas `smoke-1` a `smoke-3` para separar recibos y resultados. La carpeta `input` contiene solicitud y corpus sin etiquetas.

No enviar `case.jsonl` al clasificador. La ejecución requiere respuesta estructural válida, recibo completo, roster exacto y selección del candidato en cada llamada. Al primer fallo se detiene la secuencia y se conserva el artefacto; no se reintenta esa respuesta.
