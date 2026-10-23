# Motivos explícitos: mismo error de selección, un rol corregido

Fecha UTC:2026-10-20. La candidata supera el mínimo y recursos frente a baseline, pero **falla calidad**. En los18 casos difíciles selecciona exactamente los mismos conjuntos que Perplexity original:17 evidencias útiles y4 selecciones ajenas según el banco conciliado. Corrige el rol de Laurel, sin resolver ninguno de esos cuatro extras. No se amplía ni incorpora. El objetivo sigue activo en el paso3.

## Hipótesis e implementación

La auditoría anterior encontró atribuciones incorrectas con los referentes disponibles; no identificó `none` como causa. Esta ablación prueba si ofrecer motivos concretos de rechazo mejora la discriminación: N_subject,N_scope,N_attribute,N_requirement,N_plan,N_other, todos canonicalizados a none. Mantiene literalmente modelo/revisión Perplexity, estado, reglas, analogías, fuentes, orden, instrucciones y definiciones A/B/C. Solo cambian criteria.

Las definiciones comprueban primero B/A/C y fijan prioridad entre rechazos que se solapen. Una parte útil y un estado explícito pendiente/desconocido siguen admitidos. No se cambia gold ni se crea una etiqueta retrospectiva para motivos. Las razones negativas elegidas son salidas diagnósticas del modelo, no explicaciones verdaderas verificadas.

El contrato puro verifica las9 etiquetas y probabilidades reales, remapea únicamente los6 rechazos a none y rechaza namespace antiguo o añadido. Las pruebas comprueban que no cambia otro campo ni la definición positiva. El runner es manual `cfg(test)`; no existe política runtime nueva. El preparador histórico de2 brazos rechaza esta ablación; un preparador específico exige mínimo reconstruido antes de escribir los3 brazos.

## Mínimo y contraste contemporáneo

Mínimo:16 respuestas válidas,64/64 roles canónicos,17/17 útiles,0 extras/pérdidas, **USD 0,000586260**. Es no regresión en desarrollo conocido, no mejora causal; el Perplexity anterior ya tenía64/64.

Después se congelaron18 casos con3 variantes: baseline real Decisions/hasta20 candidatos, referencia Perplexity/LocalV5 original con7 y candidata Perplexity con9 alternativas.54 operaciones secuenciales,6 permutaciones de orden utilizadas3 veces cada una, un cliente reutilizado. Todas respondieron HTTP/2, sin errores ni reintentos. La revisión estática del crítico no encontró bloqueantes; la verificación principal comparó los54 payloads, conservando control/referencia y cambiando solo los criterios de candidata.

| Métrica difícil conocida | Baseline | Referencia original | Motivos explícitos |
| --- | ---: | ---: | ---: |
| Evidencias útiles conservadas | 11/17 | 17/17 | 17/17 |
| Selecciones ajenas revisadas | 1 | 4 | 4 |
| Errores canónicos de rol | No clasifica roles | 5/126 | 4/126 |
| Mediana de clasificación | 364,09 ms | 436,52 ms | 509,67 ms |
| p95 de clasificación | 945,19 ms | 610,79 ms | 540,46 ms |
| Coste informado | USD 0,001119900 | USD 0,000457460 | USD 0,000989180 |

Candidata/base: coste 0,883×, mediana 1,400× y p95 0,572×, dentro de 1,5×. Son tiempos del harness de clasificación sobre candidatos congelados, no `recall` ni respuesta final. p95 nearest-rank con18 es el máximo; no se retiraron primeras llamadas ni lentas. El gasto experimental incluye las tres variantes; el coste del mecanismo candidato no incorpora sus controles de investigación.

Frente a la referencia contemporánea, candidata cuesta2,162× y conserva los18 sets de selección idénticos. La única diferencia canónica es `cev5_073_17`: aporte parcial de Laurel clasificado A en vez de B. Eso cumple el indicador predeclarado de menos errores de rol sin peor selección; no demuestra estabilidad ni que una categoría negativa concreta causara ese cambio.

Los cuatro extras persisten en ambos brazos Perplexity: `cev4_061_10` (archivo individual frente a política M4), `cev5_068_07` (libreto ajeno), `cev5_069_10`/`cev5_069_12` (chequeo genérico y cita futura frente a autonomía). Se conservan las dos ambigüedades residuales del banco, sin retocar etiquetas. Los4 extras impiden calidad aunque los17 útiles se preserven y se ganen6 sobrebaseline. El gold histórico original y sus discrepancias siguen separados.

En los126 juicios candidatos se eligieron105 rechazos: N_attribute42,N_other39,N_subject20,N_requirement2,N_plan2,N_scope0. No se califica si esos105 submotivos son correctos: solo su canonicalización none tiene gold. Los cuatro recuerdos problemáticos siguieron recibiendo A/B, por lo que ampliar la oferta de rechazo no bastó.

## Decisión

No hay candidata elegible, expansión de campañas, estabilidad, runtime ni V6. La hipótesis de que rechazos concretos resolverían las selecciones ajenas no obtiene apoyo suficiente: ninguno de los cuatro cambió. Se conserva el cambio favorable de rol sin convertirlo en éxito general ni atribuirle una explicación no medida.

La próxima investigación debe actuar sobre la comprobación de relación, no limitarse a renombrar rechazos, añadir campos o bloquear toda evidencia ante desacuerdo. Debe poder corregir un rol útil y distinguir alcance/referente/atributo sin perder partes. Se mantienen todas las puertas originales antes de casos nuevos o adopción.

## Contabilidad y verificación

Tanda:70 operaciones/70 HTTP, **USD 0,003152800**, incluidos mínimo, baseline y referencia. Desglose:16 mínimo/USD 0,000586260;54 triple/USD 0,002566540. Cero costes nuevos desconocidos o llamadas inciertas. Acumulado:5.676 operaciones,23 campañas emparejadas completas, **USD 0,572600028 conocidos +4 operaciones históricas de importe desconocido**. Codex de programación/revisión queda fuera.

259 pruebas workspace pasan con y sin HTTP/2, formato y compilación correctos.15 reportes de esta línea —13 anteriores y2 nuevos— y la auditoría de frames se reconstruyen exactamente. Se conservan bancos, citas, gold y snapshots; V6 mantiene todos sus hashes y0 llamadas. No se iniciaron servicios, crearon conexionesSQLite ni cambiaron procesos del usuario, secretos, dependencias o el default. README y gráficas mantienen la ruta vigente.

Protocolos: [mínimo](negative-choice-contrast-v1-protocol-2026-10-20.md) y [triple](negative-choice-contrast-known18-v1-protocol-2026-10-20.md). Artefactos: `outputs/iromem/evaluation/contribution-local-perplexity-contrast-minimal-v1-2026-10-20` y `contribution-local-perplexity-contrast-known18-triple-v1-2026-10-20`. Cierre de huellas y contabilidad: `quality-goal-2026-10-19/snapshots/041`.
