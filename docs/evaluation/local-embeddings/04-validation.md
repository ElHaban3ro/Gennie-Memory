# 4. Regresiones y fallos

Comprobaciones finales:

```sh
cargo test --offline --workspace
cargo test --offline --workspace --features gennie-memory/local-embeddings
cargo fmt --all --check
git diff --check
```

[Registro de verificación, hashes de fuentes/binarios e informes](verification.json).

**Resultado final:** workspace con feature: **1.103 aprobadas**, cero fallos y 25 ignoradas; sin feature: **1.091 aprobadas**, cero fallos y 25 ignoradas. Son 12 controles unitarios nuevos del encoder/índice, además de 13 controles CLI con el modelo real. Formato y diff sin errores.

Las pruebas de esta ruta usan SQLite en memoria; la CLI real usa bases temporales. Los modelos no se cargan en las unitarias: tokenización y estados del índice se prueban sin red; los dos modelos reales se probaron por separado mediante el evaluador Rust y las comprobaciones CLI registradas en [runtime-validation.json](runtime-validation.json).

Los 13 controles CLI reales incluyen publicación de vectores congelados, dato útil al final del original largo, perfil cruzado, consulta excesiva, máximo de trabajo, plazo cooperativo, actor ajeno, JSON mal formado, recuperación de sesión tras errores, revisión nueva, rechazo del índice viejo, ausencia de artefactos sin fallback y hash de tokenizer alterado. Los procesos terminaron y sus DB temporales se eliminaron.

Las regresiones específicas incluyen también `persisted_profile_or_vector_corruption_is_rejected`, `artifact_hash_mismatch_is_detected_before_model_loading`, `missing_chunks_wrong_profile_and_work_budget_fail_without_partial_results` y `rrf_never_adds_semantic_votes_outside_its_top_twenty`. La migración 9 se aplica incluso sin feature para conservar un esquema determinista; los checksums históricos continúan verificados por kernel/registro.

Iteración registrada: la primera comparación V1 detectó después una contribución RRF fuera del top-20 semántico. Se corrigió, se añadió una prueba no vacua con el candidato textual en rango semántico 25 y se repitió el mismo banco como V2. V1 se conserva en los informes iniciales; la repetición es regresión sobre casos conocidos, no una reserva independiente. No cambió el encoder, los textos, etiquetas, pesos o prefijos.

No se modificó la base personal, no hubo API pagada ni publicación Git. El plazo no interrumpe kernels a mitad de inferencia. Los modelos usan CPU f32; no se verificó Metal, otra plataforma, concurrencia sostenida o recuperación tras pérdida de disco. El aviso transitivo de mantenimiento de `paste` y los límites de la revisión de dependencias permanecen visibles en la decisión 0051.
