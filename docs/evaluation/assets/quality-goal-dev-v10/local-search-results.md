# V10 preflight local de recuperación

DB SQLite SHA-256: `690289772670a06c148f2e1dcd23ad555cf9f1069edaf256407b1b1cd0c1c275`. Binario iromem SHA-256: `e99793096b1c2bc5260b062607a14082b2d7a17c0109c8933967d5a8bc89d7a2`.

`search-batch` ejecutó 16 consultas en la DB V10 antes de llamadas remotas. Recuperó los 12/12 IDs adjudicados útiles en el roster candidato. Las cuatro consultas sin evidencia útil sí devolvieron listas no vacías (3, 3, 4 y 4 candidatos): por tanto la abstención depende de selección, no de que el buscador descarte esas consultas. No se ajustó límite ni se cambió consulta después de medir.

Diario reproducible: [`local-search-results.jsonl`](../../../../../outputs/iromem/evaluation/quality-goal-dev-v10-2026-10-23/local-search-results.jsonl). La clave se usó solo para auditar cobertura local y no se envió al clasificador.
