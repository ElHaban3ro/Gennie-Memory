# Banco de regresión del producto v1

Fecha: 2026-10-24. Datos íntegramente sintéticos; etiquetas del mismo autor, sin revisión independiente o humana. Es desarrollo conocido al ejecutarlo: no es una reserva nueva, una campaña del proveedor ni un benchmark de calidad.

`manifest.json` fija por SHA-256 los bytes de `corpus.json` y `cases.json`. La prueba `apps/retrieval/tests/product_coverage.rs` verifica esos hashes antes de ejecutar la matriz. El corpus tiene 23 filas de revisión y 22 identidades actuales. Cada una de las 20 consultas declara el conjunto exacto de IDs permitido; se ejecuta en `natural` y `keywords`, con los mismos filtros y reloj fijo. La comparación no exige un orden BM25 concreto.

| Segmento | Casos |
| --- | --- |
| Español/diacríticos, inglés, portugués, árabe, japonés y Unicode combinado | 6 |
| Evidencia al final de un original de más de 12.000 caracteres, peso del resumen cero | 1 |
| Homónimos, con identidad explícita o sin resolver | 3 |
| Contradicciones condicionadas conservadas como registros distintos | 1 |
| Texto igual en dos identidades distintas | 1 |
| Intersección actor/ámbito/entidad/tipo con marcadores privados sintéticos | 1 |
| Revisión vigente y obsoleta | 2 |
| Frontera de vigencia: inicio inclusivo/final exclusivo | 1 |
| Sin coincidencia léxica, traducción no automática, actor desconocido y ámbito vacío | 4 |

Los casos multilingües comprueban coincidencias literales y conservación de bytes, no comprensión o traducción. `translation-is-not-automatic` registra una ausencia léxica esperada; no cuenta como una respuesta semántica correcta. Las contradicciones conservan ambos candidatos: el banco no exige que el motor léxico resuelva cuál condición aplica. Los IDs prohibidos en la prueba privada se evalúan con todos los filtros activos; al retirarlos explícitamente se comprueban controles positivos de accesibilidad por el operador local. Esto no acredita autenticación ni ACL de host.

Las pruebas de producto usan respuestas programadas con un adaptador que cuenta el JSON sintético antes de devolverlas. No representan payloads nativos del proveedor, inferencia real, facturación o aciertos del clasificador. Las cifras de consumo/coste de ese adaptador son señales sintéticas para comprobar conservación de recibos y ausencias; no gasto incurrido.

[Alcance, ejecución y pendientes](../../../docs/evaluation/product-coverage-v1/README.md).
