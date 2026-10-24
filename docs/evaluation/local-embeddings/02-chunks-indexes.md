# 2. Fragmentación y publicación

Ejecutar controles sin pesos descargados:

```sh
cargo test --offline -p gennie-memory-retrieval --features local-embeddings local_embeddings
```

El particionador usa el tokenizer real de cada perfil y verifica el límite incluyendo prefijos y especiales. Los tests puros cubren espacios, saltos de línea, acentos combinados, japonés y emoji; concatenar los rangos reproduce el original byte a byte. El máximo medido fue 507 tokens en E5 y 125 en MiniLM; ningún fragmento supera 512/128. Las consultas largas se rechazan, sin truncar.

`complete_publication_filters_deduplicates_and_replays_without_mutation` prueba publicación/repetición y filtros de actor/tipo. `incomplete_tampered_or_orphaned_batches_never_publish` muta fragmentos ausentes, dimensiones, NaN, norma cero, hashes, rangos, ordinales, fuentes e identidades. `publication_sql_failure_rolls_back_profile_index_and_all_chunks` induce fallo SQL y comprueba cero filas persistidas. `revision_change_blocks_prepared_publication_and_serving_old_vectors` prueba cambio entre captura y publicación, rechazo del índice obsoleto y reconstrucción explícita.

Cada publicación re-tokeniza los textos para comprobar que rangos y cantidades del lote coinciden con el procedimiento del perfil. La inferencia queda fuera de la transacción; la revisión y el texto se comprueban otra vez dentro. El manifiesto identifica el conjunto íntegro del ámbito, no solo los resultados de una consulta.

Los [vectores E5 congelados](../../../tests/fixtures/local-embeddings-v1/e5-frozen-index.json) y su [manifiesto](../../../tests/fixtures/local-embeddings-v1/manifest.json) están conservados. Se publicaron desde archivo en una DB temporal y sirvieron consultas reales; importar MiniLM en la misma sesión E5 produjo `embedding_space_mismatch`. El vector de revisión antigua nunca se sirvió después del cambio.
