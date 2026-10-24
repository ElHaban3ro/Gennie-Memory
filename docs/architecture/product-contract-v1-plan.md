# Plan de consolidación del contrato de producto v1

Fecha: 2026-10-23 (America/Bogota). Estado: diseño de referencia con implementación inicial completada. El [contrato vigente](product-contract-v1.md) documenta los campos y límites efectivos; desarrollo, calibración y prueba final del perfil Decisions/per_memory_v1 ejecutados y auditados.

## Avance de evaluación

El [banco prospectivo](../../tests/fixtures/product-contract-v1/README.md) contiene 18 casos y 36 recuerdos nuevos, particiones separadas, dos revisiones sintéticas sin etiquetas visibles y conciliación conservada. El evaluador liga corpus/política y audita revisiones; el preflight de desarrollo fue local sin proveedor. Las tres etapas se ejecutaron posteriormente sin cambiar la receta: 18/18 resultados esperados, recibos completos y puertas superadas. La aprobación se limita al banco sintético y a ese perfil; no promueve la política predeterminada. [Cierre y límites](../evaluation/product-contract-v1-results-2026-10-23.md). [Verificación](../evaluation/product-contract-v1-bank-verification.md).

## Requisitos confirmados y alcance

El usuario autorizó consolidar consulta, resultado, cobertura, degradación, capacidades de proveedores, trazas y evaluación separada. Pidió salidas que expliquen la causa concreta y permitan actuar al agente, y registrar qué aporta cada recuerdo. Confirmó expresamente que la reducción de textos debe ser automatizable y programática, sin modelo, para evitar tokens y latencia remota.

Este plan conserva las decisiones de diseño originales. Sus propuestas no sustituyen el contrato vigente enlazado arriba. La [decisión 0045](../decisions/0045-programmatic-text-segmentation.md) registra la restricción de segmentación local. No se requiere otra aprobación general para implementar lo ya autorizado.

Alcance: operaciones locales de consulta y recuperación, adaptadores existentes y su evaluación. No incorpora escritura/reconciliación completa de recuerdos, hook ni MCP. El contrato de lectura debe poder evolucionar hacia ellos sin afirmar que ya existen.

## Punto de partida anterior a la implementación

- `apps/retrieval/src/model.rs`: `SearchRequest` contiene `query`, `scope_kind`, `scope_id`, `entity_id`, `as_of`, `limit` y `query_mode`. `RecallRequest` añade proveedor y política. Hay respuestas de búsqueda, semilla, tiempos, consumo y errores.
- `apps/retrieval/src/classifier.rs`: `DecisionError` conserva código, estado HTTP y consumo parcial. No contiene todavía el diagnóstico detallado propuesto aquí.
- `bootstrap/src/openrouter.rs`: el transporte conserva estados HTTP y recibos, pero agrupa fallos bajo `transport_error` o `http_error`. En el camino observado descarta el detalle tipado del error de transporte. No basta con cambiar el mensaje para distinguir timeout de otra avería de red.
- Existen trazas de políticas experimentales y cobertura semántica del índice. No equivalen a cobertura completa de la pregunta ni habilitan atribuir roles a una elección baseline que no los informa.
- El [contrato de evaluación actual](../evaluation/contract-v1.md) y los bancos históricos se conservan. No se renombrarán resultados conocidos como una reserva nueva.

Esta comprobación es de código/documentación, no de ejecución ni disponibilidad actual de proveedores.

## 1. Consulta, contexto, textos largos y ambigüedad

### Entrada propuesta

Conservar los campos existentes al introducir la versión nueva. Diseñar la transición con CLI, `session` y evaluador; el dashboard propuesto originalmente fue retirado por petición del usuario. No romper silenciosamente diarios o fixtures históricos. Los ejemplos de este documento describen el destino, no JSON aceptado hoy.

