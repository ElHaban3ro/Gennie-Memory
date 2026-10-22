# Auditoría de etiquetas del banco M5 v3

Las 24 positivas preguntan por un hecho explícito guardado en un único recuerdo del ámbito. Las 12 negativas preguntan por una aprobación, adjudicación, cifra final o resultado confirmado que el texto cercano dice que no existe todavía. `source.json` conserva ambos lados para inspección antes de ejecutar el sistema. Los positivos `paint_rejected`, `recording_ban` y `remote_control_ban` comprueban que una decisión negativa sí puede ser respuesta válida.

Limitaciones: la redacción es sintética y relativamente literal, todos los recuerdos tienen revisión 1, estado activo y ninguna condición; una sola persona-agente preparó y revisó este banco. No se debe considerar revisión adversarial independiente ni validar con él la calidad de usuarios reales. Si una etiqueta fuese ambigua, se excluye o corrige antes de llamar al proveedor y se regeneran los hashes; después de observar resultados no se cambia para mejorar la puntuación.
