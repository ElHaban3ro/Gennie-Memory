# Revisión 35 — smoke sintético fallido

Caso fresco, revisión ciega directa y FTS top-1 1/1. La única invocación clasificador reportó Gemma 3 4B, 626/183 tokens y USD 0,00004960; latencia del clasificador 2.796,005 ms y total 2.798,132 ms. Falló cerrado con `slot_evidence_v2_kind_not_literal`; no seleccionó recuerdo. No se reintentó. El recibo conserva hashes de prompt, esquema y payload, sin guardar contenido crudo del proveedor.

Este resultado muestra que la cita separada del tipo no fue fiable en este caso; no conocemos el texto propuesto porque el raw no se conserva. La revisión posterior reemplaza tipo/sujeto por una cita de identidad conjunta y deriva localmente el tipo a partir del prefijo verificado; Rust sigue comprobando por separado el sujeto y el ID. Los cambios no relajan aceptación. r35 no demuestra una mejora semántica.
