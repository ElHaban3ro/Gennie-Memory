# Utilidad binaria compacta v2

2026-10-19. Variante experimental `per_memory_binary_v2`, perfil Jev explícito. No cambia baseline ni la versión binaria anterior. El piloto verificado v1 dejó un ID ajeno y costó2,03× la base histórica del subconjunto; no pasó y no se amplió.

Hipótesis: una pregunta binaria más estricta puede reducir la confusión entre información relacionada e información que responde un atributo pedido, sin pagar segunda etapa. Una petición contiene consulta/ámbito/regla comunes y ocho preguntas con memoria literal y dos opciones. A exige valor/estado explícito para al menos un atributo pedido o negación explícita de una premisa específica del mismo caso; none incluye atributos distintos, inferencia por silencio o por un requisito no exclusivo. Las identidades no se transfieren; la premisa puede corregirse. Las partes útiles siguen aceptadas aunque falten otras. No usa gold, ejemplos conocidos, nombres de corpus ni reglas por palabras específicas; ninguna garantía semántica procede del formato.

Mismo prefijo8, todos los positivos en orden, validación de IDs/revisiones/semilla completa, recibo global contado una vez. Búsqueda vacía omite llamada; salida inválida degrada sin semilla y conserva recibo parcial. No tablas, secretos, dependencias o servidores nuevos.

Piloto de las mismas diez consultas conocidas v5, consulta/gold/auxiliares intactos y huellas de fuente/base/casos: diez operaciones y máximo diez HTTP, DB temporal propia, CLI sin puerto, duración esperada30s. Auditar antes de interpretar. No seleccionar ningún ID ajeno ni perder ID útil de base; si falla calidad o coste histórico diagnóstico>1,5×, detener ampliación. Histórico no equivale a comparación pareada contemporánea.

Tras piloto favorable, campaña pareada completa conocida con candidatos idénticos: cero ID ajeno, cero pérdidas de evidencia/partes disponibles/degradación nueva y al menos una ganancia útil o parte adicional; recibos completos y coste,p50,p95<=1,5×base. Después congelar y evaluar cohorte nueva revisada a ciegas. V6 no se usa para ajustar. Preservar fallos y defaults hasta pase completo.
