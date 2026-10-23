# Aplicabilidad con citas v1: contrato probado y resultado descartado

**El mecanismo no es apto para promoverse: rechaza demasiada evidencia útil y añade tres degradaciones.** Se definió el [contrato inspeccionable](applicability-witness-v1-contract.md), se implementó una ruta explícita `witness_v1` y se probó con 64 preguntas nuevas, 224 recuerdos y revisión ciega. La base sigue predeterminada.

## Resultado principal

| Categoría | Base emparejada | Selección + testigo |
| --- | ---: | ---: |
| Respuesta completa: ID permitido | 16/16 | 6/16 |
| Refutación: ID permitido | 16/16 | 5/16 |
| Contexto parcial: ID permitido | 14/16 | 0/16 |
| Ausencia: semilla vacía | 16/16 | 16/16 |
| Selecciones finales de ID no permitido | 0 | 0 |
| Degradaciones técnicas | 0 | 3 |

Los tres degradados pertenecen a respuestas completas: `cev3q_051_02`, `cev3q_052_02` y `cev3q_053_02`, con `witness_invalid_evidence`. No se cuentan como abstenciones fiables. No hubo ganancias frente a la base y se activaron 35 bloqueos: 32 pérdidas semánticas de evidencia que conservaba la base y tres degradaciones nuevas. Las categorías no se compensan entre sí.

La búsqueda cubrió las 48 evidencias útiles entre veinte candidatos. Todas las consultas devolvieron veinte candidatos; el esperado fue primero en 16/16 respuestas, 8/16 refutaciones y 8/16 contextos parciales. Las pérdidas de esta campaña no son un vacío de recuperación.

## Qué se implementó y cómo se probó

Decisions propone un recuerdo con la instrucción `utility_v2`. Si se abstiene, termina sin extracción. Si propone, `openai/gpt-4.1-mini` devuelve bindings de referente, citas literales, relación y cobertura. El módulo puro de recuperación comprueba citas y acuerdo de campos; el servicio revalida la prueba antes de componer. Es una política explícita con consumo y testigo inspeccionables, sin cambios de SQL, conexiones, índices ni configuración predeterminada.

El contrato distingue `accepted`, `incompatible` y `not_established`; un esquema o cita inválido degrada. La presencia literal y el acuerdo de campos **no prueban extracción completa ni equivalencia semántica**. Se permiten múltiples condiciones con citas distintas. La normalización solo ignora mayúsculas y espacios; no resuelve alias, números dentro de sintagmas, negaciones o paráfrasis.

Los casos 050–057 son nuevos respecto de los experimentos anteriores. Un crítico separado vio solo fuente y preguntas, sin categorías, citas gold, cuotas ni resultados del modelo. Coincidió en las 64 categorías y citas y detectó la condición «presencial» faltante en ocho anotaciones. Se añadió antes de las llamadas y se conservó el gold inicial. La [revisión](../../tests/fixtures/conversation-evidence-v3/blind-review.json), [conciliación](../../tests/fixtures/conversation-evidence-v3/adjudication.json) y [etiquetas originales](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/author-labels-initial.jsonl) hacen visible ese proceso. Es revisión entre agentes de la misma familia, no humana ni independencia entre familias.

Se siguió el [protocolo previo](witness-v1-fresh-protocol-2026-10-19.md): 128 operaciones adyacentes, dos políticas por pregunta, orden alternado sin mirar etiquetas, Decisions, `keywords`, límite 20 y una SQLite temporal solo para esta cohorte. Los pares recibieron candidatos idénticos. Las anotaciones de referentes nunca entraron en las solicitudes runtime. Se congelaron datos, instrucciones, código y binarios antes de ejecutar, sin retoques o repeticiones posteriores. La interrupción anterior ocurrió antes de las llamadas remotas; no se duplicó ninguna campaña al reanudar.

Las plantillas son muy repetidas y usan calificadores explícitos, sin compresión y con doce registros auxiliares repetidos por ámbito. No son usuarios reales, paráfrasis libres o validación humana. El extractor recibe el resumen; los campos estructurados `conditions` son nulos en este banco y no se certifica su aplicabilidad en memorias que los tengan. No se ejecutó una respuesta final de Codex.

## Diagnóstico del testigo

Hubo 48 intentos de extracción: 45 trazas válidas y tres fallos de evidencia sin testigo válido. Sobre las **45 trazas válidas**, todas las propuestas eran IDs permitidos: el comprobador aceptó 11 y rechazó 34 que sí aportaban evidencia útil. Los ID de las tres propuestas con fallo técnico no quedaron en una traza válida y no se presume su corrección.

| Resultado local sobre 45 trazas | Casos |
| --- | ---: |
| Aceptado | 11 |
| Correspondencia no demostrada | 27 |
| Desacuerdo literal de campos | 7 |

Entre las trazas, 25 extracciones no incluyeron `subject` y ocho declararon cobertura incierta; hay solapamiento. La cobertura literal de todos los bindings gold fue 19/45. Veintiséis no cubrieron la cita gold de subject y una no cubrió item; también se solapan. Esta es una métrica de citas y dimensiones bajo la anotación fijada, no una prueba exhaustiva de omisión semántica: una cita más corta o una categorización distinta puede no cubrir el gold. La relación extraída coincidió con la anotada en 35/45, tampoco se presenta como verdad de una respuesta completa.

Los siete desacuerdos literales ilustran una limitación del propio contrato: cinco compararon condición presencial con una cláusula que niega la presencialidad, y dos compararon «cajas de documentos» con un sintagma que incluía además la cantidad. Ejemplos:

