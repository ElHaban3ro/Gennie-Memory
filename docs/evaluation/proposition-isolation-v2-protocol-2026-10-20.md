# Ablación de contexto y tarea de proposiciones V2

Antes de red. V1 acertó36respaldos, falló4roles y3selecciones sobre36proposiciones manuales. No construir extractor para esa versión. Hipótesis exploratoria: compartir tarea de respaldo o fuente interfiere en el juicio condicional de utilidad. Una mejora no demuestra un proceso interno ni garantiza el mecanismo completo.

Mismo banco18preguntas/36proposiciones revisadas, fuentes/preguntas/orden/etiquetas intactos, modelo Perplexity/revisión exacta y contratos. Tres brazos contemporáneos:

- `joint`: payload V1 literal con dos preguntas de respaldo y dos de rol.
- `role_source`: retirar únicamente las dos preguntas de respaldo; conservar todo el estado y las preguntas de rol literalmente.
- `role_only`: partir de role_source y retirar únicamente `state.source`. Mantener pregunta, proposiciones, reglas de respaldo/rol, instrucciones, criterios, claves y modelo. Las instrucciones que prohíben completar desde fuente permanecen idénticas; no requieren leer una fuente inexistente.

Esto permite contrastar por separado presencia de tarea y de campo fuente. No cambiar criterios/ejemplos/redacción ni probar únicamente los cuatro fallos. Proposiciones manuales pueden coincidir con frases de fuente; role_only excluye el campo fuente, no pretende borrar contenido que forma parte legítima de la proposición. No afirma aislamiento estadístico entre tareas ni modelos. Sin etiquetas/IDs de evaluación/factores ni resultados previos en payloads. Respuestas role_only/role_source contienen solo dos roles: no fabricar respaldo ni combinarlo con un voto histórico para presentar selección real.

18triples/54HTTPmáximos, las seis permutaciones de orden repetidas tres veces, un cliente secuencial reutilizado, sin warmups descartados. EstimaciónUSD0,003 y1–2minutos. Fuente/código/protocolo/banco/solicitudes/ejecutablehash congelados antes de llamar. Fallo técnico detiene y conserva recibos/intentos; desacuerdo semántico completa la tanda. Sin reintentos ni repetición favorable. Toda latencia terminada, incluidos fallos, y todo recibo de los tres brazos se cuentan. Costes desconocidos separados.

Preparar exige banco vigente íntegro y reconstrucción exacta del informeV1, con18válidas/0errores de respaldo/4de rol/0coste desconocido. El informeV1negativo permanece inmutable. Parser de cada brazo exige modelo/revisión exactos, namespaces exactos, tipos/enums/probabilidades y no inventa respuestas ausentes. Auditor recompone cada payload, seis órdenes, source/preflight/archive y diarios. Se informan errores de rol por brazo, aportes omitidos, none aceptados y roles útiles mal clasificados, además del respaldo exclusivo de joint, latencias y costes.

Señales exploratorias: menos errores al pasar joint→role_source o role_source→role_only, sin empeorar ninguna relación; solo se interpretan si las54respuestas son válidas, todos los recibos están presentes y el coste total≤USD0,003. Capacidad role_only exige además36/36roles correctos; no se llama a esto pase de selección, extracción, calidad o recursos del producto. No comparar con una corrida histórica para atribuir efecto causal ni compensar una pérdida de C/parte con otro acierto. Una sola tanda no prueba estabilidad.

Si role_only falla, no construir extractor ni ampliar campañas para esta hipótesis como solución suficiente. Si pasa, faltará demostrar extracción automática íntegra, respaldo y composición nuevos sin gold, conservar todas las partes y refutaciones y medir la cadena real con todas las llamadas/coste/p50/p95≤1,5×base; faltarán runtime, campañas completas, estabilidad y casos nuevos ciegos. No abreV6ni cambia producto/default, SQL, dependencias, secretos, servicios, dashboard, README o gráficas.
