# 0025 — Revisión documental automática única de evaluación

Fecha UTC: 2026-10-21. Estado: experimento, sin adopción.

Los borradores omiten identidad y calificadores aunque sus menciones contengan las palabras. Se conserva el contrato completo y se ensaya una única revisión obligatoria de todos los ocho borradores, tanto aceptados como rechazados estructuralmente. La guía es general, con texto/ontología/esquema propios, borrador y error local recalculado; no incorpora críticas semánticas humanas/Codex ni gold.

La revisión final sustituye obligatoriamente al borrador: no best-of, union ni fallback. Una revisión fallida degrada. Solo padres terminales HTTP200 con recibo conocido; no se reintenta HTTP incierto. Se contabilizan dos etapas para el mecanismo y solo la nueva para gasto incremental.

Propietario: módulo puro `documentary_extraction_contract/grounded_revision.rs`, compartido por preparador/reporter y runner ignorado de bootstrap. No cambia traductor, parser, política runtime, dependencias, estado o configuración. Frente a un ancla opaca, mantiene expresividad y evita introducir otra normalización de identidad no demostrada. No se presume que el segundo juicio sea más fiel.

Revisar ante ocho salidas completas con dos críticos nuevos, sin degradación semántica, y solo entonces ejecutar el banco completo y comprobar recursos/estabilidad. El pase de formato no es suficiente. [Protocolo](../evaluation/grounded-revision-v1-protocol-2026-10-21.md).
