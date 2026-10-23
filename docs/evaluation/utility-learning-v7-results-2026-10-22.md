# Selección de casos difíciles B/C con controles A compartidos

Fecha: 2026-10-22. Paso 3 del objetivo activo. Comparación exploratoria adaptativa de selección TRAIN: P3 es posterior a los scores P2 y anterior a las actualizaciones. No modifica el producto ni acredita generalización fresca.

## Resultado

### Desarrollo conocido

| Estado | Aciertos | Útiles conservados | Ajenos seleccionados |
| --- | ---: | ---: | ---: |
| Padre | 14/24 | 10/12 | 8 |
| Uniforme | 16/24 | 10/12 | 6 |
| Difícil B/C | 16/24 | 10/12 | 6 |

### Banco conocido

| Estado | Aciertos | Útiles conservados | Ajenos seleccionados |
| --- | ---: | ---: | ---: |
| Padre | 59/64 | 13/17 | 1 |
| Uniforme | 55/64 | 9/17 | 1 |
| Difícil B/C | 56/64 | 10/17 | 1 |

### Fondo TRAIN

| Estado | Aciertos | Útiles conservados | Ajenos seleccionados |
| --- | ---: | ---: | ---: |
| Padre | 155/176 | 70/72 | 19 |
| Uniforme | 170/176 | 71/72 | 5 |
| Difícil B/C | 169/176 | 71/72 | 6 |

### Pool TRAIN observado

| Estado | Aciertos | Útiles conservados | Ajenos seleccionados |
| --- | ---: | ---: | ---: |
| Padre | 219/256 | 101/128 | 10 |
| Uniforme | 226/256 | 104/128 | 6 |
| Difícil B/C | 226/256 | 106/128 | 8 |

### Sonda de atribución

| Estado | Aciertos | Útiles conservados | Ajenos seleccionados |
| --- | ---: | ---: | ---: |
| Padre | 7/8 | 3/4 | 0 |
| Uniforme | 8/8 | 4/4 | 0 |
| Difícil B/C | 8/8 | 4/4 | 0 |

### Sonda escindida/directa

| Estado | Aciertos | Útiles conservados | Ajenos seleccionados |
| --- | ---: | ---: | ---: |
| Padre | 32/32 | 16/16 | 0 |
| Uniforme | 30/32 | 16/16 | 2 |
| Difícil B/C | 29/32 | 16/16 | 3 |

## Desglose del pool TRAIN por relación

| Relación | Padre | Uniforme | Difícil B/C |
| --- | ---: | ---: | ---: |
| A | 64/64 | 63/64 | 63/64 |
| B | 18/32 | 22/32 | 24/32 |
| C | 19/32 | 19/32 | 19/32 |
| none | 118/128 | 122/128 | 120/128 |

A=apoyo o ausencia explícita pertinente; B=refutación de premisa/atribución; C=información parcial pertinente; none=ajeno. Son aciertos por relación en material TRAIN ya observado, no validación independiente.

El reranker histórico sin adaptar obtuvo 13/24 en desarrollo (12/12 útiles, 11 ajenos) y 53/64 en conocidos (16/17 útiles, 10 ajenos). Es una referencia del reranker, **no** la ruta de producto FTS5/Decisions. El padre contemporáneo se reprodujo antes de cada entrenamiento; decisiones idénticas y tolerancia de score ≤0,02.

## Puertas completas y daños

- **uniform**: puerta original 24/24+64/64: False; preservación del padre en seis conjuntos: False; integridad: True; candidato local: False.
  - pool: útiles perdidos `hpa_q01_m01, hpb_q15_m01`; ajenos nuevos `ninguno`.
  - known: útiles perdidos `cv3_03_00, cv3_08_02, cv3_14_00, cv3_16_00`; ajenos nuevos `ninguno`.
  - cleft_probe: útiles perdidos `ninguno`; ajenos nuevos `cdp_19, cdp_20`.
- **hard**: puerta original 24/24+64/64: False; preservación del padre en seis conjuntos: False; integridad: True; candidato local: False.
  - pool: útiles perdidos `hpa_q01_m01`; ajenos nuevos `hpb_q16_m04`.
  - known: útiles perdidos `cv3_08_02, cv3_14_00, cv3_16_00`; ajenos nuevos `ninguno`.
  - cleft_probe: útiles perdidos `ninguno`; ajenos nuevos `cdp_19, cdp_20, cdp_24`.

Candidato para siguiente validación según regla congelada: **ninguno**. Si ambos pasaban, se prefería uniforme; una ventaja relativa hard/uniform no sustituye las puertas originales. No se adopta ningún mecanismo en esta fase.

## Cómo se llegó a P3

P1 creó 192 filas TRAIN nuevas (96 útiles/96 none), con tres autores y dos revisores ciegos separados: 192/192 etiquetas y roles concordantes. P2 añadió 64 filas A; dos ambigüedades se corrigieron y volvieron a revisar antes de puntuar, conservando originales. Pool P2: 256 filas, 128 útiles/128 none. Los revisores son subagentes de la misma familia, sin independencia humana o de familia.

P1: 163/192 aciertos, 69/96 útiles y 2 ajenos. P2: 219/256, 101/128 útiles y 10 ajenos. Ambos exigían cuatro grupos A y cuatro C con errores positivos: A=0, B=12, C=12. **Ambos fallaron elegibilidad y no se entrenaron**. El incremento del denominador no es una comparación de rendimiento.