- `cev3q_056_04`: pregunta por acto presencial; memoria dice «el acto de entrega no será presencial». El extractor reconoce `refutes_premise`, pero sus bindings contienen frases diferentes y el comprobador la rechaza. La diferencia textual aquí es evidencia correctiva, no un referente distinto.
- `cev3q_055_05`: query binding «cajas de documentos» y memory binding «once cajas de documentos». La cantidad pedida quedó mezclada con la identidad del objeto, produciendo rechazo de una parte útil.
- `cev3q_057_06`: subject y persona estaban citados, pero el extractor declaró cobertura incierta y el comprobador se abstuvo.

El 16/16 de ausencia no demuestra eficacia del testigo para rechazar propuestas equivocadas: en esos 16 casos **el proponente ya se abstuvo y no se llamó al extractor**. Entre las trazas válidas no hubo propuestas incorrectas que permitieran medir esa capacidad. La prueba sí demuestra un fuerte rechazo de propuestas útiles. No se rebautiza ese vaciado como mejora de seguridad.

## Llamadas, tokens y gasto conocido

| Medición de campaña | Base | Testigo, ambas etapas |
| --- | ---: | ---: |
| Operaciones de consulta | 64 | 64 |
| Llamadas de selección | 64 | 64 |
| Intentos de extracción | 0 | 48 |
| Tokens de entrada reportados | 43.064 | 76.845 |
| Tokens de salida reportados | 0 | 4.818 |
| Gasto agregado conocido, USD | 0,0043064 | 0,0221292 |

La campaña hizo **176 intentos HTTP**. La ruta con testigo tuvo 112 llamadas y un gasto agregado conocido de aproximadamente 5,14 veces el de la base. Modelo de selección reportado: `openai/gpt-6-luna-decisions-20261006`; modelo de extracción reportado en las trazas: `openai/gpt-4.1-mini`. Cambiar de extractor forma parte del mecanismo evaluado; no se atribuye el resultado exclusivamente al código de comprobación.

En las 45 trazas válidas, la selección reportó USD 0,0038275, 38.275 tokens de entrada y cero de salida; la extracción reportó USD 0,0155364, 21.057 de entrada y 4.446 de salida. **No son el total de campaña:** faltan en ese desglose las abstenciones iniciales y las tres etapas fallidas. Los tres errores conservaron gasto agregado conocido de USD 0,0004535, 0,0004622 y 0,000494, respectivamente, sin una traza válida por etapa. No se inventa el desglose faltante ni se trata como cero el coste que pudiera faltar en una llamada sin recibo.

La comprobación técnica antigua fue independiente: una consulta, dos llamadas, USD 0,0004209. Confirmó la salida estructurada, pero se abstuvo por falta de subject; no se contó como acierto de selección ni se retocó con ese resultado. **Gasto conocido de campaña más prueba técnica: USD 0,0268565, con 178 intentos HTTP.** Es lo informado por las llamadas, no auditoría de factura, y no incluye el uso de Codex para programar o revisar etiquetas. Los tres errores no tienen desglose completo por etapa, por lo que se conserva la calificación de gasto conocido, sin afirmar exhaustividad contable.

| Tiempo del runner, ms | Base | Testigo |
| --- | ---: | ---: |
| p50 | 363,21 | 1.883,50 |
| p95 | 680,92 | 2.382,95 |
| p99 | 797,99 | 3.266,13 |

La mediana de la diferencia por par fue +1.495,81 ms, p95 +2.047,04 y p99 +2.872,67. Es una medición con red y una observación por pregunta/política, no latencia de integración con Codex. Los tokens incluyen selección y extracción; no son contexto enviado a Codex.

## Verificación y decisión

Pasaron las 51 pruebas del workspace y las comprobaciones de formato. Cubren citas fuera de fuente, duplicados, campos faltantes, condiciones compuestas, correcciones compatibles, costes conocidos de fallos y rechazo por el servicio de una selección sin testigo. La auditoría verificó 2.560 apariciones de candidatos y 57 recuerdos de semillas (46 base, 11 testigo), fuente declarada, texto, condiciones, ámbito, revisión, estado, vigencia, hashes y eventos. La verificación mecánica separada recontó orden, categorías, consumo y correspondencia con la revisión ciega. No verifica fuentes externas, extracción infalible o verdad de una respuesta final.

Se descarta la promoción de esta versión y se conserva para reproducir el experimento. El núcleo literal del contrato funcionó según sus reglas, pero esas reglas y la extracción no fueron una buena prueba de aplicabilidad: confundieron identidad, atributos preguntados y premisas corregibles. La siguiente propuesta debe separar esos roles y probar el comprobador con testigos anotados, antes de añadir extracción remota. No se encadena otro prompt o modelo para rescatar esta campaña, ni se vuelve a usar v3 como reserva intacta. Se mantienen `baseline`, límite 20, las métricas vigentes del README y el cierre limitado de M5.

Archivos: [banco](../../tests/fixtures/conversation-evidence-v3/manifest.json), [preflight](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/runs/preflight.json), [instrucciones](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/runs/instructions.json), [búsqueda](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/retrieval-report.json), [diario](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/runs/journal.jsonl), [reporte por política y diagnóstico](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/runs/report.json), [auditoría](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/runs/audit.json), [verificación](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/runs/verification.json), [piloto](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/technical-pilot-journal.jsonl) y [manifiesto de resultados](../../outputs/iromem/evaluation/witness-v1-fresh-2026-10-19/manifest.json).
