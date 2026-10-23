# Referencia a contexto fuente: contrato previo a implementación

Estado: contrato revisado e implementado en evaluación local;849pruebas,32controles CLI y425replays históricos verificados. No modifica DTO fuente, matcher, perfil contextual o gold; la credencial sigue sin aceptación integral. La copia prospectiva se conserva en el expediente de ejecución. Responde al límite identificado en [la inspección de la ruta actual](documentary-v3-credential-existing-transport-review-2026-10-22.md): el testigo contiene la construcción íntegra, pero su JSON aislado no impone identidad de ocurrencia ni disciplina al lector siguiente.

## Semántica y frontera

La operación entrega una referencia al contenido de un Claim **en su ocurrencia fuente**, con el contexto original como dependencia opaca. Conserva la construcción y su referencia contextual; no resuelve κ, reconstruye c0, afirma que el antecedente no exista ni demuestra autoría, verdad o frescura. `source_indeterminate` significa resolución no publicada por el extractor, no ausencia de un antecedente determinado en c0.

El runner asigna un `origin_id` opaco por ocurrencia de entrada, nunca por hash del texto, palabra, scope local o referente supuesto. Dos ocurrencias de igual texto pueden tener IDs diferentes. Varias preguntas sobre la misma ocurrencia conservan origin_id y usan delivery_id de entregas lógicas distintas. Una reejecución de la misma entrega puede conservar delivery_id: esta API sin estado no acredita frescura ni unicidad de ejecución. La autoridad es el manifiesto externo congelado del runner. Esta API no genera identidades de producción ni pretende autenticar una autoridad maliciosa que controle manifiesto e inputs.

Se incorpora solo al adaptador Rust de evaluación. Las salidas históricas siguen intactas. El lector de esta nueva referencia exige el manifiesto y la entrega original: sin esa pareja no publica contexto utilizable. El contenido publicado sigue siendo contextual, no una proposición independiente ni una respuesta final. Otro consumidor o un LLM que ignore este contrato queda fuera de la garantía; no se presenta como integración de producto.

## Formas cerradas

CLI nueva: `source-context-reference-v1 bind|consume INPUT MANIFEST OUTPUT`.

Los IDs son strings no vacíos de hasta128bytes; los digests son SHA-256 hex minúscula de64caracteres. La serialización canónica para digests es `serde_json::to_vec(Value)` con mapa ordenado del evaluador, no bytes pretty ni el orden original del archivo.

Manifest:

```text
{version:"documentary_source_context_manifest_v1",
 origins:[{origin_id,source_input_sha256}],
 deliveries:[{delivery_id,origin_id,delivery_input_sha256}]}
```

Arrays no vacíos y hasta512filas cada uno. IDs únicos por tabla, cada delivery resuelve exactamente un origen. Mismo source_input_sha256 con distinto origin_id está permitido; origen duplicado, ausente o conflictivo se rechaza. El hash de contenido por sí solo no establece identidad de ocurrencia.

Delivery (entrada de bind):

```text
{version:"documentary_source_context_delivery_v1",origin_id,delivery_id,
 selection:{request_id,claim_id},
 pair:{vocabulary,source:{text,raw},question:{text,raw},pair_scope}}
```

El digest de source_input usa exactamente `{vocabulary:pair.vocabulary,text:pair.source.text,raw:pair.source.raw}`. El digest de delivery_input cubre toda esta entrega, incluidos selección, origen, destino de pregunta y configuración. Se comprueban contra el manifiesto antes de clasificar. `origin_id` no se interpreta como persona ni scope normativo.

Esta primera versión admite documentos V3 planos fuente/pregunta, completos por el validador. Usa el clasificador contextual existente, sin crear relevancia o selección. Selecciona exactamente un testigo `nominal_possessive_context` en el request indicado y cuyo Claim fuente sea el indicado. Ausencia, multiplicidad o tipo distinto no publican referencia seleccionada; se devuelven como error local con causa, sin contarlos como negativos semánticos aprobados. Las demás construcciones conservan sus rutas existentes.

Salida de bind:

```text
{version:"documentary_source_context_reference_v1",
 origin_id,delivery_id,manifest_sha256,source_input_sha256,delivery_input_sha256,
 selection:{request_id,claim_id},
 source_input:{vocabulary,text,raw},
 witness:Witness,
 interpretation:{kind:"source_context_reference",origin_id},
 answer_terms:[],answer_propositions:[],bindings:[]}
```

