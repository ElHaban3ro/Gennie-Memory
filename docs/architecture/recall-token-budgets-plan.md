# Presupuestos de tokens para recall-v1

Fecha: 2026-10-24. Alcance: primer subpaso de completar recall sobre FTS5; E5 permanece como investigación.

1. Fijar un contrato de conteo reproducible: tokenizer local `o200k_base`, tiktoken-rs 0.12.0, JSON UTF-8 exacto emitido por el adaptador y texto final de semilla. No se afirma equivalencia con tokenizer, plantilla o facturación del proveedor. Mantener bytes/plazo como cotas independientes.
2. Limitar tokens acumulados de las peticiones reales de clasificación y revisión de equivalencias, antes de cada envío. Valores por defecto: 16.384 tokens de JSON de entrada por operación, 2.048 tokens de semilla; ambos configurables entre 1 y 65.536. No recortar candidatos ni alterar prompts/políticas para hacerlos caber.
3. Registrar cada conteo con hash de la petición, etapa, aceptación o rechazo; preservar consumo comunicado por el proveedor separado. Un adaptador sin soporte falla cerrado. Si la segunda petición no cabe, conservar la selección principal y marcar la revisión no realizada; no declarar fallo remoto o coste nuevo.
4. Contar la semilla final con IDs, fuentes, condiciones y nota de equivalencias ya incorporados. Si no cabe, conservar selección/traza y omitir la semilla completa, con diagnóstico accionable. No se publican fragmentos truncados.
5. Probar fronteras, Unicode, esquemas/candidatos/contexto incluidos, segundo pase y acumulación, ausencia de envío al rechazar, sesión CLI y regresiones offline. Sin API pagada ni base personal. Documentar mediciones y limitaciones antes de cerrar este subpaso; verificación de fuentes y relevancia/calibración siguen pendientes.

La superficie es el contrato de producto `recall-v1` y session `recall_v1`; recall experimental heredado conserva su contrato para reproducir campañas. La dependencia se verificó en crates.io (`0.12.0`, MIT, Rust >=1.85, no retirada) y docs.rs; el directorio de RustSec para tiktoken-rs devolvió 404, comprobación acotada que no acredita ausencia de vulnerabilidades ni una auditoría de transitivas. No se cambia el grafo de crates ni se añade recurso de red o descarga runtime.