| Campo | Regla propuesta | Responsabilidad |
| --- | --- | --- |
| `contract_version` | Obligatorio en el contrato nuevo; valor `1`. Las entradas heredadas siguen por el adaptador de compatibilidad explícito. | Cliente/adaptador |
| `query` | Obligatorio, texto original no vacío. Nunca sustituido por palabras clave o segmentos. | Quien llama |
| `scope_kind`, `scope_id` | Obligatorios. Mantener los ámbitos admitidos; ausencia no significa búsqueda global. | Quien llama; el futuro host autenticado debe comprobar permisos |
| `as_of` | Obligatorio en el servicio, UTC RFC 3339 con milisegundos. El adaptador puede capturar el instante actual una sola vez y devolver el valor efectivo. | Cliente/adaptador |
| `entity_id` | Opcional, solo para identidad conocida. No resolver homónimos por proximidad ni inventar IDs. | Quien llama |
| `context` | Opcional y acotado: turnos relevantes y referencias explícitas, con identificador/origen. Se conserva separado de la consulta. | Quien llama |
| `limit`, `query_mode`, `provider`, `selection_policy` | Opcionales; documentar valores efectivos, capacidades y combinaciones admitidas. Conservar defaults actuales hasta una decisión de adopción. | Configuración/cliente |
| `request_id` | Opcional en entrada; generado si falta y devuelto para correlación. No promete idempotencia. | Cliente/adaptador |
| `budgets` | Opcional con defaults locales documentados: entrada, segmentos, candidatos, salida y plazo total. Ningún cero/valor omitido significa ilimitado implícitamente. | Configuración/cliente |

El contexto mínimo es pregunta + ámbito + fecha efectiva. Añadir entidad cuando se conoce; para «ese proyecto», aportar el fragmento previo que permite identificarlo y la referencia explícita si existe. No pedir el historial completo, etiquetas esperadas, puntuaciones ni justificaciones al cliente. El contexto textual no otorga permisos y se trata como datos, no como instrucciones del sistema.

Las referencias propuestas por el cliente deben distinguirse de identidades comprobadas. Al variar contexto, entidad, fecha o versión, revisar la compatibilidad de una revisión local; no reutilizarla solo por coincidencia de pregunta.

### Segmentación local, determinista y conservadora

Propietario previsto: módulo puro dentro de `apps/retrieval`, reutilizable desde un comando CLI breve y desde recuperación. Sin crate, servicio, base de datos o dependencia nuevos salvo necesidad demostrada.

- Entrada: texto original UTF-8, versión del segmentador y límites explícitos.
- Salida: segmentos ordenados con `segment_id`, `start_byte`, `end_byte`, texto literal y tipo de frontera; hash del original, versión, advertencias y estado de cobertura estructural.
- Los rangos forman una partición íntegra, contigua y sin solapamientos. Espacios y saltos se asignan a segmentos; concatenarlos reconstruye exactamente el original. No normalizar Unicode ni eliminar caracteres antes de calcular offsets.
- Preferir fronteras de párrafo y oración. Proteger decimales, abreviaturas, URLs, código, comillas y paréntesis con reglas conservadoras y versionadas. La puntuación sola no demuestra independencia semántica.
- No dividir por comas, conjunciones o número fijo de caracteres para fabricar frases independientes. Una oración muy larga se conserva con advertencia; si excede el límite absoluto, retornar un fallo explícito sin una supuesta salida completa.
- No resolver pronombres, elipsis o sujetos implícitos; conservar orden, enlace al original y contexto. Una condición en una oración anterior puede aplicar a la siguiente.
- No alterar los originales de recuerdos ni reindexarlos como hechos independientes. Los segmentos son vistas del texto, no nuevos recuerdos.
- Separar **segmentar** de **reducir tokens**: este mecanismo hace unidades más manejables, pero la suma de texto no es menor. El ahorro exige seleccionar después qué unidades enviar, con cobertura y exclusiones visibles; no se promete de antemano.

Integración progresiva: primero segmentador independiente y metadatos de consulta; después comparar recuperación con y sin segmentación en desarrollo. No hacer automáticamente una llamada remota por frase ni fusionar listas sin un límite global. Si se buscan varias partes, conservar ámbitos, deduplicar por ID/revisión y registrar límites por parte y globales.

### Ambigüedad

Distinguir pregunta ambigua, evidencia contradictoria, respuesta inválida del proveedor y abstención válida. El contrato puede representarlas aunque el producto todavía no detecte todas de forma automática.

`clarification` describe código, fragmento de consulta, dato requerido, opciones sustentadas y pregunta breve. Ejemplo: «¿Te refieres al proyecto Norte de diseño o al proyecto Norte de infraestructura?». No generar opciones que no estén en el contexto autorizado ni asumir que se conocen solo por compartir nombre.

Si el referente afecta al ámbito, no ampliar la búsqueda. Si falta modalidad o fecha que decide aplicabilidad, no elegirla. Se permite devolver evidencia parcial claramente marcada cuando sea compatible con lo conocido; si no puede establecerse esa compatibilidad, no componer una semilla fiable. `ambiguity: unknown` expresa falta de evaluación, no ausencia de ambigüedad.

