# Evidencia del piloto v5 de desarrollo

Este directorio conserva tres muestras sintéticas de 60 consultas y doce informes por caso de recuperación, clasificación y memoria semilla. [El manifiesto](manifest.json) fija los hashes SHA-256 y los nombres de archivo; [el informe narrativo](../../../../docs/evaluation/pilot-2026-10-15.md) explica método, cifras y límites. No contiene credenciales, datos reales, pesos de Laya ni la base SQLite local.

| Muestra | Consultas | Informes |
| --- | --- | --- |
| [A](sample-a.jsonl) | Cinco por segmento; muestra usada para explorar instrucciones | `a-top1`, cuatro variantes Laya, Jev y OpenAI Decisions en dos casos cada uno |
| [B](sample-b.jsonl) | Cinco por segmento, sin IDs de A; once entidades compartidas | `b-top1`, `b-laya-choice`, `b-laya-pairwise-conditions` |
| [C](sample-c.jsonl) | Cinco por segmento, sin IDs ni entidades de A/B | `c-top1`, `c-laya-pairwise-conditions` |

Cada `*.json` de informe incluye resultados por caso, conteos, perfil y latencias. Los documentos de fuente de v5 son referencias sintéticas declaradas. La carga del modelo y la indexación se miden aparte del tiempo caliente por consulta.

Desde la raíz del repositorio, comprueba la integridad de los archivos congelados con Rust:

```sh
cargo run --offline -p iromem-eval -- verify-hashes outputs/iromem/evaluation/v5-development
```

El comando vigente coteja los hashes declarados. El verificador histórico también cotejó casos con benchmark v5, etiquetas, conteos, emparejamiento y separación de entidades de C al congelar el paquete; ese script fue retirado. Las cifras son de pilotos de desarrollo; no se ejecutó aquí la partición final ni la campaña grande.
