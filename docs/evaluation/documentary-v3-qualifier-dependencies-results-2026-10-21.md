# Dependencias efectivas de operandos de calificadores

Se demostró y corrigió un fallo de ejecución: una configuración conocida podía comparar por nombre una referencia cuyo contexto seguía pendiente y publicar una respuesta completa. El fallo se reprodujo por fuente y por pregunta, sin cambiar etiquetas ni raws anteriores. La corrección mantiene el positivo con contexto resuelto y bloquea publicación fiable con deuda alcanzable.

## Propiedad y resolución

Cada claim/request/reference declara sus dependencias estructurales directas. Un qualifier posee sus operandos: refs, described_ref, normative_scope y qualifiers internos de su body. El cierre efectivo sigue esas aristas implícitas y las dependencias declaradas de los nodos alcanzados, sin inventar ni escribir aristas en el raw. No hace falta repetir el operando de un qualifier como dependencia directa de su consumidor cuando no aparece directamente en ese consumidor.

El recorrido usa visitados y grafos finitos; Kahn detecta ciclos implícitos que el grafo declarado solo no mostraba. Se propagan resolución, guard satisfecho y las incidencias del validador pertenecientes a cada nodo alcanzado. No se aplica un veto global por incidencias de referencias ajenas. El índice de issues usa el documento original del informe, no posiciones de la vista ejecutable filtrada; el grafo sigue esa vista ejecutable.

La deuda fuente conserva la calificación existente. La deuda del request cierra todas las vías de publicación: no hay respuesta ni selección fiable. El candidato unresolved conserva request_constraint, source_constraints y pending_evidence con los testigos propuestos y sus motivos originales; esa propuesta no aparece como soporte publicado ni como memoria seleccionada. La validez estructural del DocumentResult sigue sin equivaler a elegibilidad efectiva o fidelidad semántica.

## Fallos e iteraciones preservados

1. El control inicial aceptaba un operando con dependencia desconocida. Tras corregir el cierre fuente, seguía aceptándolo por el lado pregunta: faltaba la comprobación común al final de request. Se conservan ambos fallos.
2. Un ciclo qualifier → referencia → qualifier terminaba por visited, pero se daba por resuelto. El control falló antes de añadir detección de ciclos; ahora permanece pendiente con payload completo.
3. unknown_vocabulary en semantic_type de un operando implícito no se propagaba. El control falló; ahora falla la publicación por ambos lados y sigue pasando el positivo con una incidencia en un nodo no usado.
4. La suite existente attribute_profile_mismatch_cannot_publish_and_does_not_erase_independent_fact detectó una regresión real: un índice calculado tras filtrar claims atribuía al claim válido la incidencia del eliminado. Se corrigió el mapa usando report.document y se repitió toda la suite. No se cambió la expectativa ni esa prueba anterior.

Un contexto crítico separado produjo cuatro informes estáticos, conservados. Señaló simetría request, propagación de incidencias e índices originales. No ejecutó pruebas ni consultó fixtures/resultados; recibió del coordinador descripciones de cambios y reproducciones, declaradas. La última relectura no identifica otro bloqueo concreto dentro del alcance revisado; no certifica calidad global.

## Verificación

| Alcance | Resultado |
| --- | --- |
| Workspace offline | 664 pasan, 24 omitidas, cero fallos |
| Pruebas nuevas | 3, con matriz positiva/negativa y payload |
| Motor V3 | 128 pruebas pasan |
| Controles CLI nuevos con documentos no inválidos | 8/8 |
| CLI anteriores | 18/18 |
| Informes históricos | 226/226 idénticos en bytes |
| Compilación offline y formato | pasan |

Los CLI nuevos cubren contexto conocido, deuda fuente/pregunta, ciclo implícito, tipo no registrado en ambos lados, incidencia ajena y anáfora pendiente. Verifican ausencia de selección y conservación de candidatos cuando la pregunta no puede resolverse. Son controles manuales del mecanismo, no extracción automática ni casos ciegos. No se midió latencia de producción o escalabilidad; preindexar dependencias/incidencias evita repetir búsquedas completas de esos arrays por nodo, pero no se publica un ahorro medido.

## Estado del objetivo y coste

La discusión de propiedad de operandos queda definida para la próxima guía: revisión estructural directa y cierre efectivo son comprobaciones distintas. Los dictámenes y puntuaciones de la tanda anterior no se reescriben ni se recalculan bajo esta aclaración. La última extracción conserva 80/116 completas, 36 unavailable y 72/116 aceptadas por ambos críticos, con cinco desacuerdos y puerta fallida.

Quedan la representación de frecuencias/entregas sin especialización inventada, el cierre íntegro de configuraciones y la evidencia de la cláusula correcta. Incorporar estas reglas y la notación temporal corregida en otra versión completa solo tras controles; después fidelidad completa, utilidad automática, contraste con la base, recursos/estabilidad y V6 ciega. El objetivo continúa en paso 3, sin reducir sus cinco puntos ni adoptar el mecanismo en producto.

Cero llamadas nuevas de proveedor y USD 0 de API. Un contexto crítico y cuatro turnos; coste/tokens Codex desconocidos, no gratuitos ni incluidos en el ledger. Acumulado sin cambios: 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos, excluyendo Codex. No hay servicios, puertos, credenciales o dependencias nuevos.

Artefactos en outputs/iromem/evaluation/documentary-v3-qualifier-dependencies-2026-10-21/: originales negativos, protocolos, ocho controles, replays, suite fallida/intermedia/final, cuatro críticas y verificaciones. Snapshot 097 conserva código, pruebas y documentos; todos los artefactos y snapshots previos se verificaron por hashes e inventarios.
