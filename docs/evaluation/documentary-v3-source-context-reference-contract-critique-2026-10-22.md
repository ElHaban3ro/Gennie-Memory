# Referencia a contexto fuente: crítica del contrato

Estado: revisión estática del [diseño](documentary-v3-source-context-reference-contract-2026-10-22.md) y de la [nota de transporte existente](documentary-v3-credential-existing-transport-review-2026-10-22.md). Se leyeron `AGENTS.md` y las reglas pertinentes de backend y testing. No se implementó el adaptador ni se ejecutaron Cargo, fmt, builds, API, servicios o pruebas. Esta crítica no consulta ni adjudica calidad de fuente, gold, campañas o resultados.

## Dictamen

El diseño satisface conceptualmente el objetivo acotado: conservar una referencia contextual a la ocurrencia de entrada que el runner identifica, con el documento íntegro como dependencia, y verificarla en un lector explícito de evaluación. No encuentro un bloqueo que exija cambiar el matcher, resolver κ, reconstruir c0 o autenticar la emisión histórica de la fuente. Las dos precisiones identificadas inicialmente quedan cerradas tras releer las correcciones del contrato; se conserva abajo su razonamiento como historial de esta crítica.

## Cierre estático de las correcciones

1. **Cerrado:** el contrato define ahora entregas lógicas distintas por pregunta y permite reejecutar una entrega con el mismo `delivery_id`; declara que la API sin estado no acredita frescura ni unicidad de ejecución. El lector sigue recalculando y comparando la referencia. Esto delimita su garantía de verificación actual y evita exigir un registro de ejecuciones o una prueba de emisión histórica que el objetivo no necesita.
2. **Cerrado:** se identifica `documentary_v3::context_budget`, con máximo de 1.000.000 pasos y 16 MiB acumulados de lecturas/reservas. Se fijan cargos por nodos/aristas, cadenas, hashes, copias, índices, igualdad y reclasificación, sin reinicio; la preinspección acumula las raíces sin clonar un árbol combinado y los bytes de lectura se cargan antes de evaluar. Las cotas estructurales continúan siendo conjuntas. Ya existe un umbral contractual comprobable para los controles de agotamiento.

No queda otro defecto concreto de diseño dentro del alcance revisado. El cierre es documental: falta demostrar en implementación que el presupuesto se conserva y que las comprobaciones y rechazos se ejecutan.

## 1. Hallazgo inicial cerrado: entrega lógica, ejecución y emisión

**P2 — Ambigüedad comprobable en la identidad de invocación.** La línea 9 exige `delivery_id` distintos para varias invocaciones; las líneas 55–61 requieren que `consume` reciba y revalide la entrega original. Bind y consume son ejecuciones distintas que necesariamente comparten esa entrega. Además, con el mismo manifiesto e input, dos ejecuciones independientes de bind pueden producir la misma referencia: las formas cerradas no contienen un registro de uso del ID ni lo necesitan para conservar el contexto.

La igualdad completa contra una reclasificación ofrece una comprobación efectiva del contenido y del vínculo registrado. No demuestra que el archivo presentado haya sido emitido antes por una ejecución particular de bind: una referencia construida por otro medio que coincida exactamente con la calculada también satisface ese criterio. Eso no vulnera el objetivo local, siempre que no se publique como prueba de emisión pasada.

**Cambio mínimo:** definir `delivery_id` como identidad de una entrega lógica congelada del runner, que contiene fuente, pregunta, selección y configuración. Preguntas o entregas lógicas nuevas reciben IDs distintos; bind, consume y los reintentos de la misma entrega pueden reutilizarlo. Declarar que consume acredita equivalencia con la referencia que la ruta Rust calcula ahora para esa entrega registrada, sin acreditar una ejecución histórica de bind. La asignación de ocurrencias por el runner sigue siendo una precondición externa de confianza.

**Comprobación de aceptación:** repetir bind y consume con la misma entrega conserva origen e identidad lógica; una entrega registrada con otra pregunta conserva `origin_id` y usa otro `delivery_id`. Un ID no registrado o una referencia desigual falla. No añadir un diario persistente ni firmas para probar una propiedad ajena al objetivo.

## 2. Hallazgo inicial cerrado: cota compartida

**P2 — Falta la definición del límite de trabajo.** La línea 65 fija bytes, profundidad y cardinalidades, pero exige precobrar recorridos, hashes, índices, igualdad, clonaciones y reclasificación a un presupuesto compartido sin identificar su contador, unidad o máximo. La línea 73 pide un control de agotamiento por `steps`. Con esa redacción, dos implementaciones pueden cumplir los máximos estructurales y discrepar sobre qué trabajo agota la operación; el control no tiene todavía un umbral contractual verificable.

**Cambio mínimo:** referenciar explícitamente el presupuesto existente que se reutilizará, con su unidad y cota, o fijar esos datos en este contrato si no existe uno apto. Precisar que consume incluye la referencia recibida, la referencia esperada y su comparación dentro de la misma contabilidad, y que la serialización canónica para hashes y escritura se reserva antes de materializar sus buffers. No hace falta imponer latencia de producción ni diseñar un sistema nuevo de recursos.

**Comprobación de aceptación:** un caso permitido estructuralmente que agote el contador global durante la reclasificación o comparación debe fallar sin publicar archivo exitoso; fraccionar el mismo trabajo entre wrapper y clasificador no debe renovar el contador. Los límites de lectura y duplicados JSON se comprueban también antes de construir una representación completa no acotada.

## Lo que ya está suficientemente delimitado

El origen no deriva del hash: el manifiesto permite dos ocurrencias de texto idéntico y cada entrega resuelve un origen concreto. Esto mantiene identidades distintas sin afirmar que sus referentes sean distintos. La autoridad del runner está expresamente asumida; exigir descubrir una sustitución coherente de manifiesto e inputs por esa misma autoridad ampliaría el encargo.

Bind obtiene el testigo de la ruta real, exige una selección única y contrasta documento, Claim y arrays redundantes; consume repite la ruta y compara la referencia completa. Es un mecanismo inspeccionable, no la aceptación tautológica de `interpretation.kind`. Las alteraciones propuestas pueden fallar por una comparación distinta de las declaraciones del caller. La implementación y los controles deberán demostrar que esa comparación efectivamente ocurre.

La conservación de documentos y deuda posesiva, los arrays de respuesta vacíos y la ausencia de un resolver de destino evitan que este lector publique la resolución de la pregunta como resolución de la fuente. La garantía termina en ese lector y en el wrapper conservado: no obliga a un LLM posterior, no prueba autoría, verdad, frescura ni fidelidad semántica de la representación. `source_indeterminate` ya recibe el significado acotado correcto.

Con las dos precisiones incorporadas al contrato, el siguiente paso puede ser implementar y verificar el transporte local descrito. Esta revisión documental no acredita comportamiento ejecutado ni autoriza adopción de producto.