## 2. Resultado y contribución de cada recuerdo

Respuesta propuesta: versión, `request_id`, valores efectivos, estado técnico, resultado semántico, etapas, candidatos, evidencia seleccionada, semilla, cobertura, aclaraciones, incidencias y trazas. La semilla conserva referencias, revisión, condición, estado y fuente declarada.

Separar dimensiones para evitar un enum con combinaciones contradictorias:

- Ejecución: `completed`, `degraded`, `failed`.
- Resultado: `evidence_selected`, `no_candidates`, `abstained`, `clarification_required`, `not_evaluated`.
- Etapas: `pending`, `running`, `completed`, `skipped`, `failed`, con causa cuando corresponda. La respuesta terminal no deja etapas `running`.

Cada evidencia puede aportar una o varias relaciones por parte de la pregunta: `supports_answer`, `refutes_premise`, `partial_context`, `unrelated`, `undetermined`. `unassessed` significa que el mecanismo no evaluó esa relación. No equiparar `undetermined` con irrelevancia, ni una refutación con falta de evidencia.

Registrar origen del juicio (`provider`, `local_review`, `local_rule`), versión y alcance de la comprobación. Las citas literales demuestran ubicación del texto, no verdad ni correferencia. Una política baseline que entrega solo un ID conserva relación `unassessed`; enriquecerla exige un mecanismo capaz y una evaluación propia, no una etiqueta inventada.

`reason` es opcional y conserva procedencia. Ausencia se representa con `null`. Los diagnósticos locales del sistema van en `issues`; no se atribuyen al proveedor.

### Incidencias accionables

Cada incidencia tendrá `code`, `stage`, `component`, `message`, `http_status`, `retryable`, `retry_after_ms`, `suggested_action` y causa/origen comprobado. Campos desconocidos usan `null`; mensajes se generan con plantillas y metadatos seguros, no copiando sin filtro el cuerpo remoto.

Taxonomía a implementar: credencial ausente, autenticación rechazada, créditos/pago requerido, límite de peticiones, timeout, transporte, proveedor indisponible, rechazo de la solicitud, contexto excedido, respuesta truncada, JSON/esquema inválido, ID/revisión inválidos y límites locales. Mantener también un error no clasificado cuando falte evidencia.

Ejemplo ilustrativo, condicionado a un rechazo de pago comprobado:

```json
{
  "execution_status": "degraded",
  "outcome": "not_evaluated",
  "issue": {
    "code": "provider_payment_required",
    "stage": "classification",
    "component": "openrouter",
    "http_status": 402,
    "message": "Se encontraron recuerdos candidatos, pero OpenRouter rechazó la selección por un requisito de pago o créditos.",
    "retryable": false,
    "retry_after_ms": null,
    "suggested_action": "check_billing"
  }
}
```

Solo decir «créditos agotados» si el dato específico está confirmado. Un 429 no demuestra saldo agotado; un fallo de conexión no demuestra timeout. Si la búsqueda local terminó, no decir «no se pudieron recuperar proyectos». En fallos multietapa, preservar causa raíz y etapa, no reemplazarlas por un código genérico del contenedor.

`retryable` describe conveniencia técnica, no autorización de reintento automático. Una llamada incierta puede haberse cobrado. El contrato no incorpora reintentos ni fallback ocultos.

## 3. Cobertura, presupuestos y degradación

| Dimensión | Qué expresa | Qué no demuestra |
| --- | --- | --- |
| Entrada | Texto íntegro representado por segmentos; límites alcanzados | Comprensión semántica exhaustiva |
| Índice | Elegibles e indexados bajo ámbito, revisión y espacio efectivo | Recuperación de toda la evidencia útil |
| Búsqueda | Método, candidatos, límites y partes consultadas/omitidas | Que no existan más recuerdos pertinentes |
| Selección | Candidatos evaluados, omitidos y mecanismo | Veracidad del juicio o respuesta completa |
| Pregunta | Partes evaluadas, apoyadas, refutadas, parciales o desconocidas | Completitud si las partes no fueron identificadas fiablemente |
| Semilla | Evidencias incluidas/excluidas y presupuesto aplicado | Verificación externa de fuentes |

Usar estados explícitos `complete`, `partial`, `unknown`, `not_applicable` con denominadores cuando existan. No derivar completitud semántica del número de segmentos. FTS5 con semántico desactivado no es por sí mismo una avería.

