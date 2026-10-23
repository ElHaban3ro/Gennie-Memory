# V8: preservar decisiones correctas de TRAIN

Fecha:2026-10-22. Paso3 activo. [Protocolo previo](utility-learning-v8-preservation-protocol-2026-10-22.md). Nuevo contraste exploratorio adaptativo tras V7, con TRAIN exactamente idéntico en ambos brazos y cambio único de objetivo. No altera producto ni acredita generalización fresca.

## Calidad

| Serie | Padre: aciertos / útiles / ajenos | Control | Preservación |
| --- | --- | --- | --- |
| background | 155/176 · 70/72 · 19 | 169/176 · 71/72 · 6 | 166/176 · 71/72 · 9 |
| pool | 219/256 · 101/128 · 10 | 226/256 · 106/128 · 8 | 226/256 · 106/128 · 8 |
| development | 14/24 · 10/12 · 8 | 16/24 · 10/12 · 6 | 15/24 · 10/12 · 7 |
| known | 59/64 · 13/17 · 1 | 56/64 · 10/17 · 1 | 57/64 · 11/17 · 1 |
| attribution_probe | 7/8 · 3/4 · 0 | 8/8 · 4/4 · 0 | 8/8 · 4/4 · 0 |
| cleft_probe | 32/32 · 16/16 · 0 | 29/32 · 16/16 · 3 | 30/32 · 16/16 · 2 |

El reranker histórico sin adaptar fue 13/24 en desarrollo,12/12útiles,11ajenos;53/64conocidos,16/17útiles,10ajenos. Referencia de reranker, no ruta productiva FTS5/Decisions. Los seis conjuntos son TRAIN o material repetidamente observado; tampoco los no seleccionados del pool son validación independiente.

## Puertas y decisión

**control**: puerta24/24+64/64=False; preservación del padre en seis conjuntos=False; integridad=True; elegible local=False.

- pool: útiles perdidos `hpa_q01_m01`; ajenos nuevos `hpb_q16_m04`.
- known: útiles perdidos `cv3_08_02, cv3_14_00, cv3_16_00`; ajenos nuevos `ninguno`.
- cleft_probe: útiles perdidos `ninguno`; ajenos nuevos `cdp_19, cdp_20, cdp_24`.

**preserve**: puerta24/24+64/64=False; preservación del padre en seis conjuntos=False; integridad=True; elegible local=False.

- pool: útiles perdidos `hpa_q01_m01`; ajenos nuevos `ninguno`.
- known: útiles perdidos `cv3_08_02, cv3_14_00`; ajenos nuevos `ninguno`.
- cleft_probe: útiles perdidos `ninguno`; ajenos nuevos `cdp_19, cdp_20`.

Candidato para próxima validación según regla anterior al modelo: **ninguno**. Si ambos pasaban se prefería control. Una ventaja relativa preserve/control no sustituye las puertas completas ni la comparación contra el padre. Ningún resultado local acredita adopción.

## Mecanismo probado

Mismo padreV5matched7522394b350d8843d300420f622695a638ce20bb2e6c90fe723a3cf1990c6a74. Mismas224filasTRAIN V7hard,96útiles/128none,orden,tokens,unaépoca56updates,presupuesto,optimización y umbral0,5. Mantiene LoRAFP32r8q/vúltimas8capas327.680parámetros/baseFP16. AdamWfresh lr1e-4,acumulación4,clipglobal1. La selección hard no se demostró superior en V7: se fija como material para aislar el nuevo contraste de loss.

`L0=CE+y·relu(2−m)^2`; `m=logit_yes−logit_no`, `s=(2y−1)m`. Teacher canónico inicial del control: `t=min(8,max(0,(2y−1)m_parent))`, máscara solo cuando su decisión coincide con goldTRAIN. `L=L0+λ·mask·relu(t−s)^2`: λ0control/λ1preserve, sin normalizar por cantidad masked. No hay búsqueda de coeficiente/cap/época/semilla/checkpoint. Cap8 limita el target de margen, no la loss o el gradiente. No se asume confianza calibrada.

Teachers canónicos:224filas, 191 protegidas por coincidencia con gold; 83 positivas y 108 negativas. SHA idéntico en ambos brazos. Los errores del padre quedan sin penalización de preservación; pueden corregirse por L0. El mask se congela, no se actualiza según resultados posteriores.

Se usan logits reales de los forwards iniciales ya previstos, no logits reconstruidos desde probabilidades. Teacher fuera deAD, whitelist224exacta por fuente,ID,label,role,texto ytokens. Conocidos88,sondas8/32 yno seleccionados208delpool no entran enloss/teacher. Preserve verifica logits/márgenes iniciales≤0,05 y decisiones iguales, pero usa exactamente el teacher del control.

## Activación observada del mecanismo

| Brazo | Pasos con retención activa | Media penalización de retención | Media loss base |
| --- | ---: | ---: | ---: |
| control | 0/224 | 0.000000 | 3.225046 |
| preserve | 48/224 | 0.350245 | 3.600378 |

Retención final del subconjunto TRAIN protegido: control: 191/191, preserve: 191/191. El control ya retenía todas esas decisiones; esta corrida no demuestra una ventaja de preservación dentro de ese subconjunto. La evaluación externa al TRAIN sigue siendo material observado, no validación fresca.

