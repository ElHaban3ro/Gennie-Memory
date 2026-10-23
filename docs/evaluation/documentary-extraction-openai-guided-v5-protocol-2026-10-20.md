# Extracción documental V5: esquema que expresa restricciones de dominio

UTC 2026-10-20. V4 completó ocho HTTP 200, ocho JSON válidos por forma, pero cero DTO completos por contrato de dominio. Coste informado USD 0,006513200, sin importes desconocidos nuevos. La corrección de parámetros permitió llegar al modelo; no pasó capacidad ni fidelidad. Las dos críticas ciegas y sus discrepancias quedan conservadas aparte, sin cambiar salidas/gold.

V5 mantiene el mismo piloto posicional de ocho textos, GPT-4.1 Mini/OpenAI exclusivo, JSON Schema nativo, temperatura y máximo de salida de [V4](documentary-extraction-openai-v4-protocol-2026-10-20.md). No añade ejemplos, nombres de entidades, frames manuales ni etiquetas. Cambia dos partes explícitas del payload: el esquema expresa restricciones del contrato ya existente y el mensaje de sistema aclara su codificación general. No es contraste de solo modelo ni validación nueva.

Restricciones estructurales nuevas, ya exigidas por el parser de dominio:

- `anyOf` distingue atributo `measurement` de los otros atributos: solo el primero admite identidad/ámbito de medición; el resto exige esos campos nulos.
- Cada hecho tiene exactamente uno de `datum` o `only_values` no nulo; el conjunto exhaustivo solo admite polaridad positiva.
- `readings` contiene al menos una lectura; ausencia de ambigüedad usa lectura literal sin hechos adicionales.

El validador local implementa esas uniones y mínimo, conserva el parser de dominio original y no repara salidas. Las pruebas rechazan metadatos de medición inaplicables, dato/conjunto simultáneos y lecturas vacías, y conservan hechos escalares, conjuntos y peticiones de medición válidos. Las versiones anteriores retienen esquemas, mensajes y reportes sin cambios.

La aclaración general también distingue requisito de valor/ignorancia, evento de medición de magnitud, uso de cumplimiento, sujeto singular de clase universal, relación aditiva de propiedad/conexión, cero escalar de conjunto de alternativas y citas literales de preguntas reescritas. Son reglas de codificación generales; no se suministran respuestas del piloto. Una cantidad solicitada no autoriza inventar una medición.

Preparación/código/diario nuevos, máximo ocho HTTP/cuatro concurrentes/sin reintentos. Estimación desde catálogo y payloads anunciada antes de red. La comprobación de parámetros de V4 permanece. Puertas: ocho salidas completas válidas y ocho fieles en revisiones críticas de todos los elementos, sin anotaciones manuales ni resultados de ablaciones. Los críticos de desarrollo pueden haber revisado la versión previa del mismo piloto; no se afirma independencia de una cohorte nueva ni revisión humana. Incertidumbre/defecto/ausencia no pasan. Si hay discrepancia, conservar ambos juicios y no declarar pase sin resolverla desde el texto.

Las cuatro ablaciones siguen mostrando los 100 pares y las operaciones no cubiertas; no se usa su denominador parcial para promover. Un pase del piloto solo permitiría preparar el banco completo. Los 126 difíciles, comparación con baseline, recursos/estabilidad y V6 quedan pendientes. No se modifica producto, banco/gold, secreto, dependencia, SQL ni servicio.
