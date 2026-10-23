# Selección de parcial epistémico en una llamada V1 — resultados

## Decisión

En V10 la candidata logró **16/16**, igual que Decisions `baseline`; no perdió decisiones baseline ni eligió recuerdos ajenos. Pasó el gate local de calidad y los límites de coste y p95 congelados. Sin embargo, **no demostró una mejora de calidad frente al baseline**, que también quedó perfecto en esta cohorte. No se promueve ni se adopta; baseline permanece vigente mientras el paso 3 sigue abierto.

## Calidad por relación

| Relación | Casos | Baseline | Single-pass V1 |
| --- | ---: | ---: | ---: |
| Apoya respuesta | 4 | 4/4 | 4/4 |
| Refuta premisa | 4 | 4/4 | 4/4 |
| Parcial/provisional | 4 | 4/4 | 4/4 |
| Ajeno | 4 | 4/4 vacíos | 4/4 vacíos |
| **Exactitud** | **16** | **16/16** | **16/16** |

La candidata no añadió un acierto respecto al control. La revisión previa fue ciega para el modelo clasificador y la hizo un subagente Codex de la misma familia; la cohorte es pequeña, sintética y de desarrollo, no una estimación poblacional ni revisión humana. El caso V10 no reproduce una omisión de baseline, por lo que no confirma que la candidata resuelva las dos omisiones de V9.

## Coste y latencia

| Métrica | Baseline | Single-pass V1 | Razón candidata/control |
| --- | ---: | ---: | ---: |
| Operaciones / peticiones HTTP | 16 / 16 | 16 / 16 | 1,00× |
| Coste comunicado | USD 0,000403300 | USD 0,000585700 | 1,452× (**+45,2 %**) |
| p50 de servicio | 416,5 ms | 414,3 ms | 0,995× |
| p95 de servicio | 707,3 ms | 557,1 ms | 0,788× |
| p50 de tokens de entrada | 259,5 | 373,5 | 1,439× |
| Recibos completos / degradaciones | 16/16 / 0 | 16/16 / 0 | pasa |

Coste API combinado: **USD 0,000989** por 32 peticiones, sin reintentos; bajo el techo de USD 0,003. Los gates fijados (coste y p95 ≤1,5×) pasan en esta muestra, pero el coste subió y no hay ganancia de calidad. La p95 con solo 16 casos es inestable y no prueba una reducción general.

## Procedencia y verificación

V10 se congeló en `docs/evaluation/assets/quality-goal-dev-v10/`; sus hashes y la revisión ciega están junto a los fixtures. El preflight de búsqueda encontró los 12/12 IDs útiles y mantuvo listas no vacías para los cuatro negativos. Cada brazo ejecutó 16 casos con Decisions, el mismo roster, DB, candidatos y orden, sin etiquetas en los mensajes. DB SHA-256 `690289772670a06c148f2e1dcd23ad555cf9f1069edaf256407b1b1cd0c1c275`; casos SHA-256 `56831267ccff77d63aa8b25dfd4db4950bf9f7553354cbc7e802ab4abfd32be9`; binario SHA-256 `e99793096b1c2bc5260b062607a14082b2d7a17c0109c8933967d5a8bc89d7a2`.

`cargo fmt --all --check` y `cargo test --offline -p iromem-retrieval -p iromem-eval -p iromem --quiet` pasan: 1.029 pruebas aprobadas, 25 ignoradas, cero fallidas. Diarios y reportes por brazo: [`outputs/iromem/evaluation/quality-goal-dev-v10-2026-10-23/`](../../outputs/iromem/evaluation/quality-goal-dev-v10-2026-10-23/).

## Estado del objetivo

La hipótesis en una llamada es eficiente en peticiones, pero el prompt añade 45,2 % de coste sin mejora medida en V10. Por eso no se activa como predeterminado. El paso 3 sigue abierto: aún no hay un mecanismo con mejora diferencial confirmada. El paso 4 tiene una comparación nueva y ciega de desarrollo, no una confirmación independiente. El paso 5 no se cierra mientras no haya una mejora demostrada y una decisión de adopción/no adopción respaldada por esa confirmación.
