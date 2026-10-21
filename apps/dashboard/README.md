# Dashboard local de iROMEM

El módulo `iromem-dashboard` añade una mesa de revisión HTTP local al mismo proceso y base SQLite de `iromem`. Su primera función es revisar preguntas y recuerdos, pero la estructura admite futuras secciones. La UI está en `web/` con React y Vite; el servidor Rust expone `/api/cases`, `/api/view`, `/api/reviews` y `/api/export` en `127.0.0.1` por defecto o en una IP privada elegida expresamente.

Para compilar el cliente desde la raíz del repositorio:

```sh
cd apps/dashboard/web
npm ci
npm run build
```

Con `IROMEM_DATABASE_PATH` configurada, inicia el servidor local con `cargo run --offline -p iromem -- dashboard 8787`. El puerto se pasa como argumento, no se abre ningún puerto durante los demás comandos. Para abrirlo desde un móvil en la misma red local, compila la UI y pasa como tercer argumento la IP privada de esta Mac: `IROMEM_DATABASE_PATH=.local/m5-review.db cargo run --offline -p iromem -- dashboard 8787 tests/fixtures/conversation-m5-v1/development.jsonl "$(ipconfig getifaddr en0)"`. El servidor imprime la URL simple `http://IP:8787/`; ábrela desde cualquier dispositivo de la misma red local. No hay clave ni autenticación: cualquier equipo de esa LAN que alcance el puerto puede consultar y guardar revisiones. Esta sesión HTTP no cifra el tráfico: úsala únicamente en una red de confianza y cierra el servidor al terminar. Para revisar la cola sintética de M5, pasa además `tests/fixtures/conversation-m5-v1/development.jsonl`. Carga antes `tests/fixtures/conversation-m5-v1/memories.json` en esa misma base si es una base nueva. La UI muestra solo ID, segmento, pregunta y candidatos; omite `expected_ids`, `expected_fact` y demás etiquetas del archivo. En casos de la cola, también oculta las referencias de fuente porque contienen nombres de categorías que sesgarían la revisión. Selecciona el conjunto `Conversación · response_style` para ver los cinco casos de la auditoría.

Al abrir una pregunta se muestran primero cinco resultados de búsqueda. Puedes marcar solo los recuerdos que respondan y guardar; «Mostrar más» añade cinco candidatos cada vez. «Lista completa» abre los 30 recuerdos del ámbito cuando se necesita investigar el caso. «Ninguno responde» exige mostrar todos los resultados de la vista rápida antes de guardarse; una lista parcial no demuestra ausencia. Esta revisión rápida sirve para feedback de producto y no sustituye la auditoría ciega independiente de M5.

Cada revisión queda en `dashboard_reviews` con candidatos, revisiones y decisiones. Una revisión posterior crea un nuevo registro; la última compatible se muestra en la UI. `recall` y `session` pueden reutilizar una respuesta confirmada para la misma pregunta, ámbito, entidad, modo de búsqueda y límite si el conjunto de candidatos sigue vigente. En la vista de todos los recuerdos, la selección solo se reutiliza si los elegidos aparecen también en la búsqueda. Se admite hasta ocho recuerdos en la semilla. Una marca «falta un recuerdo» se registra, pero no crea contenido ni se convierte en selección automática. La coincidencia exacta o levemente normalizada de la pregunta no generaliza a paráfrasis; medir eso requerirá más datos y una política calibrada.

`Compartir / exportar` permite elegir revisiones y descargar JSON con pregunta, ámbito, resúmenes de candidatos y decisiones. La descarga queda en el dispositivo desde el que se usa el dashboard; no se sube automáticamente a ningún servicio externo. No existe todavía un servicio compartido, subida, identidad multiusuario ni consentimiento persistente. Antes de añadirlos hay que acordar destino, autenticación, retención, eliminación y qué campos se pueden enviar. El dashboard local tampoco sustituye una auditoría ciega independiente ni modifica las métricas históricas de M5.
