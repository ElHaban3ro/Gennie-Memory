# Sustitución del segundo crítico: filtro previo con propuestas archivadas

Antes de red. Los pilotos de críticos completos conservan calidad pero fallan p95; HTTP/2medido no lo resuelve. El diagnóstico archivado de familias muestra complementariedad: Jev rechaza la lectura de requisito como exclusión; Microsoft rechaza carné/avalista y tapa verde que Jev acepta. Retirar un crítico sin reemplazar esos rechazos no es una mejora segura.

Hipótesis: sustituir Microsoft por Decisions con la MISMA tarea RoleV5, contexto siete y preguntas dirigidas a las propuestasB. Modelo solicitado openai/gpt-6-luna-decisions, revisión servida exigida openai/gpt-6-luna-decisions-20261006; Jev1.13/revisión20260917 se conserva. No cambiar criterios, gold, testigo, selección A ni regla de intersección. No se incorpora aún política runtime.

Filtro fijo: tomar TODAS las propuestasB aceptadas por rol+testigo del primer Mercury completo del contraste de familias, ligado al snapshot032. Son seis preguntas y once propuestas (cinco permitidas y seis no permitidas por el gold original). La lista se obtiene sin usar gold para elegir propuestas. Mantener todos los recuerdos originales como contexto, preguntar solo esosB, sin respuestas/confianza previas en el payload; guardar mapasID/revisión. Jev y Decisions ven el mismo payload salvo model, orden alternado. Cada recibo global se cuenta una vez.

Máximo12HTTP, estimaciónUSD0,001 y menos de un minuto, sin reintentos. Detener al primer fallo técnico. Archivar respuestas, costes conocidos/desconocidos, fuentes y código antes de red. Este filtro ejecuta llamadas secuenciales para observar juicios; NO mide la latencia de una cadena ni habilita una conclusión de recursos.

Solo pasa si las doce respuestas son válidas y la intersección conserva las cinco propuestasBpermitidas, rechaza las seis no permitidas y conserva el testigo/revisiones. Informar también errores individuales de cada crítico: acuerdo no prueba verdad o independencia. No se exige que cada crítico por sí solo sea perfecto, sí que el mecanismo conjunto cumpla esos controles. Un error o rechazo de una útil impide ampliar.

Un pase permite piloto de18pares con propuestas nuevas Mercury y críticos paralelos Jev+Decisions, medido contra baseline contemporáneo con las puertas originales: cero extra/pérdida actual/histórica/degradación y ganancia, cinco regresiones resueltas; coste/p50/p95≤1,5×base y recibos completos. No se abreV6antes de mecanismo real, campañas completas y estabilidad. No repetir un filtro fallido para seleccionar la mejor respuesta.
