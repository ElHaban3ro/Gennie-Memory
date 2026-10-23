# Diagnóstico Decisions vs. roles_v1 — 2026-10-23

## Decisión

`roles_v1` no mejora la selección del baseline actual Decisions en las tres muestras examinadas. No se adopta y no se repite sobre estas cohortes. La V7 es exploratoria; las otras dos son diagnósticos retrospectivos. Ninguna es una estimación poblacional ni una confirmación de adopción.

El resultado más útil es que los falsos positivos que Gemma mostró en el banco de atribución **no reaparecieron con Decisions**: Decisions quedó 9/9 en esa misma fixture. La matriz de borrador/versión final también quedó 8/8 con Decisions. En V7, preparada y revisada a ciegas antes de esta campaña, Decisions y `roles_v1` empataron 64/64. Esto apunta a un problema de calidad del proveedor/configuración experimental anterior, no a un defecto reproducido del baseline de producto en estos casos.

## Resultados

| Muestra | Política | Exactos | Evidencia útil | Selecciones ajenas | Degradadas | Coste informado | Clasificador p50/p95 |
|---|---|---:|---:|---:|---:|---:|---:|
| V7 exploratoria (64) | Decisions + baseline | 64/64 | 56/56 | 0 | 0 | USD 0,00380380 | 462 / 828 ms |
| V7 exploratoria (64) | Decisions + roles_v1 | 64/64 | 56/56 | 0 | 0 | USD 0,00537180 | 446 / 1.108 ms |
| Atribución de sujeto, retrospectiva (9) | Decisions + baseline | 9/9 | 7/7 | 0 | 0 | USD 0,00019100 | 628 / 1.594 ms |
| Atribución de sujeto, retrospectiva (9) | Decisions + roles_v1 | 9/9 | 7/7 | 0 | 0 | USD 0,00041150 | 478 / 2.236 ms |
| Versiones/corrección, retrospectiva (8) | Decisions + baseline | 8/8 | 8/8 | 0 | 0 | USD 0,00019020 | 520 / 918 ms |
| Versiones/corrección, retrospectiva (8) | Decisions + roles_v1 | 8/8 | 8/8 | 0 | 0 | USD 0,00038620 | 411 / 1.203 ms |

No hubo casos discordantes entre políticas en ninguna muestra. `roles_v1` aumentó el coste informado 41 % en V7, 115 % en atribución y 103 % en la matriz de versiones; en los tres pares añadió latencia p95 y no ganó calidad. Las tres ejecuciones suman **162 peticiones completas y USD 0,01035450 informados**. No se reintentó ninguna petición.

## Alcance y siguiente paso del objetivo

Las muestras son distintas y no se agregan como una única tasa. V7 contiene ocho patrones semánticos repetidos en ocho ámbitos, por lo que su 64/64 no demuestra generalización. Los bancos de 9 y 8 casos se reutilizan solo para comprobar transferencia del proveedor y comparar una instrucción nueva; son retrospectivos, no casos nuevos. No se atribuye causalidad más allá de que cambiar baseline por `roles_v1` no cambió ninguna selección en estas ejecuciones.

Se cierran como **no candidatas** `slot_evidence_v2` (retirada en el adaptador actual), `slot_evidence_v3` (0 trazas seleccionables en 14 respuestas previas y tres fallos adicionales), y `roles_v1` (sin ganancia en las tres muestras y con mayor coste). El guard local de atribución sigue siendo útil como control estructural cuando dispone de una traza tipada, pero esa extracción no es fiable en la ruta V3 y no se cuenta como mejora de producto.

El objetivo de calidad no se declara cumplido: estas pruebas no encontraron una mejora que promover. Para completar el paso 3 hace falta una candidata de extremo a extremo que aporte al menos una ganancia útil frente a Decisions sin pérdidas ni selecciones ajenas. El trabajo siguiente debe dirigirse a una causa de error reproducible en el baseline actual y no reabrir estas tres variantes ni sus cohortes.

Los casos, diarios, recibos y manifiestos están en [`roles-v7-exploratory-2026-10-23`](assets/roles-v7-exploratory-2026-10-23/), [`roles-sentence-scope-retrospective-2026-10-23`](assets/roles-sentence-scope-retrospective-2026-10-23/) y [`roles-revision-matrix-v1-exploratory-2026-10-23`](assets/roles-revision-matrix-v1-exploratory-2026-10-23/). Los requests enviados contienen consulta, ámbito y política, sin etiquetas ni claves de respuesta.
