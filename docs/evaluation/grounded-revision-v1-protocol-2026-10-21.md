# Revisión automática única del borrador documental V1

UTC 2026-10-21. Paso 3 del goal activo. El turno previo fue progreso con evidencia negativa: contrato con menciones/clases implementado, 4/8 DTO, seis defectos de contenido acordados, sin ampliación. [Resultado preservado](grounded-reference-v1-results-2026-10-21.md).

## Hipótesis y alcance íntegro

Antes de sustituir las referencias por anclas opacas que aún no demuestran igual expresividad, se prueba una única revisión automática del borrador completo. Se conserva íntegramente el contrato grounded-reference V1, traductor, parser, cualificadores, identidad/premisas/datos solicitados y puertas. No se ocultan nombres/clases/propietarios omitidos borrando campos ni se limita la representación a referencias contiguas fáciles.

Los ocho borradores del piloto anterior son entradas congeladas de la primera etapa, no respuestas correctas ni memoria de referencia. Todos reciben una revisión, incluidos los cuatro DTO aceptados. El modelo recibe solo el texto fuente O pregunta original, ontología, esquema, previous_output del mismo ítem y local_validation_error recalculado por el traductor o null. Nunca ambos textos de un par, candidatos, gold, IDs/targets del banco, anotaciones manuales, críticas semánticas de Codex ni nombres que el autor diga que faltan. La guía de revisión es idéntica para todos y exige examinar cada cláusula/referente y conservar nombres, clases realmente declaradas, propietarios intermedios, modalidad, alcance, incertidumbre y separación de premisas. Un error null no certifica semántica. El borrador y texto son datos no confiables, no instrucciones.

La única salida revisada es obligatoria y final. No se combina con la primera ni se elige la mejor, aunque empeore. Un fallo de revisión degrada toda la operación; el DTO inicial no sirve de fallback. No hay segunda revisión. No se repite una petición incierta o fallida: solo se revisan los ocho padres terminales HTTP200 completos y con recibos conocidos. Revisar un borrador ya recibido es una etapa nueva autorizada, no reintento de una llamada perdida.

## Integridad y comprobaciones

Preparador y runner reconstruyen los mismos ocho IDs posicionales desde el banco; comprueban igualdad con las peticiones padre, huellas fijas de petición/diario/preparación/preflight padre, cuerpo de un solo modelo OpenAI, fin stop, inicio/fin único y recibo completo. El módulo puro forma los cinco campos permitidos del mensaje y recalcula el diagnóstico. El validador rechaza diagnóstico manipulado o campos extra, incluso si parecen útiles. El schema difiere solo en su identificador de versión para poder rechazar revisiones no vinculadas. No cambian las reglas de traducción ni el parser.

Pruebas antes de red: todos los borradores, no filtrado por DTO; aislamiento; no target/gold en payload; igualdad del resto del request; rechazo de padre incompleto/duplicado/HTTPfallido/modelo o recibo inválido, y diagnóstico forjado. Probar también wire revisado y conversión por el adaptador original. Reproducir cuatro bancos manuales y nueve informes previos, además del nuevo.

## Ejecución y puertas

Ocho revisiones HTTP como máximo, cuatro concurrentes, OpenRouter/OpenAI exclusivo, mismo alias openai/gpt-4.1, temperatura0, JSON object, salida4096. Sin fallback de proveedor ni reintentos. Catálogo, estimación de entrada con borrador y duración se anuncian antes de ejecutar. Preparación, fuente ejecutada, diario y recibos nuevos; padres inmutables. No se reejecuta la primera etapa ni se suman dos veces sus cargos al ledger.

Dos críticos nuevos sin historial revisan TODOS los ocho originales/revisados y normalizados disponibles, sin gold, roles ni reportes anteriores. Diferenciar semántica, evidencia, forma y cambios respecto del borrador; conservar también regresiones de DTO/fidelidad. No llamar reparación válida a una salida que solo pasa JSON. Puerta sin rebajar: ocho DTO completos y ocho fieles para ambos, sin ausencia/unsupported/incertidumbre o contradicción de campo. Un fallo impide ampliar. Pasar solo permitiría una ejecución conocida completa, controles difíciles, calidad/recursos y estabilidad antes de V6 ciega; no habilita adopción por sí solo.

## Coste y límites

Reportar gasto nuevo de revisión y coste lógico del mecanismo = los ocho padres + ocho revisiones. El padre ya figura en el ledger histórico y no se factura otra vez en el acumulado. Conservar costes de todos los fallos; ausencia no es cero. Si falta recibo nuevo, el total queda parcial conocido. Separar latencias de ambas etapas; son ejecuciones históricas separadas, no tiempo end-to-end ni suma de medianas. No presumir ahorro amortizado ni pase de recursos por reutilizar padres.

Misma familia/modelo puede repetir errores y copiar un borrador equivocado. La crítica de fidelidad independiente del borrador permanece necesaria; no se presenta autocorrección como prueba lógica de verdad. La comparación usa primeros borradores históricos y no sustituye una prueba completa contemporánea de la tubería ni estabilidad. No se alteran gold, snapshots001–056, baseline, README/gráficas vigentes o V6. Sin nuevas dependencias, secretos, SQL, puertos, servicios o cambios del dashboard.
