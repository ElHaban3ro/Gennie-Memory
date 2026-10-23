# Contraste de unidad de evidencia en la instrucción

Fecha:2026-10-22. Paso3 activo. [Protocolo previo](utility-unit-instruction-v1-protocol-2026-10-22.md). Una candidata única, un prefijo de dos frases y control íntegro; pesos y datos congelados, sin AD ni entrenamiento. Hipótesis adaptativa al diagnóstico anterior sobre TRAIN; no generalización nueva.

## Resultado del filtro TRAIN

| Conjunto | Control: aciertos / útiles / ajenos | Prefijo de utilidad |
| --- | --- | --- |
| background | 155/176 · 70/72 · 19 | 147/176 · 72/72 · 29 |
| pool | 219/256 · 101/128 · 10 | 215/256 · 105/128 · 18 |

Gate adicional TRAIN: **fallido**. ÚtilesC recuperados frente alcontrol: 1; IDs `hpc_q06_m01`.

- background: nuevas pérdidas útiles `ninguna`; nuevos ajenos `x_f13_q05_m02, x_f13_q06_m02, x_f14_q04_m02, x_f14_q05_m03, x_f14_q05_m04, x_f14_q06_m03, x_f15_q04_m03, x_f16_q02_m01, x_f16_q04_m02, x_f16_q06_m02`; ganancias totales 2; degradaciones 10.
- pool: nuevas pérdidas útiles `ninguna`; nuevos ajenos `hpa_q02_m03, hpa_q04_m03, hpa_q06_m03, hpa_q11_m03, hpa_q12_m03, hpa_q15_m03, hpa2_q12_m03, hpa2_q13_m03`; ganancias totales 4; degradaciones 8.

El filtro exigía≥1ganancia útilC y cero daños nuevos porID en432filas. Es una condición adicional: no sustituye la puerta original24/24+64/64 ni la preservación de seis conjuntos. El material TRAIN completo incluye filas no presentadas en los entrenamientos anteriores; ya estaba observado y revisado, no es reserva ni validación fresca.

## Evaluación condicional no ejecutada

El fallo TRAIN detuvo el contraste antes de cargar/puntuar desarrollo24,conocidos64 ysondas8/32. Son128pares×dosinstrucciones=256forwards no ejecutados. **No hay nuevas métricas para esos conjuntos**: puerta original y preservación de seis conjuntos figuran `null`/no medidas. El rechazo local ya está determinado por el requisito previo fallido; no se imputa una precisión inventada al banco no evaluado.

Candidato para siguiente validación: **ninguno**. No se adopta un mecanismo en esta fase. Reservas original24 yV6finalcerradas; producto sin cambios.

## Qué cambió

Solo se antepusieron estas dos frases a la instrucción previa, que permanece íntegra:

> Evaluate the document as one piece of evidence for a later answer, not as a standalone complete answer. One actual requested part or one explicit component of a requested total can be useful even when the other parts are absent and the total cannot be computed.

El control ya permite evidencia parcial. No se afirma que faltara esa regla ni que la completitud causara los fallos anteriores. Se mide el efecto del paquete de prefijo y su longitud/posición, no una causa semántica pura. No hubo búsqueda entre candidatos, ejemplos de los88, cambios de preguntas/documentos/gold, nuevos thresholds, training o actualizaciones.

Mismo padreV5matched SHA7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74,baseFP16+32LoRAFP32,327.680parámetros,fullyfrozen/modoeval. Una sola cargaMetalGPU. No AD/optimizador. Instrucciones/inputs/tokens/código/revisión/runtime congelados antes de inferencia; control con score≤0,02 y mismas decisiones que referencias anteriores.

## Verificación

Preflight nativo independiente2.912/2.912 antes del modelo; auditoría offline 22234/22234 comprobaciones. Hashes/tokenIDs/logits→margen→score/gate porID/orden/eventos/terminal reconstruidos sin nueva inferencia. Base yadaptador permanecieron idénticos por fingerprints registrados antes/después; archivo padre intacto. Fingerprints finales no son prueba de inmutabilidad continua ni reconstrucción independiente de todos los tensores no guardados.

Antes de freeze, py_compile detectó un error de espacios en la sintaxis condicional. Borrador y reparación preservados; cero ejecuciones o inferencias antes de corregirlo. La carga de128condicionales se dejó bajo gateTRAIN. No se modificó la fuente congelada tras iniciar.

Mismos213hashes de implementación ybinario de evaluación. No Cargo nuevo:879default/881HTTP2(24/75ignoradas) de las mismas fuentes verificadas. Sin cambios enDB/secretos/dependencias/descargas/servicios/ramas/commits. Los resultados previos yfigurasREADME permanecen históricos, sin reemplazarlos por este contraste.

## Tiempo, tokens y coste

CLI total medido: **98.964s**; 864forwards/scores;0backward y0updates. Preparación/revisión/auditoría excluidas. No es latencia ni coste de ruta productiva.

| Instrucción | Tokens medidos en pares ejecutados | Media por par |
| --- | ---: | ---: |
| control | 110265 | 255.243 |
| utility | 131433 | 304.243 |

Estos son tokens del input del clasificador local, **no** tokens enviados aCodex ni un recibo monetario. El prefijo añade input; no se presume ahorro. Recursos completos/recibos/p50/p95≤1,5× yestabilidad siguen pendientes.

API nueva:0llamadas,USD0. Coste económico local yCodex desconocidos, no se declaran gratuitos. Acumulado remoto intacto:6155operaciones/23campañas emparejadas completas/USD4,566379428conocidos+24operaciones de coste desconocido.

## Revisión retrospectiva y continuación

Se rechaza el prefijo: recupera6útiles y0pérdidas útiles, pero introduce18ajenos nuevos; el total TRAIN pasa374/432→362/432. Solo1gananciaC. Los128condicionales porbrazo no se puntuaron y sus puertas permanecen no medidas. Los márgenes subieron en214/232none; el patrón describe aceptación más amplia y no una solución selectiva ni una causa de completitud. No seguir buscando prefijos/umbrales para rescatarlo. Próximo diagnóstico fijo: comparar el mismo reranker sin contribución del adaptador frente al padreV5 sobre estos432TRAIN, con una sola ablación y sin entrenar ni tocar88. La tabla de errores complementarios puede descartar o motivar una hipótesis posterior de combinación, pero no constituirá un clasificador ni una precisión oracle. Cualquier combinación necesitaría mecanismo propio, entrenamiento soloTRAIN, puertas originales completas y validación fresca.

Objetivo íntegro activo y permisos resueltos. Casos nuevos con revisión ciega ycomparación productiva completa solo tras puerta local, antes de considerar adopción. Artefactos:`outputs/iromem/evaluation/utility-unit-instruction-v1-2026-10-22/`.

## Descripción de márgenes, posterior al contraste

| Relación | Pares | Margen yes−no aumentó | Cambio medio |
| --- | ---: | ---: | ---: |
| A | 100 | 96 | 1.146895 |
| B | 45 | 39 | 1.435330 |
| C | 55 | 46 | 1.054048 |
| none | 232 | 214 | 1.571426 |

El aumento de margen aparece también en214/232none, con media superior aC. Es compatible con mayor aceptación general; no prueba un mecanismo interno ni permite atribuir causalmente el resultado a completitud. No se usaron estos márgenes para elegir otro umbral o prefijo. Todo se derivó de los mismos864scores, sin inferencias adicionales.