Definir cotas exactas durante la primera fase, con unidad y rango: bytes/caracteres de entrada, número de segmentos, candidatos, memorias, bytes o tokens de semilla y tiempo total. Si no existe tokenizer del modelo, medir bytes y declarar cualquier estimación de tokens; no fingir presupuesto exacto de tokens.

Al exceder un límite no truncar silenciosamente ni eliminar condiciones. En fallos de selección mantener candidatos y metadatos; no publicarlos como semilla aprobada. El plazo total de operación debe incluir todas las etapas, además del timeout HTTP existente.

## 4. Capacidades por proveedor

Matriz versionada separando transporte, modelo solicitado, modelo servido y política. Verificar documentación oficial, modelo/disponibilidad y capacidades efectivas antes de llamadas. La lista de modelos del código no prueba disponibilidad actual.

Columnas: tarea tipada/salida estructurada, elección única/múltiple, roles, justificación, puntuación y semántica, abstención/rechazo, esquema, límites de entrada/salida, timeout, recibos y costes, errores reconocibles, fecha y evidencia de verificación. Distinguir `supported`, `unsupported`, `unverified`.

Rechazar una capacidad solicitada que el adaptador no ofrece antes de pagar. No simular roles o razones ausentes. No normalizar puntuaciones de proveedores como probabilidades comparables ni sumarlas con BM25/coseno. Calibración por modelo, versión, tarea y población; RRF combina rangos, no esas puntuaciones.

## 5. Trazas y conservación

Una traza por operación y eventos por etapa, enlazados por IDs y secuencia. Registrar inicio/fin UTC RFC 3339 con milisegundos y duraciones monotónicas; versión de contrato/código/política, configuración efectiva, hashes, IDs/revisiones, cantidades, omisiones, errores y acciones sugeridas.

Consumo por llamada y agregado conocido: tokens opcionales, coste opcional, moneda, estado de recibo (`complete`, `partial`, `unknown`, `not_called`). Una operación con alguna llamada sin recibo no tiene coste total conocido, aunque pueda conservar un subtotal. No duplicar recibos compartidos por varias decisiones.

La respuesta estructurada siempre permite al consumidor guardar el resultado. La persistencia de trazas se diseña por separado: reutilizar los mecanismos existentes, destinos explícitos y permisos restrictivos, sin escribir texto completo ni credenciales por defecto. Guardar texto completo solo en diagnóstico sintético explícito. Los hashes no anonimizan el contenido por sí solos.

Un evento de inicio sin terminal representa ejecución interrumpida/desconocida, no fallo semántico ni éxito. No anunciar escrituras de recuerdos antes de confirmar transacción. Este hito no promete auditoría duradera de operaciones de escritura que aún no existen.

## 6. Evaluación y secuencia de entrega

| Fase | Entrega verificable | Puerta para continuar |
| --- | --- | --- |
| A. Contrato y compatibilidad | Tipos/campos, estados e invariantes, límites exactos, ejemplos válidos e inválidos, mapa de consumidores | Ninguna ausencia convertida en certeza; lectura histórica preservada; decisiones nuevas identificadas |
| B. Segmentación | Función Rust pura y comando local; rangos/versión/advertencias | Reconstrucción exacta UTF-8; fronteras protegidas; límites explícitos; cero HTTP |
| C. Diagnóstico y proveedores | Errores tipados desde transporte hasta respuesta; matriz comprobada | Timeout/créditos/rate limit no confundidos; secretos excluidos; causa raíz y recibos conservados |
| D. Integración | Entrada/contexto, resultado, contribuciones, cobertura y trazas en CLI/session y consumidores | Casos éxito/vacío/abstención/ambigüedad/parcial/fallo; incompatibilidad de revisión local detectada |
| E. Banco y protocolo | Desarrollo, calibración y reserva nueva; etiquetas/particiones/versiones congeladas | Sin fuga de familias/entidades/conversaciones ni gold al runtime; umbrales fijados antes de medir |
| F. Validación | Pruebas de contrato y evaluación del recorrido local completo | Informe de calidad/recursos por segmento; sin presentar síntesis o esquema válido como calidad semántica |

Desarrollo sirve para cambiar código e instrucciones. Calibración fija umbrales y presupuestos de una receta estable. Final evalúa una versión congelada; abrirlo consume su condición de reserva y no permite ajustes retrospectivos para declarar aprobación. Separar por fuente/conversación, entidad y familias de plantillas, no solo IDs de filas.

