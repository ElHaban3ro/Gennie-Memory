# 0032 — Un piloto de adaptación local a utilidad documental

Estado: ensayo completado y rechazado por pérdida útil; sin integración ni adopción. Fecha: 2026-10-22.

El reranker original conserva una inversión entre evidencia útil y ajena que ningún umbral monótono sobre esos scores fijos resuelve. La optimización MLX con prefijo exacto sí supera viabilidad instrumental, sin mejorar esas decisiones. Se prueba una hipótesis distinta: aprender la relación de utilidad con datos nuevos revisados, preservando el modelo base y los errores originales.

Una sola corrida LoRA q/v en las últimas ocho capas, rango8 y multiplicador8, adapta únicamente32tensores FP32 sobre base congelada FP16. Seis épocas sobre48pares, acumulación4,72actualizaciones AdamW predefinidas. La única candidata son los pesos finales; developmenty64regresiones conocidos solo abren o cierran puertas. Los24cerrados yV6no se usan para entrenar, decidir hiperparámetros o rescatar checkpoints.

El criterio no es reducir la pérdida de entrenamiento: exigir cero errores en development/regresiones, ninguna pérdida útil nueva y al menos una ganancia. La reserva pequeña se abre una vez solo tras ese pase. Si falla, conservar originales, pesos, errores y diagnóstico; no promover promedios que escondan pérdidas. Recorrido completo, comparación contemporánea de recursos, estabilidad yV6siguen siendo requisitos posteriores del objetivo original.

Coste: una ejecución local temporal, estimada5–15minutos y limitada a30minutos, con entorno/pesos compartidos ya existentes; sin APIs, instalaciones, dependencias productivas, servicios o cambios de DB/configuración. Energía/amortización/Codex desconocidos. Los adaptadores y diarios quedan fuera del producto. Mantener APIUSD0 separado de coste económico total.

[Protocolo previo](../evaluation/utility-learning-v1-protocol-2026-10-22.md) fija datos, pesos, semilla, orden, optimizador, controles, cómputo y puertas. Código oficial instaladoMLX0.32.4/mlx-lm0.32.2 verificado; hashes de API ligados al runtime antes de inferencias.

[Resultado](../evaluation/utility-learning-v1-results-2026-10-22.md):33→44/48train,13→17/24desarrollo,53→54/64conocidos, pero16→8/17útiles conservados. Puerta fallida; closed/V6sin abrir. No se selecciona checkpoint alternativo ni se calibra umbral retrospectivamente.

Contraste prospectivoV2: mantener datos/semilla/órdenes/base/adaptadores/presupuesto/corte y cambiar solo pérdida aCE+y·max(0,2−m)², peso1. La penalización se activa tras caer un útil por debajo del margen2; no garantiza preservación y puede aumentar ajenos. [ProtocoloV2](../evaluation/utility-learning-v2-protocol-2026-10-22.md) fija un único ensayo antes de scores, con gradientes escalares comprobados, hashes iniciales iguales aV1y evento de checkpoint previo a evaluación. Desarrollo pasa a observado; reservas intactas.

[ResultadoV2](../evaluation/utility-learning-v2-results-2026-10-22.md): mejora59/64conocidosy14/17útiles respecto aV1, pero pierde dos útiles adicionales frente a base y conserva2ajenos; falla. Una corrida completada, reserva intacta. Siguiente hipótesis: ampliar cobertura con datos nuevos revisados, sin barrer penalizaciones ni entrenar bancos conocidos.

[Contraste V3](../evaluation/utility-learning-v3-results-2026-10-22.md): dos brazos prospectivos de 264 actualizaciones, original 48×22 épocas y ampliado 176×6. Ambos fallan: conocidos 56/64 con 10/17 útiles frente a 55/64 con 9/17; un ajeno cada uno. Extensión 108→124/128, pero en el segundo brazo está entrenada. No se adopta ninguno ni se abre reserva. Más actualizaciones y esta ampliación no cumplen la preservación útil; revisar residuos y fijar una intervención causal antes de otra corrida, sin barridos o rescates retrospectivos.

