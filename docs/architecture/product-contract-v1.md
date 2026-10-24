# Contrato de producto v1

Implementado el 2026-10-23. Entrada explícita `recall-v1` y operación `recall_v1` de `session`. `recall` y los bancos históricos conservan su formato anterior; el dashboard fue retirado por decisión del usuario. Esta frontera no declara aprobada la calidad semántica de una política.

## Uso local

```sh
cargo run --offline -p gennie-memory -- segment-text /ruta/texto.txt
cargo run --offline -p gennie-memory -- provider-capabilities
cargo run --offline -p gennie-memory -- recall-v1 tests/fixtures/small-recall-v1.json
cargo run --offline -p gennie-memory-eval -- validate-product-suite /ruta/suite.json
```

`recall-v1` usa la base configurada y puede llamar al proveedor cuando encuentra candidatos y hay credencial. `segment-text`, el catálogo y el validador son locales. En session: `{"op":"recall_v1","request":{...}}`, `{"op":"segment_text","request":{"text":"Texto original."}}` y `{"op":"provider_capabilities"}`.

## Consulta

Ejemplo mínimo (también disponible en `tests/fixtures/small-recall-v1.json`):

```json
{
  "contract_version": 1,
  "search": {
    "query": "¿Qué formato de informes prefiere Marina los lunes?",
    "scope_kind": "project",
    "scope_id": "demo_alpha",
    "as_of": "2026-10-15T12:00:00.000Z"
  }
}
```

Obligatorios: versión, consulta original no vacía, ámbito y fecha UTC RFC 3339 con milisegundos. Tipos de ámbito: `user`, `agent`, `project`, `conversation`; ID ASCII alfanumérico con `_-.:`, máximo 128 bytes. La fecha y el ámbito los aporta quien llama; no se inventa una identidad ni se amplía la búsqueda al faltar contexto. Son ámbitos locales, no autenticación.

| Campo opcional | Valor efectivo y límite |
| --- | --- |
| `search.entity_id` | Filtro de identidad conocida; ausencia conserva búsqueda por ámbito |
| `search.limit` | 20; rango heredado 1–20 |
| `search.query_mode` | `keywords`; modos admitidos por SearchRequest |
| `provider` | `decisions`; también `jev`, `jev114` |
| `selection_policy` | `baseline`; también `per_memory_v1`, `per_memory_v2` |
| `request_id` | Generado si falta; mismo formato ASCII de ID; correlación, sin idempotencia |
| `context` | Lista de `{id,source,text}`, hasta 16, IDs únicos, fuente no vacía ≤256 bytes |
| `unresolved_references` | Hasta 16 rangos `{start_byte,end_byte,required_field}` sobre la consulta UTF-8; campo `entity`, `time` o `condition` |
| `budgets` | `input_bytes:32768`, `seed_bytes:16384`, `deadline_ms:25000` |

Consulta ≤8000 caracteres. Entrada textual suma consulta y textos/IDs/fuentes del contexto; presupuesto entre 1 y 32768 bytes. Semilla entre 1 y 65536 bytes, plazo entre 1 y 25000 ms. Sobre JSON de entrada hay un límite adicional de 128 KiB; también sobre el payload completo al proveedor. No son presupuestos de tokens ni de dinero. Campos desconocidos del contrato nuevo se rechazan.

Quien llama debe aportar los turnos necesarios para identificar «ese proyecto», la entidad conocida, fecha y condiciones relevantes. El contexto se mantiene separado en una representación JSON para el clasificador; no modifica la consulta FTS ni crea autoridad. Si declara una referencia sin resolver, recibe `clarification_required` con rango y dato requerido, sin búsqueda ni inferencia. Sin esa declaración, `ambiguity:not_assessed`: no hay detector semántico de ambigüedad ni resolución automática de pronombres.

## Segmentación literal

`segment-text <archivo UTF-8>` no necesita base, configuración ni proveedor. `recall-v1` aplica el mismo módulo puro a consulta y contexto. `session` ofrece `segment_text` con `request:{text:...}`.

Versión `literal_segments_v1`: rangos `[start_byte,end_byte)` contiguos, texto literal y SHA-256 del original. Concatenar los segmentos reconstruye exactamente todos los bytes, incluidos espacios y saltos. Fronteras conservadoras de oración/párrafo con protección de comillas, paréntesis, código, abreviaturas, URLs y decimales; no se divide por comas, conjunciones ni longitud fija. No es un analizador lingüístico completo.

Límite independiente: 256 KiB y 512 segmentos. Umbral blando 512 bytes: una frase larga se conserva con advertencia. Un exceso absoluto falla sin resultado parcial. `semantic_independence:not_assessed` evita presentar segmentos como hechos independientes. Se mantienen negaciones, condiciones y orden por conservación literal; no se resuelve su alcance semántico.

No se resumen textos, reducen tokens, reindexan recuerdos ni hacen llamadas por segmento. La búsqueda conserva el original. Seleccionar fragmentos para ahorrar tokens requerirá una evaluación posterior de cobertura.

## Resultado y trazas

`execution_status` distingue `completed`, `degraded`, `failed`. `outcome` distingue `evidence_selected`, `no_candidates`, `abstained`, `clarification_required`, `not_evaluated`. Un fallo técnico nunca se publica como abstención fiable.

