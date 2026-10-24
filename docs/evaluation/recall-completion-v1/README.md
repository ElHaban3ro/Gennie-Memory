# Verificación de mecanismos de recall v1

Fecha: 2026-10-24. [Contrato](../../architecture/recall-completion-v1.md), [decisión0056](../../decisions/0056-opt-in-recall-factors-and-source-import.md).

Resultado: los mecanismos y su evaluación se completaron. El calibrador experimental de dos intervalos **empeoró Brier en los tres factores para ambos proveedores en la reserva final**: se conserva para reproducir ese resultado negativo, sin adopción, umbral nuevo o cambio de baseline.

Implementación: integración optativa de factores existentes en producto, puntuaciones individuales, calibrador de frecuencias por intervalos con huella exacta y conector explícito de adquisición HTTPS. No cambia baseline, búsqueda FTS5 predeterminada o E5 de investigación.

Pruebas deterministas del producto cubren separación de contexto/utilidad, bloqueo de un candidato relevante de otro contexto, reproducción de semilla al aplicar calibración informativa y rechazo de huella incompatible antes de llamar al modelo. Parser conserva scores/recibo y rechaza valores malformados/incompletos. Conector valida origen/protocolo/credenciales/query/puerto y bytes/hash/UTF8/tamaño sin red en unitarias. Los resultados ejecutados se detallan abajo y en el informe M4; los tests no acreditan semántica remota, TLS real o verdad externa.

El banco/campaña prospectivos `tests/fixtures/m4-factor-v1` y su informe M4 son la fuente de calibración/fiabilidad medida por factor. No se extrapolan los dos intervalos estimados con ocho observaciones por factor a confianza general ni se presentan como mejora discriminativa.

Verificación focal ejecutada en el árbol actual durante implementación:

- `cargo test --offline -p gennie-memory-retrieval product::tests --lib`:20 aprobadas,0 fallos.
- `cargo test --offline -p gennie-memory factors --bin gennie-memory`:4 aprobadas,0 fallos, incluye captura del payload completo y parser.
- `cargo test --offline -p gennie-memory source_import --bin gennie-memory`:2 aprobadas,0 fallos, sin red.
- `cargo test --offline -p gennie-memory --test product_coverage_m4_concurrency reserved_lock -- --nocapture`:3 aprobadas,0 fallos tras reproducción/arreglo dirigido de contención. [Diagnóstico](sqlite-contention-diagnosis.md).
- `cargo build --offline -p gennie-memory`:compilación correcta; se conservan dos avisos previos de funciones sin uso en slot_evidence.

Huella congelada del adaptador de factores para la campaña coordinada: `2a4e1a95035032f66b4b272c1ac91c1b5537cd84aa1c922eb8bc861ca34cff77`. La campaña remota está documentada abajo. La suite conjunta se registra en el informe M4; esta lista identifica solo las pruebas focales de implementación.

## Adquisición HTTPS ejecutada

El [registro real de adquisición](source-import-live.json) documenta tres controles sobre una DB temporal propia: SHA256 incorrecto rechazado con cero snapshots; importación CLI del fixture sintético público `small-corpus.json` en el commit 70332d73 por HTTPS con bytes coincidentes con el hash de `git show`; replay por JSONL conservando el mismo snapshot y exactamente una fila. Incluye URL, fecha UTC del recibo, hashes de binario/código y contenido. Tres controles aprobados; sin proveedor remoto, puertos o base personal; DB temporal eliminada. Esto verifica adquisición TLS real y vinculación de bytes en ese origen/instante, no autoría o verdad factual del fixture.

La primera llamada sintética real devolvió `openai/gpt-6-luna-decisions-20261006` al solicitar el alias `openai/gpt-6-luna-decisions`. La revisión del contrato separa requested_model/served_model y mantiene matching exacto del modelo observado al aplicar el calibrador. Pruebas de producto verifican aplicación con snapshot observado, rechazo del alias cuando se exige snapshot y rechazo de proveedor ajeno antes de red.

Tras separar alias y snapshot se ejecutó `cargo test --offline -p gennie-memory-retrieval product:: --lib`:23 aprobadas/0 fallos (21 de producto y 2 del calibrador),0.74 s. El archivo de factores y su huella permanecieron idénticos.

## Evaluación remota prospectiva completada