[Contraste V4](../evaluation/utility-learning-v4-results-2026-10-22.md): agrupación por consulta frente a control contemporáneo por filas, mismos 176 pares/seis épocas/264 actualizaciones y pérdida. Conocidos: 55→59/64 y útiles 9→13/17, un ajeno en ambos; todavía tres pérdidas útiles adicionales respecto a base. Ajenos entrenados 5→29 y desarrollo 4→8: no mejora general de discriminación. Integridad 3899 comprobaciones posteriores más 65 previas; ambos fallan calidad. Reporte agrupado y proceso terminal confirmados tras perder el handle, código de salida desconocido conservado. No adopción ni apertura de reserva.

[Contraste V5](../evaluation/utility-learning-v5-results-2026-10-22.md): control por consulta frente a permutación igualada por target binario TRAIN y origen en cada posición, mismos 176 pares/seis épocas/264 updates. Ambos obtienen 59/64 conocidos, 13/17 útiles y un ajeno, con tres pérdidas útiles nuevas respecto a base. Extensión entrenada 108→117/128 y desarrollo 15→14/24; no se acredita ventaja semántica de compartir consulta ni calidad suficiente. Integridad 3956 comprobaciones posteriores y 90 previas, ambos exit0 observados. Sin adopción o reservas. Diagnóstico directo antes de otra variante de sampler/hiperparámetro.

[Diagnóstico de margen](../evaluation/utility-gradient-v1-results-2026-10-22.md): matriz de los 176 train en matched congelado, 180 forward/backward y cero updates. Jacobianos no nulos; 104 correcciones ideales opuestas, con 3/4 controles numéricos incompatibles que limitan reconstrucción. No prueba capacidad global o trayectoria AdamW. Estudio posterior de igualdades lineales factible y revisado 12/12, delta sin aplicar y hasta 72,39% de cambio relativo por bloque: siguiente replay único prospectivo solo train, sin buscar escalas ni abrir reservas. Calidad V5 continúa fallida; integridad 10137, no adopción.

[Replay no lineal fijo](../evaluation/utility-gradient-replay-v1-results-2026-10-22.md): mejora real TRAIN 155→174/176, útiles70→72/72,ajenos19→2,sin errores nuevos; una delta coef1 sobre copia, sin gradiente/optimización/búsqueda de escala. Compatibilidad lineal falla (4/176), no asumir garantía de márgenes. Candidato congelado, auditoría9194y307preflight. Siguiente evaluación prospectiva separada24desarrollo+64conocidos observados; no generalización fresca/adopción/reservas por fit.

[Puerta local del candidato fijo](../evaluation/utility-gradient-local-gate-v1-results-2026-10-22.md):88casos completos,dev16/24yknown56/64,útiles20/29,ajenos7. Contra padre dos ganancias y tres pérdidas útiles nuevas; gatefallida pese aTRAIN174/176. No adopción/reservas ni ajuste sobre observados. Revisión nativa técnicamente566checks válidos pero temporalmente posterior al inicio, conservada como tal; API0/Codexdesconocido. Diagnosticar cobertura TRAIN sin nuevos modelos antes de otra intervención.

[Sonda de realización](../evaluation/attribution-realization-probe-v1-results-2026-10-22.md): ocho casos×tres estados,24forward/0backward/0updates. Revisión ciega8/8,corrección accesoria anterior a inferencia,preflight nativo previo. Resultado completo y contrastes en informe; no reevalúa puerta fallida ni autoriza reserva/adopción,API0,Codexdesconocido.

[Contraste V6 de diversidad y repetición](../evaluation/utility-learning-v6-results-2026-10-22.md): mismo padre y208posiciones,32nuevosB/none frente a32antiguosf05×4,fondo176conservado,1época/52updatesporbrazo. Corpus nuevo32/32ciego y preflight1803previo; todos los resultados y cambios por caso conservados. No se adopta por mejora en TRAIN o sonda; puerta original y reservas intactas.

[Sonda escindida/directa](../evaluation/cleft-direct-probe-v1-results-2026-10-22.md):32celdas procedentes de16nominalesTRAIN,cuatro estados exactos,128forward0backward/updates. Cambia solo realización de agente con16equivalencias y32labels revisados a ciegas antes de modelo. No es entrenamiento ni mejora de la puerta original; no adopción/reservas.