Se devuelven consulta efectiva validada, candidatos, evaluaciones, semilla opcional, cobertura, incidencias, consumo y etapas `validation`, `segmentation`, `search`, `classification`, `equivalence_review` y `composition`, con duración y motivo. En `baseline`, si se selecciona un recuerdo y hay otros candidatos, `equivalence_review` hace una segunda petición acotada para proponer como máximo un posible duplicado. La propuesta incluye IDs/revisiones y se incorpora a la semilla como pareja separada con `decision: agent_review_required`; no combina, edita ni borra recuerdos. La segunda petición suma tokens/coste conocidos a `usage`; si falla, la respuesta queda degradada, conserva la recuperación principal y marca cobertura `unavailable`. Puede elevar el coste de una recuperación con candidatos. La salida terminal no deja etapas en ejecución. El plazo monotónico empieza al recibir la solicitud; limita ambas peticiones remotas y se comprueba entre etapas y tras composición. No cancela en mitad una consulta SQLite. HTTP tiene además máximo 20 segundos. Una semilla que exceda bytes o plazo no se publica truncada.

Cada evaluación conserva ID/revisión, selección, inclusión real en semilla, relación, origen y validación estructural. Las políticas por recuerdo aportan `supports_answer`, `refutes_premise`, `partial_context`, `unrelated`. Son juicios del proveedor; pertenencia y revisión comprobadas no demuestran verdad semántica. Baseline conserva `unassessed` y `reason:null`: elegir un recuerdo no proporciona una justificación textual. No se equiparan probabilidades ni se exponen como confianza factual.

`coverage` separa entrada, búsqueda acotada, candidatos presentados, clasificación, revisión de equivalencia y semilla. Un prefijo evaluado no cuenta como todos los candidatos. Cobertura de pregunta permanece `unknown`; índice semántico `not_applicable` en esta ruta FTS. Evaluaciones y recuerdos de semilla exponen `provenance_status`: `declared_unverified` cuando el llamante aportó una fuente y `not_provided` en otro caso. Ningún conector verifica todavía fuentes externas. `possible_duplicate` es una hipótesis del proveedor, no una puntuación de similitud ni una acción de fusión.

Tiempos intercambiados UTC con milisegundos y duraciones monotónicas. `input_sha256` identifica el JSON recibido (en session, su serialización tipada); puede faltar en entrada excedida. `implementation_sha256` identifica exclusivamente la concatenación de product.rs, diagnostics.rs, segmentation.rs y service.rs, no el binario completo ni el adaptador. Para campañas registrar además revisión Git/binario, configuración y corpus. La respuesta incluye texto original: no es un log público anonimizado.

`local_review:not_available_dashboard_removed`: el dashboard fue retirado. Ninguna ruta activa consulta ni reutiliza sus votos. En bases antiguas se conservan las tablas y se verifica su checksum histórico; no se crean en bases nuevas. No queda una integración con el dashboard pendiente.

## Proveedores e incidencias

`provider-capabilities` (también `provider_capabilities` en session) publica la matriz derivada del adaptador: modelo solicitado, políticas y límites. Decisions, Jev 1.13 y Jev 1.14 usan decisiones tipadas. Baseline presenta hasta 20 y elige como máximo uno; per_memory_v1 hasta 16 y per_memory_v2 hasta 8, ambas experimentales. Gemma estructurado y las demás políticas experimentales siguen disponibles solo por rutas heredadas. No hay nuevos embeddings ni equivalencia entre formatos.

La disponibilidad, límites de contexto y precios actuales figuran **sin verificar**. Recibos/modelo servido son opcionales. La matriz declara ausencia de reintentos/fallback implementados por Gennie Memory, no las decisiones de enrutamiento internas de OpenRouter.

Incidencias exponen `code`, etapa, componente, mensaje español, `cause_code`, HTTP real, posibilidad de reintento, espera conocida y acción sugerida. Se distinguen credencial ausente, autenticación/acceso, créditos/pago, presupuesto, gasto en curso, tasa, timeout, conexión, ventana de contexto, contenido, indisponibilidad y respuesta inválida. Sin evidencia suficiente, causa sin clasificar; no se inventa «cuota agotada» a partir de cualquier fallo. Los mensajes describen el fallo de selección si la búsqueda local ya recuperó candidatos. No copian el cuerpo remoto ni credenciales.

Se reconoce error estructurado incluso dentro de HTTP 200. `Retry-After` acepta segundos enteros acotados; fecha HTTP u otro formato queda desconocido. Metadatos tipados se interpretan mediante una lista cerrada, basada en la [documentación oficial de errores de OpenRouter](https://openrouter.ai/docs/api_reference/errors-and-debugging), consultada el 2026-10-23. Esto no acredita disponibilidad real de los modelos.

Tokens/coste ausentes son `null`, nunca cero inventado. Recibo `complete`, `partial`, `unknown` o `not_called`; el coste informado en fallo no demuestra coste total. Búsqueda vacía y aclaración no llaman al modelo. El cliente debe leer el estado de producto dentro del sobre de session; `complete` del sobre solo confirma que se produjo respuesta. Fallos de apertura/configuración/lectura de archivo siguen siendo errores del adaptador CLI en stderr.

## Verificación y evaluación

Pruebas locales de reconstrucción UTF-8, límites, ámbitos, aclaración sin llamadas, pertenencia/revisión, roles, degradación, recibos, plazo compartido y error HTTP 200. Suite offline completa con mocks propios; no llamadas pagadas. El [protocolo prospectivo](../evaluation/product-contract-v1-protocol.md) separa desarrollo/calibración/final. La campaña posterior de 18 casos superó las tres puertas con Decisions/per_memory_v1; [resultado y límites](../evaluation/product-contract-v1-results-2026-10-23.md). No se extrapola al baseline, que mostró una abstención con contexto en el smoke.

[Registro de verificación local](../evaluation/product-contract-v1-verification.md).

[Preparación del banco y revisión sintética](../evaluation/product-contract-v1-bank-verification.md): 18 casos nuevos, corpus ligado y dos revisiones separadas; con [campaña posterior completada](../evaluation/product-contract-v1-results-2026-10-23.md) para el perfil congelado.
