# 3. Semilla íntegra

Estado: implementado. La composición incluye IDs, fuentes, condiciones y, cuando corresponda, el aviso de revisión de equivalencias antes de contar. Si excede tokens o bytes se conserva toda la selección, pero no se publica semilla. No se truncan recuerdos, negaciones o condiciones para hacerla caber.

`cumulative_input_and_final_seed_have_exact_inclusive_limits` compara semilla idéntica en el límite y omisión completa un token por debajo, con selección conservada y marca de inclusión falsa. El caso incluye la nota de equivalencias, cuya contribución se cuenta. `seed_and_deadline_limits_never_publish_truncated_or_late_seed` conserva las cotas previas de bytes/plazo. Las pruebas existentes preservan fuente, roles y texto de contexto.

Arreglo incidental: `included_in_seed` se publica únicamente tras composición aceptada y dentro del plazo; los equivalentes presentados para revisión también se marcan incluidos cuando su texto sí aparece. Antes, podían figurar incluidos al omitir la semilla y desaparecer de la marca tras una composición aceptada. La selección del clasificador permanece separada de esa marca.