[Selección difícil B/C](../evaluation/utility-learning-v7-results-2026-10-22.md): P1/P2 fallan soporte A y no entrenan; P3 nuevo contraste adaptativo con A compartido, 224 pares y56updates por brazo. Comparación local, daños por ID y puertas originales intactas; candidato=ninguno. Sin adopción ni reservas; API0, coste Codex/local desconocido.

[V8 preservaciónTRAIN](../evaluation/utility-learning-v8-results-2026-10-22.md): mismo224TRAIN/56updates por brazo;teacher canónico común/maskgoldcorrecta/logits reales;λ0vs1/cap8 fijados antes. Candidato=ninguno,sinadopción/reservas;API0,Codex/local desconocido.

[Prefijo unidad de evidencia](../evaluation/utility-unit-instruction-v1-results-2026-10-22.md): mismo padre/preguntas/documentos/corte;sinAD/updates;gateTRAIN antesde128condicionales. Candidato=ninguno,sinadopción/reservas;API0,Codex/local desconocido.

[Ablación del residual LoRA](../evaluation/adapter-contribution-ablation-v1-results-2026-10-22.md): 432 TRAIN por estado, 864 forwards, 0 AD/actualizaciones. 13 C y 128 none exclusivos off; comprobación posterior descarta rescate sin daños mediante funciones globales monótonas de ambos márgenes. Sin selector/adopción ni apertura de reservas; la siguiente hipótesis debe aportar señal semántica adicional. API 0; coste Codex/local desconocido.

[Contrato de vínculo semántico](../evaluation/semantic-link-contract-v1-results-2026-10-22.md): diagnóstico nativo ciego de 52 TRAIN existentes, 51/52 concordantes y 1 indeterminado. Puerta prospectiva no superada, sin cambiar gold ni convertir fichas manuales en predictor. API 0; coste Codex/local desconocido.

[Sonda de readout congelado](../evaluation/frozen-utility-readout-v1-protocol-2026-10-22.md): tras fallar las fichas generativas, se propone una cabeza logística sobre hidden1024. Antes de medir se cambia el encoder al base original sin LoRA: usar V5 adaptado produciría exposición previa del encoder a176TRAIN. Una sola receta y OOF agrupado por fuentes; no más adaptación del transformer, ni afirmación de generalización fresca o adopción.

Readout congelado completado:343/432 OOF,157/200 útiles,46 ajenos y daños nuevos22/24 frente al padre. Cinco fits perfectos en sus TRAIN no transfieren al componente excluido; no adopción, umbral o regularización alternativos. 432forwards/0backwards del encoder,41pasosCPU,50,05s,API0. Se inicia ampliación estructural deTRAIN128, no reserva de evaluación. [Informe](../evaluation/frozen-utility-readout-v1-results-2026-10-22.md).


[Contraste de diversidad con readout fijo](../evaluation/utility-readout-diversity-v1-results-2026-10-22.md):128TRAIN nuevos en16familias, revisión fuente128/128 y exclusión de homólogos. Control343/432 idéntico151; ampliado328/432,165/200útiles y69ajenos, frente al padre16pérdidas útiles/45ajenos nuevos pese a7ganancias C. Falla y no se adopta.128forwards nuevos,432cacheados,10fits/81pasosCPU,19,80s,API0. Mayor cobertura no mejora separación en este contraste; diagnóstico posterior registra deterioro concentrado en componentes312/72, sin cambiar labels, umbral, receta o reservas.


[Intervención de normalización](../evaluation/readout-normalization-mediation-v1-results-2026-10-22.md): mismos560features/etiquetas/folds/TRAIN ampliado, estadísticas del control;cincofitsCPU/40pasos,ceroencoder/API.326/432 frente a328del ampliado,mismos165útiles ydosajenosmás;hipótesis no apoyada,puerta fallida,sinadopción/reservas. No más variantes de normalización; diagnóstico siguiente de sensibilidad a la pregunta con documento fijo, previo a otra modificación del predictor.


