# 0014 — Dashboard local de revisión

Estado histórico: implementado el 2026-10-17 y retirado por decisión del usuario el 2026-10-23; [decisión vigente de retirada](0047-remove-dashboard.md). El contenido siguiente documenta el diseño anterior, no funciones disponibles hoy.

## Contexto

M5 encontró etiquetas ambiguas y fallos de elección en `conversation`. La revisión manual en un documento no escala ni produce feedback reutilizable para cada instalación. El usuario pidió una app `dashboard` más del workspace Rust, servida por HTTP local, con React y Vite, y que este trabajo figure en los pasos M5.

## Alternativas

- Mantener un formulario o archivo de auditoría separado: evita servidor y dependencias, pero no ofrece una función de producto ni retroalimentación local.
- Hacer todo en React con almacenamiento del navegador: simplifica el backend, pero separa las correcciones de la SQLite y del recorrido `recall`.
- Crear `apps/dashboard` con tablas propias y React/Vite: añade un módulo y superficie HTTP, pero permite revisar, persistir y reutilizar con la conexión SQLite existente. Esta es la opción adoptada.

## Decisión y límites

`dashboard` depende de `retrieval`, tiene su migración propia y recibe la conexión desde bootstrap. El servidor escucha en loopback por defecto. Si el operador pasa una IP privada, escucha solo en esa interfaz y permite acceso directo desde esa LAN. Valida `Host` y `Origin`, pero no autentica a los equipos de la red: cualquiera que alcance el puerto puede leer y guardar revisiones en la base del proceso. Esta simplificación fue solicitada expresamente para abrir el dashboard desde Safari tras el navegador integrado de ChatGPT, que perdía la clave al traspasar la URL. HTTP LAN no cifra el tráfico, por lo que este modo se limita a redes de confianza. Cada revisión es un evento nuevo y conserva una instantánea de candidatos, sin modificar recuerdos. `recall` puede aplicar la última revisión compatible para una consulta equivalente si las revisiones y candidatos actuales siguen iguales; una elección fuera del top de búsqueda no se aplica. No se generaliza a preguntas parecidas. El resultado indica `classification_status: review_reused` y `served_model: local_review`, sin coste remoto informado.

El cliente sirve una cola de casos sin etiquetas esperadas, permite múltiples coincidencias y estados de ausencia separados. La exportación es una descarga seleccionada por el usuario; no hay subida a una base compartida hasta definir destino, identidad y política de datos. Esta limitación evita simular un consentimiento sin servicio real.

Se verificaron versiones exactas y capacidades en los registros oficiales: [Axum 0.8.9](https://docs.rs/axum/0.8.9/axum/), [Tokio 1.53.1](https://docs.rs/tokio/1.53.1/tokio/), [React 19.3.0](https://www.npmjs.com/package/react/v/19.3.0) y [Vite 8.3.4](https://www.npmjs.com/package/vite/v/8.3.4). Se consultó [RustSec](https://rustsec.org/advisories/) para avisos conocidos; esa consulta puntual no es una auditoría completa de dependencias. Las versiones reales quedan fijadas en `Cargo.lock` y `web/package-lock.json`.

## Condición de revisión

Antes de exponer HTTP públicamente o a usuarios diferentes, diseñar sesiones, autorización individual, cifrado y separación de datos por usuario. Antes de afirmar mejora general, medir repeticiones y nuevas preguntas en sujetos separados, tasas de falsos recuerdos y brechas de cobertura. Antes de habilitar subida, revisar esquema, minimización, consentimiento por lote, retención y borrado.