P3 pregunta algo distinto: sustituir cuatro grupos B y cuatro C uniformes por los cuatro primeros según errores positivos, errores totales, pérdida V2 e ID. A mantiene exactamente los mismos cuatro grupos en ambos brazos. Selección adaptativa respecto a P2; protocolo/selección/datos/hiperparámetros/decisión congelados antes de entrenar. Crítica de diseño separada aceptó el contraste con límites; no rescata P1/P2. Los anchors A solo cubren uno de los nueve ajenos iniciales, por lo que la retención se mide explícitamente.

Cada brazo parte del mismo padre V5 matched, con AdamW nuevo; 176 filas de fondo +48 suplementarias, 224 presentaciones, una época y 56 actualizaciones. Mismos roles/targets por posición y presupuesto. LoRA q/v en últimas ocho capas, 327.680 parámetros, base FP16 congelada/adaptador FP32; loss CE+penalización positiva de margen 2; lr1e-4, acumulación4 y clip1. Sin barridos de épocas, semillas, umbrales o checkpoints.

## Verificación y límites

- uniform: 26537/26537 comprobaciones offline; 2.691 eventos, checkpoint congelado antes del final y salida real 0 observada.
- hard: 26534/26534 comprobaciones offline; 2.691 eventos, checkpoint congelado antes del final y salida real 0 observada.

Preflight independiente 2.378/2.378 antes del modelo. Su primera versión comparó por error cinco referencias contra seis: fallo y fuente preservados, checker corregido antes de inferencia, sin cambiar el runner o los datos congelados. La rama no utilizada del materializador también se corrigió antes del modelo conservando fuente y plan originales.

Auditoría reconstruye IDs, targets, orden, tokens, pérdidas escalares, eventos y hashes. No se persistieron gradientes completos, estados AdamW, logits o tensores base finales: no se afirma reconstrucción numérica independiente de esos tensores; la inmutabilidad base durante entrenamiento se apoya en fingerprints registrados.

Las seis series son TRAIN o material ya observado; los no seleccionados del pool tampoco son reserva independiente. La comparación de un paquete B/C, una semilla y una ejecución por brazo no identifica dificultad pura, efectos B/C separados ni superioridad poblacional. Original24 reservado y V6 final permanecen cerrados. Puerta de producto, casos nuevos, recursos completos ≤1,5×, recibos y estabilidad siguen pendientes.

Se verificaron los mismos 213 hashes de implementación Rust/contratos y el binario de evaluación; no cambió su código ni se repitió Cargo. Las pruebas anteriores 879 predeterminadas y 881 HTTP2 (24/75 ignoradas) permanecen vinculadas a las mismas fuentes/binario. No hay cambios en DB, secretos, servicios, ramas o producto.

## Tiempo y coste

| Ejecución | Tiempo medido | Forward | Backward | Actualizaciones |
| --- | ---: | ---: | ---: | ---: |
| uniform | 160.815 s | 1.344 | 224 | 56 |
| hard | 211.307 s | 1.344 | 224 | 56 |
| Minería P1+P2 | 56,077 s | 448 | 0 | 0 |

Total de modelos de esta fase: **428.199 s**, 3.136 forwards, 2.688 scores de evaluación/minería, 448 backwards y 112 actualizaciones. Preparación, revisión Codex y auditoría no están incluidas en ese tiempo; no es latencia del producto ni una comparación de p50/p95 de la ruta completa.

**0 llamadas API nuevas y USD 0 facturado por API en esta fase**. Coste Codex y coste económico local no cuantificados; no se declaran gratuitos. Acumulado remoto histórico sin cambios: 6.155 operaciones, 23 campañas emparejadas completas, USD 4,566379428 conocidos +24 operaciones sin coste conocido.

Artefactos: `outputs/iromem/evaluation/utility-learning-v7-2026-10-22/`, junto con pools/minerías v1/v2 y selección P3; planes, resultados negativos, revisiones, journals, checkpoints, comparador y auditorías preservados.

## Revisión retrospectiva y siguiente hipótesis

La revisión crítica separada rechaza ambos brazos. Reutilicé 1.120 scores persistidos, sin nuevas inferencias, para desglosar transiciones por rol y pertenencia al TRAIN: de seis positivos B fallidos seleccionados en hard se corrigió uno; de cinco C, ninguno. Ambos perdieron el mismo positivo A no presentado en TRAIN. Son descripciones del resultado, no prueba de una causa de gradiente. `transition-diagnostic.json` conserva las 48 agrupaciones y cada distancia score−0,5, que no es un margen logit ni probabilidad calibrada.

No continuar esta receta mediante más minería, épocas o barridos del banco conocido. La siguiente hipótesis es incorporar preservación explícita de decisiones correctas del padre en TRAIN, tanto positivas como negativas, comparando solo ese cambio de objetivo contra control idéntico. Requiere mecanismo concreto, coeficiente único y protocolo congelado antes de entrenar; no usar conocidos/sondas como restricciones ni destilar errores del padre como verdad. Está autorizada la continuación; no hay permisos pendientes. Preservar al padre por sí solo tampoco cumple la puerta, porque el padre aún falla.