[Diagnóstico de sensibilidad](../evaluation/query-binding-sensitivity-v1-results-2026-10-22.md):18filas/ochofuentes fijas;diezconsultas nuevas y lectores persistidos sin fits. Control16/18,dos ternasB correctas;ampliado8/18,estadísticasfijas7/18,base10/18. La ampliación perjudica conducta que el mismoencoderpermite alcontrol;noatribuir todo aausenciatotaldeseñal. InversiónBútil/none impide rescateporcorteúnico. Preflight396,auditorv2aprobado conbugdelprimero preservado;10forwards,API0,sinadopción/reservas.


[Ensayo de tres cabezas](../evaluation/rolewise-utility-readout-v1-results-2026-10-22.md): quince lectores OVR A/B/C con OR fijo, mismos560features/datos/folds.322/432 frente a328del binario ampliado,150/200útiles,60ajenos;33pérdidas y41ajenos nuevos frente al padre. Falla sin cambiar agregación/λ/corte. Quincefits/100pasosCPU,0encoder/API. La siguiente hipótesis es supervisión por consultas opuestas con documento fijo y control de repeticiones, no otra variante de OR. Reservas/producto intactos.


[Contraste de consultas opuestas V1](../evaluation/query-conditional-training-v1-results-2026-10-22.md):323→331/432 emparejados,168útiles y77→69ajenos; dos pérdidas útiles/dos ajenos nuevos vscontrol y12/43vs padre, calidad fallida sin adopción. Cierre semántico previo de cuatro componentes objetivo para ambos brazos; normalización común background560. Roles/estilo no emparejados, no efecto puro de condicionalidad.32forwards/ocho fits/64pasosCPU/API0. Ampliación final a128fuentes con receta fija; si falla, descartar esta variante.


[Ampliación final de consultas opuestas V2](../evaluation/query-conditional-training-v2-results-2026-10-22.md): 336/432 frente a 324/432 del control; 172 útiles en ambos, 80→68 ajenos, pero cinco pérdidas útiles y tres ajenos nuevos emparejados. Contra padre: diez pérdidas/42 ajenos nuevos, seis ganancias C. Variante rechazada; sin más expansión ni barridos de este lector. 96 forwards/ocho fits/64 pasos CPU, API USD 0. Cierre semántico corregido antes de inferir; V1/152 conservados como históricos.


[Interacción pregunta–documento V1](../evaluation/query-document-interaction-v1-results-2026-10-22.md): interacción pregunta–documento completa 355/432 frente a 336/432 del control reproducido, con 169/200 útiles y 46 ajenos; 16 pérdidas útiles y 11 ajenos nuevos frente al control. Puerta de calidad fallida; candidata rechazada. 859 forwards nuevos, ocho fits CPU, API USD0. Objetivo activo paso3, 2/5 etapas cerradas; sin adopción ni reservas. La resta y normalización propia constituyen una intervención conjunta, sin prueba de causalidad semántica.


[Verificador de relaciones entre tokens V1](../evaluation/token-relation-rescue-v1-results-2026-10-22.md): módulo nuevo de36.929parámetros sobre tabla léxica congelada, aplicado solo a232abstenciones del padreV5. Tres fits/708actualizaciones en3,87s; conserva374/432pero no rescata ningún útil. Auditoría NumPy idéntica en decisiones y diagnóstico TRAIN0/58útiles seleccionados al corte fijo. No es solo transferencia ni predictor literalmente constante: BCE supera el mejor constante en los tresTRAIN. Candidata rechazada, sin bajar corte o rescatar épocas/checkpoint, sin adopción/reservas.


[Objetivo balanceado del verificador](../evaluation/token-relation-balanced-v1-results-2026-10-22.md): mismo modelo/init/orden/receta161salvo pesosTRAINporclase. TRAIN53/58útiles yAUC mayor en los tres grupos; OOF0/29útiles/35ajenos nuevos,339/432 compuesto frente a374control. Candidata rechazada: no cambiar cortes/pesos/checkpoint. Tres fits/708updates en2,84s,API0,auditoría aprobada; sin producto o reservas.