La [campaña v2](../m4-completion-v1/campaign-v2/protocol.json) congeló corpus, etiquetas, binarios, huella del adaptador y revisión humana antes de inferencia; fuentes distintas por desarrollo/calibración/final, ocho casos por partición. Las familias de plantillas se comparten entre particiones: no es una reserva de todas las construcciones lingüísticas. La muestra humana confirmó ocho etiquetas antes de ejecutar; los datos siguen siendo sintéticos.

Se completaron48 operaciones Decisions/Jev en 61,75 s, sin degradaciones. Coste comunicado v2:USD 0,002298616; sumando el único smoke de desarrollo de la corrida v1 abortada por un error del orquestador:49 HTTP,USD 0,002361316. Se conserva la [corrida abortada y su diagnóstico](../m4-completion-v1/campaign-v1/setup-failure-diagnosis.json), sin presentarla como campaña completa ni ocultar su coste. No hubo reintentos automáticos. [Terminal v2](../m4-completion-v1/campaign-v2/terminal.json).

Los calibradores se ajustaron únicamente con ocho observaciones de calibración por proveedor/factor, usando dos intervalos fijados previamente (corte 0,5) y suavizado Laplace. Se congelaron antes de final y se aportaron inline a la solicitud de producto. El auditor Rust comprobó que las puntuaciones publicadas en las respuestas finales coinciden exactamente con el mapeo de sus artefactos; esa igualdad acredita implementación, no fiabilidad semántica.

| Proveedor/modelo servido | Selección final acorde a etiqueta | Degradaciones finales |
| --- | --- | --- |
| Decisions `openai/gpt-6-luna-decisions-20261006` |8/8|0|
| Jev `typesafe/jev-1.13-20260917` |7/8|0|

Jev omitió el contexto parcial de `final-03`; no se interpreta como error de calibración porque el mapeo es informativo y no cambia elecciones. Decisions conserva las ocho decisiones esperadas en esta pequeña muestra; esto no demuestra superioridad general ni autoriza una promoción del perfil.

| Proveedor | Factor | Brier crudo | Brier tras calibrador |
| --- | --- | ---: | ---: |
| Decisions | mismo contexto |0,016338|0,109375|
| Decisions | dato solicitado |0,085788|0,119592|
| Decisions | corrección de premisa |0,063900|0,078704|
| Jev | mismo contexto |0,113588|0,138889|
| Jev | dato solicitado |0,061450|0,119592|
| Jev | corrección de premisa |0,003263|0,024691|

Brier menor es mejor. Las seis comparaciones empeoran: **la prueba reservada rechaza adoptar estos artefactos como mejora de fiabilidad**. ECE muestra cambios distintos según factor y proveedor, descritos en los JSON; no se selecciona retrospectivamente la métrica favorable para declarar éxito. No se reajustan bins tras mirar final ni se reutiliza esa reserva para obtener otro resultado favorable. El mecanismo continúa optativo y experimental; `calibration_status:empirical_profile_bound` significa que se aplicó el artefacto ligado, no que haya superado su evaluación.

Informes con datos por caso/factor, Brier/ECE y hashes: [Decisions final](../m4-completion-v1/campaign-v2/decisions-final-report.json), [Jev final](../m4-completion-v1/campaign-v2/jev-final-report.json). Artefactos congelados: [Decisions](../m4-completion-v1/campaign-v2/decisions-calibrator.json), [Jev](../m4-completion-v1/campaign-v2/jev-calibrator.json). No aplicar a otro snapshot/alias suponiendo identidad; producto falla cerrado ante mismatch.

Se cierra implementación de adquisición explícita, factores separados y mecanismo de puntuación empírica con evaluación negativa documentada. No se declara resuelta la calidad semántica universal, una probabilidad factual calibrada, un nuevo umbral de producción ni la integración automática Codex. E5/híbrido mantienen su condición de investigación.

## Regresión conjunta final

Las dos suites offline del código compartido terminaron con código 0: **1.173 aprobadas** por defecto y **1.189 con `local-embeddings`**, cero fallos y 26 ignoradas en cada una. `cargo fmt --all --check` y `git diff --check` pasan. [Registro, hashes y logs completos](../m4-completion-v1/joint-verification.json). La única modificación posterior a las suites fue formato del adaptador de comandos de evaluación, sin cambio de comportamiento; no se altera el perfil congelado ni se repite inferencia.
