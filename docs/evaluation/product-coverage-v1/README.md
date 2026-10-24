# Cobertura adversarial del producto v1

Fecha: 2026-10-24. Alcance: ampliar controles deterministas de M4 sobre la API pública de retrieval y la CLI/JSONL vigente. [Banco y etiquetas](../../../tests/fixtures/product-coverage-v1/README.md). Fuentes, escritura/reconciliación y ciclo de vida se desarrollan en chats separados; este cambio solo añade pruebas, fixtures y documentación propios. No modifica producción, migraciones, proveedores, modelos o políticas predeterminadas.

## Contratos cubiertos

| Control | Evidencia observable |
| --- | --- |
| Matriz versionada: 20 consultas × 2 modos | Conjuntos exactos de candidatos, ausencia/presencia y cobertura; hashes del corpus y etiquetas |
| Filtros retirados de forma controlada | Entidad/tipo añaden exactamente los controles previstos; actor y ámbito siguen separados |
| Pesos BM25 | Peso cero cambia puntuación, no retira coincidencias de un campo del `MATCH`; no se confunde con ablación del índice |
| Segmentación | Rango UTF-8 válido, continuidad y reconstrucción byte por byte de los 23 originales, más CRLF, comillas y código |
| Composición de cinco casos multilingües/largos | Resumen, negaciones, condiciones y fuente declarada conservados; ninguna relación semántica inventada |
| Contradicción y fallo de segunda revisión | Dos candidatos distintos, selección inicial preservada, condición roja intacta y recibos conocidos agregados |
| Selección forjada y llamadas posteriores | Un ID de otro actor no se publica; ningún marcador privado ajeno llega al adaptador o respuesta; la misma conexión sigue utilizable |
| Recibo incompleto de fallo | Una invocación, sin reintento; tokens conocidos preservados, coste desconocido sigue null |
| Vacío, referente pendiente y contexto excesivo | No llaman al adaptador ni fabrican una semilla; aclaración y límite no se presentan como ausencia de recuerdos |
| Mantenimiento por 64 revisiones nuevas | Una proyección vigente por identidad, revisión obsoleta rechazada al componer y ocho rollbacks de lote sin publicación parcial |
| Reapertura CLI y reimportación rechazada | Tres intentos fallidos de importar revisiones duplicadas preservan datos, revisión 2, duplicados distintos y aislamiento |
| Recuperación de JSONL | Tras JSON mal formado, límite inválido y operación desconocida, las siguientes consultas funcionan con sus propios filtros |
| Concurrencia local | Cuatro procesos/sesiones, 24 lecturas cada uno; dos entidades alternadas, 96 respuestas exactas y consulta posterior correcta |

La prueba de revisiones es una secuencia breve, no operación sostenida. Las cuatro sesiones comparten la ruta de una DB temporal y cada proceso posee su única conexión; no se añaden conexiones paralelas dentro de una app. La barrera libera a la vez cuatro trabajadores, pero no instrumenta solapamiento de transacciones ni fuerza contención SQLite. No hay escritor concurrente, carga remota, puertos, servicio persistente o base personal. Al terminar se cierran procesos/stdin y se retiran solo directorios propios.

## Reproducción

Desde la raíz del repositorio, coordinar primero el uso de Cargo si hay otros chats trabajando:

```sh
cargo test --offline -p gennie-memory-retrieval --test product_coverage
cargo test --offline -p gennie-memory --test product_coverage_cli
cargo fmt --all --check
```

Las pruebas usan SQLite en memoria o un directorio temporal generado por PID/reloj/secuencia. Los subprocesos CLI trabajan desde ese directorio y reciben ruta de DB y clave de proveedor vacía de forma explícita; no usan `.env` del checkout. Ningún control intenta llamadas remotas ni carga pesos. Son controles cortos; no se añadió un benchmark grande a la suite obligatoria.

## Interpretación y pendientes

El pase acredita los contratos comprobados en el código local ejecutado. No acredita comprensión multilingüe, resolución semántica de homónimos/contradicciones, fuente externa auténtica, calidad del clasificador o respuesta final. El adaptador programado recibe el ID a devolver desde la prueba y no puede usarse para medir recuperación semántica.

M4 permanece abierto. Faltan preguntas naturales con revisión humana cuando se retome, etiquetado semántico independiente, comparación emparejada de proveedores y variantes completas, ablaciones reales de embeddings/RRF/umbrales y sin recuperación, mediciones de latencia/recursos bajo concurrencia instrumentada con escritores y mantenimiento sostenido. Las fuentes nuevas, reconciliación y lifecycle necesitan sus pruebas propias y una regresión conjunta cuando sus contratos se estabilicen. E5 continúa como investigación; estos controles no justifican adopción ni ANN.

Resultado de los enfoques: **10 pruebas de retrieval y 3 de CLI aprobadas, cero fallos**. Tiempos de ejecución de tests: 6,46 s y 0,55 s, excluyendo compilación; no son latencias de producto. Compilación CLI con tres avisos `dead_code` existentes. [Resultados y hashes de los archivos de prueba y fixtures](verification.json). Las fuentes productivas se editan simultáneamente en otros chats: no se afirma que estos resultados acrediten cambios posteriores. Cierre conjunto posterior: el chat de escritura ejecutó las suites workspace y aquí se contrastaron los logs existentes, sin repetir Cargo: **1.153 aprobadas por defecto y 1.169 con embeddings**, cero fallos y 25 ignoradas en cada una. Ambas incluyen los diez tests `product_coverage` y los tres `product_coverage_cli`. Los hashes de los logs quedan en `verification.json`; formato y diff fueron informados correctos por el chat coordinador. No hay commit, push o despliegue asociado.