Primero crear un banco local pequeño: textos largos, negaciones/condiciones, abreviaturas, URLs/código, referencias ambiguas, varias preguntas, homónimos, datos parciales, refutación, corpus vacío, revisiones cambiadas, índice parcial, límites y fallos remotos simulados. Las pruebas estructurales no necesitan llamadas pagadas ni acreditan calidad del clasificador.

Métricas: fidelidad/reconstrucción, evidencia útil recuperada/seleccionada por relación, evidencia ajena, omisiones por parte, abstenciones, aclaraciones, degradación por causa, consumo conocido/desconocido y p50/p95 del recorrido completo. Comparar políticas sobre los mismos casos, conservar cada regresión y desglosar segmentos; no compensar recuerdos ajenos con un promedio mejor.

Puertas estructurales propuestas: cero pérdida de bytes en segmentación aceptada, cero ampliaciones de ámbito, cero referencias inválidas publicadas, ningún fallo técnico contado como abstención válida y ningún coste desconocido contado como cero. Las puertas semánticas y de recursos, el tamaño de muestra y el número máximo de llamadas se fijarán en el protocolo antes de la campaña; este plan no los presenta como resueltos. La validación natural independiente sigue pendiente y no se sustituye por generación sintética.

Verificación proporcional: primero unitarias de comportamiento; al cambiar JSON/contratos, workspace y comprobaciones CLI/session con DB temporal propia, según [testing.md](../rules/testing.md). Anunciar alcance/duración y resolver la autorización aplicable antes de pruebas largas. Las campañas remotas requieren protocolo, límites técnicos, coste y duración registrados; no son requisito para escribir el plan.

## Estado de ejecución

- [x] Revisar contratos, propietarios y manejo de errores.
- [x] Registrar requisitos y segmentación sin modelo.
- [x] Preparar el plan y documentar el contrato implementado con límites exactos (fase A, implementación inicial).
- [x] Implementar y verificar segmentación local sin pérdida de bytes (fase B).
- [x] Implementar diagnósticos tipados y catálogo de capacidades del adaptador (fase C, parte local).
- [x] Verificar precios/límites publicados y disponibilidad real de Decisions/Jev/Jev114: nueve combinaciones con contratos/recibos válidos, con omisión semántica de baseline registrada (fase C).
- [x] Integrar consulta/contexto, resultados, contribuciones, cobertura y trazas en recall-v1 y session (fase D).
- [x] Retirar el dashboard y toda reutilización de sus revisiones en recall/session por decisión del usuario; conservar integridad y datos históricos.
- [x] Preparar 18 casos nuevos, corpus ligado, tres particiones y protocolo con umbrales (fase E).
- [x] Completar dos revisiones sintéticas separadas y conciliar observaciones preservando versiones (fase E).
- [x] Ejecutar pruebas estructurales, suite offline y preflight de desarrollo sin proveedor (fase F, parte local).
- [x] Completar y probar ejecutor con diario durable, control de llamadas/tiempo/reserva monetaria y detención ante incertidumbre.
- [x] Ejecutar desarrollo, calibración y final secuencialmente, con las tres puertas superadas por el perfil congelado (fase F).
- [x] Emitir informe de calidad/recursos y cerrar el contrato local y validación sintética del perfil, con límites explícitos.

### Pendientes que no se presentan como completados

- [x] Diagnosticar la abstención de baseline con contexto: efecto aislado y corrección candidata con evidencia explícita en state; [ensayos y límites](../evaluation/baseline-context-diagnosis-2026-10-23.md).
- [x] Implementar y evaluar la corrección candidata en desarrollo: 30/30 resultados esperados con Decisions/baseline, incluido Aliso; sin cambiar defaults ni reutilizar la reserva final consumida. [Resultados y límites](../evaluation/baseline-evidence-30-results-2026-10-23.md).
- [x] Eliminar el dashboard del alcance actual; no queda una integración de UI pendiente.
- [ ] Validación natural/humana independiente: aplazada por el usuario, se retomará cuando lo indique. Verificación externa de fuentes e integración Codex permanecen en sus hitos respectivos.

La implementación inicial y su verificación local están documentadas en [contrato vigente](product-contract-v1.md), [verificación de código](../evaluation/product-contract-v1-verification.md) y [banco revisado](../evaluation/product-contract-v1-bank-verification.md). No se marca M0 completo antes del cierre de sus puertas pendientes.

[Cierre de fases y evidencia real](../evaluation/product-contract-v1-results-2026-10-23.md). La parte local y el perfil congelado pasaron; los pendientes explícitos anteriores no se descuentan ni ocultan con un porcentaje global.
