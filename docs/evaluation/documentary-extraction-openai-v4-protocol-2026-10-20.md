# Extracción documental V4: compatibilidad de parámetros

UTC 2026-10-20. El [piloto V3](documentary-extraction-openai-v3-protocol-2026-10-20.md) terminó tras cuatro HTTP 404 completos de OpenRouter: `Filter by Parameters`, sin salida del modelo ni recibos de coste. Cuatro importes quedan desconocidos; no se atribuyen errores semánticos ni gratuidad. Las cuatro peticiones restantes no se enviaron. No se reintenta el diario V3.

El catálogo observado del endpoint OpenAI no declara `reasoning`. V3 conservaba `reasoning.enabled=false` para igualar el payload de Gemini; la evidencia de routing apunta a ese campo como causa. V4 retira **únicamente ese campo desactivado**: mismo modelo/proveedor exclusivo, ocho posiciones/IDs, mensajes, ontología, JSON Schema nativo, temperatura y límite de salida de V3. Si V4 llega al modelo, confirma la compatibilidad de esta corrección; no se afirmará éxito antes de ejecutarla. No se añade razonamiento, fallback ni cambio semántico.

Se añade comprobación local de parámetros de generación presentes (`temperature`, `max_tokens`, `response_format`, `reasoning`) contra el catálogo declarado, tanto en preparación como en runner antes de red. Una prueba negativa reproduce el bloqueo del campo no soportado y comprueba igualdad del resto del payload. Este control no garantiza que el proveedor acepte todos los esquemas ni evita límites de tráfico.

Directorio/preparación/fuentes/diario nuevos. Mismas ocho operaciones, máximo cuatro concurrentes, sin reintentos; se anuncian coste y tiempo antes de llamar. Se mantienen todas las puertas V3: ocho DTO completos y ocho extracciones fieles por críticas ciegas separadas, sin gold ni reparación de salidas. Ausencia, defecto o incertidumbre no pasan. Las ablaciones conservan los 100 pares y degradaciones, sin convertir un piloto en pase completo.

No cambian banco/gold, baseline, producto ni V6. Todos los informes históricos deben reproducirse byte a byte; el nuevo control previo no reinterpreta los errores del diario V3 ni borra sus costes desconocidos.