La loss total entre brazos incluye términos distintos y no es una métrica comparable de calidad. Activación no demuestra preservación semántica fuera de TRAIN.

## Repetición y auditoría

Control repite TRAIN/objetivo de V7hard; cambios de decisión por conjunto frente a esa corrida: background=0, pool=0, development=0, known=0, attribution_probe=0, cleft_probe=0. Es una repetición concreta, no prueba de estabilidad poblacional.

Diferencia máxima absoluta del primer update entre brazos: 0.0. Se guardaron los32arrays y los cuatro ejemplos previos; comparación descriptiva con iniciales comunes, no garantía de igualdad de updates posteriores.

- control: 55895/55895 comprobaciones offline;2692eventos;1120eval+224back+56updates;terminal0real observado y proceso ausente.
- preserve: 52310/52310 comprobaciones offline;2692eventos;1120eval+224back+56updates;terminal0real observado y proceso ausente.

Preflight independiente3.807/3.807antesdeinferencia. Cada brazo ejecutó140comprobaciones escalaresAD de loss/componentes/derivadas antes del modelo; no se cuentan como forwards/backwards del transformer. Auditor posterior separado del autor reconstruye logits→margen→score,224teacher/máscara/targets/tokens,componentes,orden,eventos,32arrays/checkpoints ypuertas. Se preservan fuentes de auditor y preflight. No se persistieron gradientes/estadosAdamW/basefinal completos: fingerprints registrados no son reconstrucción independiente continua de esos tensores.

Implementación213hashes ybinario de evaluación sin cambios. No se repitió Cargo:879predeterminadas/881HTTP2(24/75ignoradas) heredadas de las mismas fuentes verificadas. SinDB/secretos/dependencias/descargas/servicios/ramas/commits. README ygráficas productivas conservan su evidencia anterior.

## Recursos y coste

| Brazo | Tiempo medido | Forward transformer | Backward transformer | Updates |
| --- | ---: | ---: | ---: | ---: |
| control | 180.281 s | 1344 | 224 | 56 |
| preserve | 339.169 s | 1344 | 224 | 56 |

Total modelos:519.450s,2688forward,2240scores_eval,448back,112updates. Preparación/revisión/auditoría excluidas; no es latencia productiva ni validación de p50/p95≤1,5×.

API nueva:0llamadas/USD0. Coste Codex y económico local desconocidos, no se declaran gratuitos. Histórico remoto intacto:6155operaciones/23campañas emparejadas completas/USD4,566379428conocidos+24operaciones de coste desconocido.

## Continuación

V8 se rechaza: la preservación recupera un útil frente al control, pero sigue perdiendo dos frente al padre y añade un error de desarrollo respecto al control. Ambos conservaban191/191TRAIN protegidos; la restricción no demuestra una ventaja binaria allí. Diagnóstico posterior de80paresTRAIN con revisión ciega nueva:80/80acuerdo,0ambigüedades,32parciales; sin cambiosde gold o métricas. C compuesto10/16 ycomponente9/16,idénticos porID enpadre/control/preserve; directos/ausencia8/8,none40/40. No respalda un fallo exclusivo de totales. Próxima hipótesis: alinear explícitamente la unidad operativa de selección con evidencia útil parcial, mediante UNA instrucción nueva contra la actual con el mismo padre y textos originales. La instrucción vigente YA menciona parcialidad: no se presenta como un requisito ausente o causa demostrada. Contraste futuro separado, congelado y criticado antes de scores, sin más penalizaciones/minería, sin instrucciones elegidas por los88 y sin prometer resultado.

La reserva original24 y V6final siguen cerradas. Solo un pase local completo permitiría casos nuevos con revisión ciega; después siguen comparación productiva completa,recursos/recibos/estabilidad antesdeincorporar. Objetivo íntegro activo; permisos resueltos. Artefactos en `outputs/iromem/evaluation/utility-learning-v8-2026-10-22/`.

## Diagnóstico posterior de evidencia parcial

Se revisaron de nuevo80paresTRAIN existentes (16gruposC ylos4primeros gruposA ordenadoslexicográficamente), con IDsneutros y revisión nueva sin etiquetas/scores. Concordancia80/80roles/labels,0ambigüedades;4respuestas completas,4ausenciasexplícitas,32parciales,40none. Citas/hashes/IDs verificados. Revisión nativa de mismafamilia, no humana ni validación nueva;gold y métricas originales intactos.

| Subtipo | Padre | Control | Preservación |
| --- | ---: | ---: | ---: |
| Parcial en consulta compuesta | 10/16 | 10/16 | 10/16 |
| Componente de total | 9/16 | 9/16 | 9/16 |
| Directo/ausencia explícita A | 8/8 | 8/8 | 8/8 |
| Ajenos, rechazo correcto | 40/40 | 40/40 | 40/40 |

Las decisiones individuales de los32positivosC, no solo su agregado, permanecen iguales entre los tres estados. TRAIN seleccionado difícil conserva3/8yresto16/24: selección por errores impide atribuir causalmente la diferencia a entrenamiento. Tampoco comparar A/C no emparejados en dominio demuestra una causa. La hipótesis exclusiva de totales no queda respaldada: ambos subtipos parciales fallan. Artefactos yrevisión en `outputs/iromem/evaluation/partial-evidence-coverage-v1-2026-10-22/`.