`manifest_sha256` usa el Value canónico completo del manifiesto. El testigo se toma de la ejecución real: no se fabrica ni se acepta un dictamen aportado por el caller. `source_document` debe ser exactamente el raw registrado; `source_constraint` exactamente su Claim seleccionado; cada array fuente redundante debe coincidir con el documento. Los arrays de pregunta y `request_constraint` deben coincidir con la pregunta de la entrega. Sus arrays de respuesta/bindings vacíos y la deuda posesiva se conservan. No se elimina el resto de la fuente para aislar el nominal.

Entrada de consume:

```text
{version:"documentary_source_context_consume_v1",delivery:Delivery,reference:Reference}
```

El lector revalida la entrega contra el manifiesto y vuelve a obtener la referencia esperada desde la misma ruta Rust. Exige igualdad completa de la referencia, no solo flags o hashes repetidos. Un cambio de origen, cuerpo, arrays, selección, pregunta, respuesta o interpretación se rechaza. El resultado es exactamente la referencia verificada, utilizable solo con su dependencia fuente. La operación de lectura no acepta un contexto de destino ni un resolver de κ: no hay instrucción que pueda reinterpretar el posesivo. Tras cambiar de pregunta hay que generar otra entrega registrada, manteniendo el origen original. Desprender el envoltorio no produce una referencia válida.

## Límite y publicación

Los dos archivos se leen acotados:16MiB acumulados antes de parsear el segundo completo. JSON estricto sin claves duplicadas en ninguna profundidad; no se hereda la tolerancia histórica por ausencia de perfil. Campos ajenos o ausentes se rechazan. Profundidad64,100knodos,512owners y4096evidencias sobre el conjunto de entrada/manifiesto; listas de evidencia32. Se usa una sola operación de `documentary_v3::context_budget`: máximo1.000.000pasos y16MiB acumulados de lecturas/reservas. Un paso por nodo/arista visitado o comparado; las cadenas y hashes cobran al menos su longitud en bytes antes de hash/materialización; copias/reservas cobran también los bytes serializados multiplicados por el número de copias. La preinspección de las raíces de input/manifiesto acumula los contadores de nodos/owners/evidencia sin clonar un árbol combinado. Los índices/listas no se asignan antes de reservar sus tamaños. Igualdad consume nodos/bytes antes de comparar; reclasificación consume ese mismo presupuesto y nunca lo reinicia. Lecturas y parseo previos están acotados por los16MiB conjuntos; al entrar al presupuesto se cobran esos bytes antes de evaluar. Las cotas son unidades conservadoras del evaluador, no una medición de CPU/RAM. El límite invalida la operación completa; no existe selección sobre una vista truncada. Un rechazo no escribe un artefacto exitoso. La salida debe ser nueva mediante el adaptador existente, sin sobrescritura ni servicios.

No se cambia Cargo/dependencias, configuración, SQLite, autenticación, baseline o código de apps. El coste nuevo es local: verificación de hashes y dos copias de datos fuente/testigo; consume vuelve a clasificar para comprobar la referencia. Se mide tamaño/tiempo de estos controles aparte de producción. No se emplean llamadas API.

## Pruebas y aceptación del mecanismo

Exigir bind positivo y consume positivo de un nominal genérico; varias entregas/preguntas del mismo origen conservan Claim/H/U y documento; dos orígenes de texto idéntico conservan identidades distintas. Probar que hash de contenido solo no satisface la identidad. Cambiar el propietario solicitado nunca resuelve ni reescribe el posesivo fuente.

Rechazar origen/delivery duplicado, ID no registrado, digest cambiado, selección ajena o no soportada, raw parcial/inválido, manifiesto alterado tras bind, reference desnuda, source_input/Claim/H/U o arrays redundantes alterados, respuestas/bindings añadidos y origin intercambiado aunque el texto sea idéntico. Incluir duplicados JSON, sobrelímites acumulados/depth/steps, salida existente y negativa sin archivo exitoso. Los controles semánticos requieren fuente y pregunta completas; los inválidos se informan aparte. Verificar replays históricos exactos sin comando nuevo y suite del workspace.

La prueba acredita únicamente este transporte y lector de evaluación. La fuente conocida requiere crítica semántica separada con la nueva evidencia, preservando todo dictamen anterior. No autoriza convertir incertidumbre en acierto ni dar por cerrada la extracción autónoma: siguen el banco completo, guía/vocabulario compartidos, utilidad/base, recursos/estabilidad y casos nuevos ciegos antes de adopción.
