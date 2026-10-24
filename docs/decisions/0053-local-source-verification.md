# 0053 — Procedencia literal contra snapshots locales

Fecha: 2026-10-24. Estado: implementada; verificación y límites en el informe enlazado.

Contexto: source_kind/ref/locator se conservaban como declaración. La existencia de texto en memory_source_evidence o su flag histórico no acreditaba resolver una fuente. El diseño pide registro local autorizado, versión/rango, comparación y recibo; aún no hay conector que autentique origen externo.

Decisión: snapshots inmutables y versionados dentro de retrieval, separados por actor/ámbito/tipo/ref. El operador aporta texto UTF-8 explícito (hasta 256 KiB), sin acceso automático a rutas, URLs o servicios. Una versión no cambia de contenido; la última nueva registrada es vigente y replay de una anterior no la reactiva. Fuente de confianza: operator_registered_snapshot. La migración aditiva 10 añade source_snapshots y source_verification_checks; no reescribe migraciones aplicadas, abre conexiones adicionales o añade dependencias.

El verificador compara original_text literal con utf8:start:end (fin exclusivo); sin locator requiere ocurrencia única, también ante solapamiento. Fuente ausente en el registro, cita ambigua y locator opaco permanecen no resueltos. Cita ausente/distinta, rango inválido o integridad alterada son rechazo. Perfil local_snapshot_literal_v1 con hashes, versión/rango, instante y motivo; recibos idempotentes confirman commit antes de retornar y se cotejan de nuevo contra evidencia actual, no se toman como permisos permanentes.

Source-register/source-verify y JSONL adaptan el núcleo. check_evidence acepta Connection/Transaction prestada sin abrir transacción/persistir; permite reutilizar exactamente la comparación en propuestas de escritura, sin inventar memoria aún inexistente. Verify/check_candidates gestionan su transacción y recibo. No se considera verificado el resumen, condiciones añadidas o verdad factual; semantic_support queda not_assessed.

Recall-v1 comprueba candidatos y revalida los que finalmente compone después del clasificador. Policy report es compatible con fuentes no resueltas; require_verified exige procedencia literal de toda semilla, incluidos equivalentes. El rechazo de una fuente incluida bloquea en ambos modos. Conserva selección, candidatos y recibos de proveedor, sin fallback/repetición/abstención inventada. El aviso de procedencia se incorpora antes de medir bytes/tokens. Las políticas limitan publicación, no acceso del clasificador a candidatos. Recall heredado conserva protocolo de reproducción.

Costes: texto adicional persistente por versión y recibos por estado observado; CPU de hashes/comparación y transacción SQLite por conjunto, con máximo veinte candidatos. El texto de procedencia incrementa la semilla y puede hacer que se omita por su presupuesto. No hay GC, conectores externos, autenticación de host, reloj de vigencia de fuente o garantía sobre cambios posteriores a la instantánea. E5 continúa como investigación.

Alternativas: confiar en una etiqueta histórica o en el resumen no comprueba origen; leer directamente cada ref como URL/ruta introduce permisos/red y semántica de versiones no definidos; certificar verdad con un modelo excede la comparación literal. Una integración externa futura deberá acreditar su propio origen y versiones.

[Plan](../architecture/recall-source-verification-plan.md), [contrato](../architecture/product-contract-v1.md), [pruebas por paso](../evaluation/recall-source-verification/README.md). Aplicabilidad/relevancia calibradas y calidad semántica siguen abiertas.
