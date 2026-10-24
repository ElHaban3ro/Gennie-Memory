# 3. Comparación y escalado

Mac arm64, 24 GiB de RAM; CPU f32, `RAYON_NUM_THREADS=4`, `TOKENIZERS_PARALLELISM=false`. Ambos perfiles usan el mismo corpus, casos y runtime Rust. 25 positivas y cuatro ajenas, tres rondas por método: 87 mediciones de latencia por fila. Aciertos contados una vez por caso, no multiplicados por rondas. La carga del modelo queda fuera de los tiempos calientes.

Resultados finales V2 tras corregir la cota RRF:

| Método | Top-1 / 25 | Top-5 / 25 | Top-20 / 25 | p95 caliente |
| --- | ---: | ---: | ---: | ---: |
| FTS5 (corrida E5) | 13 | 18 | 18 | 2,91 ms |
| E5 coseno exacto | 24 | 25 | 25 | 29,13 ms |
| MiniLM coseno exacto | 24 | 25 | 25 | 27,21 ms |
| E5 + FTS5 / RRF | 19 | 24 | 25 | 31,29 ms |
| MiniLM + FTS5 / RRF | 20 | 24 | 25 | 29,44 ms |

Fuentes: [E5 V2](e5-v2-results.json), [MiniLM V2](minilm-v2-results.json). Los primeros intentos [E5 V1](e5-results.json) y [MiniLM V1](minilm-results.json) se conservan como antecedente del defecto RRF. La repetición conserva textos y etiquetas: no es otra reserva. El orden de IDs/revisiones se mantuvo en las [348 comparaciones entre rondas](stability.json).

FTS5 deja vacías tres positivas; E5 y MiniLM cubren las tres. Las siete omisiones textuales en top-20 se recuperan con ambos encoders en top-5. Los cuatro negativos producen candidatos semánticos: no se midió abstención o verdad de una respuesta final. El banco es pequeño, sintético y etiquetado por el mismo autor; no demuestra calidad poblacional.

La fusión RRF pierde `q-rust` en top-5 con ambos perfiles, aunque conserva top-20 y el coseno solo lo recupera primero. No se ajustó RRF con este caso ni se promovió esa política. Se eligió el perfil E5 según la puerta previa de empate y contexto máximo; la ruta FTS5 predeterminada sigue igual.

## Escalado del coseno exacto

Se reutilizaron dos vectores reales del perfil por recuerdo para crear copias controladas con IDs distintos. La ruta real vuelve a verificar inventario, filtros y revisiones, lee BLOB f32, valida norma, calcula cosenos y ordena. No se usa una matriz desvinculada de SQLite. Tres rondas por tamaño; el p95 por rango más próximo coincide con el máximo de esas tres muestras.

| Recuerdos / fragmentos | p95 barrido exacto | p95 total con consulta |
| --- | ---: | ---: |
| 100 / 200 | 0,76 ms | 11,68 ms |
| 1.000 / 2.000 | 7,47 ms | 19,07 ms |
| 10.000 / 20.000 | 84,16 ms | 107,55 ms |

[Escalado final V2](scale-v2-results.json); [primera corrida](scale-results.json): 1,02/11,19/124,90 ms de barrido, preservada para no ocultar variación entre corridas. Ambas cumplen el objetivo previo de ≤250 ms para este tamaño. Los textos son repetidos y las fuentes caben en caché: no es concurrencia sostenida, diversidad semántica o garantía para cualquier longitud. El índice admite hasta 100.000 fragmentos, pero solo se midieron 20.000. No hay base medida para introducir ANN en este alcance.

## Recursos y reproducción

Un proceso nuevo cargó E5, importó los vectores congelados y realizó tres consultas: [recursos medidos](resources.json), pico RSS **1.295 MiB**, carga **670,6 ms** y pared total del proceso **891,3 ms**. Esto incluye arranque y tres consultas, no un turno Codex ni clasificación remota. La comparación V2 registra por separado sus cargas, preparación e importación; son muestras distintas.

```sh
cargo build --offline --release -p gennie-memory -p gennie-memory-eval --features gennie-memory/local-embeddings
RAYON_NUM_THREADS=4 TOKENIZERS_PARALLELISM=false target/release/gennie-memory-eval benchmark-local-embeddings target/release/gennie-memory config/embedding-profiles/e5-small.json tests/fixtures/local-embeddings-v1/suite.json .local/e5-new-run
RAYON_NUM_THREADS=4 TOKENIZERS_PARALLELISM=false target/release/gennie-memory-eval benchmark-local-embeddings target/release/gennie-memory config/embedding-profiles/minilm.json tests/fixtures/local-embeddings-v1/suite.json .local/minilm-new-run
RAYON_NUM_THREADS=4 TOKENIZERS_PARALLELISM=false target/release/gennie-memory-eval benchmark-local-cosine target/release/gennie-memory config/embedding-profiles/e5-small.json .local/scale-new-run
```

Cada directorio debe ser nuevo. El evaluador conserva corpus, diario de consultas, perfil, recibo e índice calculado. No modifica bases personales ni llama al proveedor remoto. Las métricas anteriores de LoCoMo y sus SVG siguen ligadas a su binario y cohorte históricos; esta medición no los reemplaza.
