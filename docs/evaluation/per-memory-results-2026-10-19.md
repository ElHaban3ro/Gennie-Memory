# Evaluación por recuerdo: resultados conservados

2026-10-19. Goal activo, sin promoción. Se implementó un juicio por recuerdo y composición de todos los positivos; la selección y auditoría inspeccionan el conjunto entero. Dos evidencias compatibles ya no se vetan por tener IDs diferentes. Esto prueba el mecanismo de composición, no su precisión semántica.

| Variante | Alcance conocido | Resultado | Coste USD |
| --- | --- | --- | ---: |
| V1 Jev,16 | Un piloto técnico |16juicios; conserva ambas partes; coste1,7866× recibo histórico de la misma pregunta. No se amplía. |0.000117558 |
| V2 Jev,8 | Un piloto técnico |8juicios; conserva ambas partes; ratio diagnóstico1,0226×. |0.000067284 |
| V2 Jev,8 |64pares/128operaciones |5preguntas con recuerdos ajenos añadidos; recursos pasan. Calidad falla. |0.007766678 ambas políticas |
| V3 Jev,8, pregunta repetida |8diagnósticos |4de5errores originales persisten; complementarias conservadas. No se amplía. |0.000632982 |
| V4 Decisions,8, compacto |10diagnósticos |0selecciones ajenas;2preguntas pierden evidencia permitida de base. Coste diagnóstico2,344× histórico. No se amplía. |0.001221300 |

V3 cambia tanto ubicación de consulta como instrucciones y criterios; V4 cambia texto y proveedor. No se atribuye causalidad a un único factor. Los diagnósticos elegidos entre errores no son una estimación representativa.

## Campaña pareada válida V2

| Categoría | Base | V2 | Total |
| --- | ---: | ---: | ---: |
| Apoyo: semilla enteramente permitida |16 |15 |16 |
| Refutación: semilla enteramente permitida |15 |13 |16 |
| Parcial: semilla enteramente permitida |15 |15 |16 |
| Ausencia: semilla vacía |15 |16 |16 |

V2 registra cinco selecciones contaminadas, aunque incluya también evidencia buena; no se ocultan con el primer ID. Recupera dos omisiones y añade recuerdos permitidos en doce preguntas, pero eso no compensa los cinco errores. Los ID añadidos confunden tiempo de uso con tiempo de clasificación, recogida con adquisición de propiedad, carné requerido con exclusión de avalista, otro objeto con el solicitado y ubicación del contador con ubicación de la sonda.

Coste base USD0,003480200; V2 USD0,004286478 (1,2317×). Tokens de entrada reportados34802 frente a102059. p50 de operación378,71 frente a368,06ms; p951095,00 frente a408,53ms. Esta cola de red pequeña no garantiza una mejora general. Todas las operaciones y recibos completos. Bytes de semilla p95:208,85 frente a488,70; más evidencia también aumenta contexto.

## Error de lanzamiento separado

El coordinador lanzó inicialmente las128filas con opciones globales Decisions/baseline en vez de per_case/per_case. Esas128operaciones costaron USD0,006960400; no evalúan V2 y se excluyen de comparaciones de calidad. Se conserva el diario original y se incluye todo su coste en el acumulado. El runner ahora rechaza contradicciones entre opciones globales y casos explícitos antes de hacer llamadas; prueba de regresión pasa. La comparación correcta usa otro directorio y verificó las primeras dos rutas antes de continuar126operaciones. No se repitieron llamadas inciertas.

## Validación y siguiente requisito

73pruebas workspace pasan. Se corrigió un límite real del compositor: API pública mantiene ocho; ruta automática V1 admite dieciséis. Pruebas SQLite conservan9/16positivos y rechazan9en la API pública. La auditoría revisa todos los textos, revisiones, fuentes, ámbitos y recibos. El evaluador rechaza positivos omitidos, ID extra y cambios de revisión; conserva cada ID permitido de base. La relación resumida por pregunta se etiqueta como la del primer ID, separada de la calidad del conjunto.

Banco V6 congelado:224recuerdos,64preguntas, revisión ciega separada más conciliación y dos reformulaciones previas;17apoyo,16refutación,15parcial,16ausencia. Se preservan autor/revisión/preguntas iniciales y todos los desacuerdos. Tiene cobertura explícita por parte solicitada, separando partes disponibles de datos no documentados. Esa cobertura no verifica por sí sola todos los calificadores del contexto ni verdad de una respuesta final. No tiene llamadas remotas y espera un mecanismo que supere desarrollo. La revisión entre agentes de igual familia no es revisión humana ni independencia de modelo.

Próximo paso de investigación: diseñar una prueba mínima que distinga fallo de atribución semántica de sensibilidad al formato/proveedor, conservando dos evidencias complementarias. No ampliar campañas de variantes ya fallidas, bajar las puertas ni usar V6 para ajustar prompts. El predeterminado continúa Decisions/baseline; README sin nuevas cifras experimentales.
