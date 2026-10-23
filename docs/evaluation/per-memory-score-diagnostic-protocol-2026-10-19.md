# Diagnóstico prospectivo de scores por recuerdo

2026-10-19. Investigación posterior a V3/V5 conocidas fallidas. **Método fijado antes de recoger scores; instrumentación optativa implementada sin cambiar la selección.** Goal activo; no cambia selección ni baseline. Las probabilidades numéricas que ya exige el parser de Decisions son scores del clasificador, no probabilidades calibradas de verdad.

## Pregunta y datos obligatorios

¿Existe un umbral global sobre el score A del payload binario V5 que separe en desarrollo todos los recuerdos útiles anotados de todos los no permitidos del prefijo? La instrumentación optativa deberá emitir exactamente el mismo payload V5 (modelo, consulta, reglas, criterios, textos, orden y prefijo8) y conservar su elección. Añadir solo traza de los scores A/none tal como llegaron: no normalizar, reemplazar choice, rescatar none ni adoptar un umbral al recogerlos. No enviarlos como afirmaciones dentro de seed_text, ni guardarlos como verdad de memoria en SQLite.

Unidad: pregunta×ID×revisión, posición y resumen/hash, ámbito/versión de búsqueda, hash de payload/binario, modelos solicitado/servido y elección. Trazas completas para todos los candidatos presentados; un recibo por HTTP global, no por pregunta interna. Registrar si choice discrepa del score mayor. Valores finitos[0,1] y suma dentro de tolerancia ya admitida por parser; preservar valores sin reescalar. Score ausente/inválido o timeout son errores, no cero ni irrelevancia. Coste ausente sigue desconocido.

Interfaz prevista: RecallRequest.selection_policy=`per_memory_scored_v1`, perfil de evaluación Jev explícito. La traza opcional `PerMemoryTrace.choice_scores` contiene `{memory_id, revision, a, none}` por cada candidato del prefijo. Se genera en orden `memory_00`…`memory_07` desde las respuestas del mismo payload; no se infiere un índice de verdad ni se crea tabla/índice SQLite. Las variantes anteriores omiten ese campo. Servicio y evaluador deben exigir cardinalidad, ID/revisión, scores válidos y correspondencia con toda semilla; el instrumento no cambia choice ni relation. Bootstrap extrae recibo/valores; retrieval posee tipos y validación; tools/evaluation posee preparación y ajuste offline.

## Cohorte, preparación y límite

64consultas v5 conocidas, una corrida completa de diagnóstico, sin elegir solo los éxitos. Consultas/gold/corpus y candidatos vinculados con huellas de fuente y auditoría ordinaria; prefijo8 igual a V5. Separar evidencia útil fuera del prefijo; un umbral no la recupera. Instrumentación debe probar que no cambian elección/semilla y que se rechazan scores omitidos, extra, cambiados de ID/revisión o inválidos. Antes de llamadas: revisión crítica y workspace/fmt. CLI sin puerto, DB temporal propia, máximo64HTTP en64operaciones, duración prevista hasta60s, sin reintentos inciertos. Registrar recibos; no incluye Codex. La autorización vigente cubre investigación sintética, no cambiar defaults.

## Ajuste predefinido y descarte

Único score permitido: A original, mayor significa más utilidad. Positivos: todos los IDs útiles anotados presentes en el prefijo; negativos: todos los demás candidatos presentados. Sin excluir preguntas, referentes, relaciones, segundos positivos o negativos difíciles. Si faltan operaciones/juicios o alguna es técnicamente inválida, no declarar separación de la cohorte completa. Reportar desconocidos y fuera de prefijo aparte.

Calcular min_positive y max_negative. Banda factible solo si ambos grupos existen y min_positive-max_negative>=0,05. Este margen es la separación TOTAL entre grupos, no0,05 a cada lado. Elegir threshold=(min_positive+max_negative)/2 y aceptar scoreA>=threshold (empates se aceptan). Método congelado antes de observar scores; ningún umbral por consulta/ID/relación ni transformación alternativa tras ver resultados. No mezclar choice con score en la regla futura: el experimento de ajuste es exclusivamente scoreA>=threshold. Si no hay banda, descartar esta hipótesis global y conservar evidencia; no relajar margen ni cambiar etiquetas para obtenerla.

Aun con banda, solo hay ajuste conocido, con candidatos/preguntas correlacionados y etiquetas sintéticas: no generalización, calibración factual o garantía de cada parte solicitada. Comprobar toda semilla, todas las relaciones, conservación de IDs útiles de base y partes disponibles, y reportar cualquier evidencia no presentada. No puede aprobarse el objetivo con una banda que pierde partes fuera del prefijo.

## Puertas posteriores

Si hay banda y cobertura, congelar score, umbral, modelo/payload y decodificador en una política explícita versionada. Medir la ruta completa en comparaciones pareadas conocidas v5/v4/v2, incluyendo variación real; cero ID ajeno, pérdidas útiles/partes o degradación nueva, al menos una mejora, recibos completos y coste,p50,p95<=1,5×base. Después V6 nueva revisada a ciegas; no usarla para ajuste. Incorporar solo con todos los pases. La recogida diagnóstica no es promoción ni prueba de truth de respuesta final.

La revisión crítica previa del método exigió congelar dirección/empates/margen/método del umbral, no ocultar fallos ni evidencia fuera del prefijo, auditar conjuntos y distinguir scores de verdad. No leyó V6 ni ejecutó llamadas.
