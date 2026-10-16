# 0011 — Recorrido textual completo en Rust

Fecha: 2026-10-15. Estado: implementado como CLI local; calidad remota en el nuevo recorrido todavía por medir.

## Contexto y alternativas

La ruta rápida era la política base, pero las campañas encadenaban búsqueda Rust, clasificación OpenRouter y composición Rust mediante Python. El usuario pidió integrar ese recorrido y retirar Laya si dejaba de necesitarse. La ruta híbrida sigue disponible de forma explícita y necesita vectores, por lo que se conserva su codificador experimental; el nuevo recorrido predeterminado no lo carga.

## Decisión

`recall` recibe una consulta textual y un proveedor opcional: Decisions por defecto o Jev explícito. El servicio de recuperación obtiene candidatos autorizados, llama a un contrato interno `Classifier` y compone la semilla tras revalidar ID y revisión. El adaptador OpenRouter vive en bootstrap; usa una pregunta tipada `choice` con opciones A–T y `none`, el prompt `direct` de las campañas generales, HTTPS con plazo de veinte segundos y una respuesta limitada a 1 MiB. No hay cascada ni umbral añadido. La clave se lee del entorno o `.env` y nunca se registra. Un fallo de transporte, HTTP, formato o selección devuelve candidatos y semilla ausente con estado degradado; no hay reintento automático de una llamada de resultado incierto. Un corpus sin candidatos omite la llamada remota.

Se eligió `reqwest 0.12.28` con `blocking`, `json` y `rustls-tls` sin features predeterminadas, fijado en el workspace y disponible en la caché local. [La versión y sus features](https://docs.rs/crate/reqwest/0.12.28/features) se comprobaron antes de incorporarla; Cargo resolvió `rustls 0.23.45`, versión que [RustSec identifica como corregida](https://github.com/RustSec/advisory-db/blob/main/crates/rustls/RUSTSEC-2026-0285.md) para el aviso revisado. Se contrastó la forma del endpoint y las preguntas tipadas con [OpenRouter Decisions](https://openrouter.ai/openai/gpt-6-luna-decisions-20261006) y el sondeo existente del proyecto. Esta revisión puntual no sustituye una auditoría de todo `Cargo.lock`.

## Límites y revisión

Una comprobación funcional con una base temporal y una memoria sintética terminó con Decisions seleccionando el recuerdo y componiendo semilla (167 tokens de entrada informados; 763 ms de recorrido). Jev respondió al mismo contrato y eligió `none` (373 tokens de entrada; 542 ms). Son una llamada por proveedor en corridas separadas, no una comparación de calidad o velocidad. En el entorno restringido, la primera llamada devolvió `transport_error`; con acceso de red funcionó. Sin clave, el comando devolvió `missing_api_key`, candidatos y ninguna semilla, como indica el contrato.

La CLI Rust no lee el perfil JSON de campañas, no genera vectores, no incluye el comando en `session` y todavía no impone presupuesto de tokens de entrada o semilla. Los resultados anteriores de velocidad y calidad no miden este nuevo recorrido. Revisar tras pruebas por segmento y medición de latencia con proceso caliente y frío; no activar automáticamente en Codex hasta contar con evidencia propia.
