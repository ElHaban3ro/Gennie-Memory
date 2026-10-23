# Premisa incompatible V1: protocolo de flujo real

Se implementa `per_memory_complete_premise_v1` como variante explícita experimental, sin cambiar `baseline`. El diagnóstico anterior obtuvo 12/12 y 43 comprobaciones completas con preservación/precisión y coste reconstruido de 1,275× V5 y 1,354× V4. Es una condición para implementar y medir, no una validación del flujo actual.

## Procedencia del diagnóstico

El preflight original omitió la entrada separada del archivo de pregunta atómica. Se conserva intacto y se informa como fallo de ese vínculo directo. La huella de todas las solicitudes sí se guardó antes de red; cada una contiene el texto completo de la cuarta pregunta. El auditor exige que las 43 solicitudes coincidan con el fixture, además del resto del payload y las fuentes. Esta vinculación transitiva permite auditar los juicios sin repetir llamadas ni presentar el preflight directo original como aprobado. El generador se corrigió para incluir la entrada directa en futuras ejecuciones.

## Mecanismo de producto optativo

Jev propone tipos sobre el mismo prefijo de siete candidatos con el payload V2. Cada positivo recibe una petición aislada a `perplexity/pplx-decider-v1.2`; la revisión servida debe ser `perplexity/pplx-decider-v1.2-20261016`. No se acepta una revisión diferente silenciosamente. Las cuatro preguntas y composición coinciden con el diagnóstico: dato OR (corrección ordinaria AND incompatibilidad necesaria) OR contraste explícito. El tipo de propuesta no elimina vías. No se comprueban ni seleccionan propuestas negativas.

`CheckedMemory.complete_paths` conserva los cuatro booleanos reales y un recibo por petición; las vías históricas quedan ausentes. El servicio revalida proveedor/revisión, esquema por política, IDs/revisiones, cuatro vías, utilidad y recibos. El auditor JSON independiente replica la composición y valida las cuatro entradas incluso si OR podría cortocircuitar. Las trazas antiguas conservan su JSON al omitir el campo nuevo.

Las comprobaciones por candidato son concurrentes, máximo siete, tras una propuesta; máximo ocho HTTP por operación. Un error de cualquier comprobación degrada la operación completa y conserva una vez el consumo conocido de cada etapa. Falta de recibo no es cero. No se añaden proveedores elegibles al enum de propuesta, variables, secretos, SQL, conexiones, dependencias o puertos.

## Medición antes de ampliar

Congelar código, binarios, protocolo, casos, corpus y gold. Ejecutar una vez los trece casos V5 de preservación y las cinco regresiones V4 mediante la CLI real, con nuevas propuestas y comprobaciones. Se usan bases temporales propias y se eliminan al terminar. Estimación USD 0,003 y uno a dos minutos, máximo 144 HTTP, sin puertos ni reintentos inciertos. La autorización general de pruebas sintéticas por OpenRouter continúa vigente.

Exigir V5 sin IDs no permitidos, pérdidas de base/scopedV1 o degradación, con alguna ganancia; V4 cinco sets exactos sin pérdidas/extras. Coste completo de cada grupo ≤1,5× del subconjunto histórico. Mostrar omisiones y tiempos CLI, sin confundir diagnóstico con evaluación nueva. Solo con ambos pases: campañas completas conocidas V5/V4/V2, puerta global Y preservación, coste/p50/p95 ≤1,5× y consistencia fija. V6 permanece reservada hasta superar esas puertas. Una pérdida o coste excesivo detiene la ampliación; no se relabela ni elige la mejor repetición para pasar.
