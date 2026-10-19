# Diagnóstico de red restringida

El primer intento de `campaign` para Decisions y Jev terminó con 405/405 respuestas `degraded` y `transport_error` por proveedor, sin estado HTTP, tokens ni coste informado. En el mismo entorno, `curl` no pudo resolver `openrouter.ai` (código 6). Una comprobación HTTP fuera de esa red devolvió 200; se ejecutaron campañas nuevas en los diarios del directorio padre. Estos archivos **no entran en la evaluación de calidad ni latencia remota**. Se conservan para explicar el diagnóstico y evitar presentar un fallo de transporte como una decisión de los modelos.
