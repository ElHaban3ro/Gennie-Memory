# Referencias con evidencia distribuida V1

UTC 2026-10-21. Investigación local del paso 3. La tanda previa fue progreso: catálogo y matriz mostraron una restricción de contención de propietarios y una pérdida de compatibilidad sensor/sensores. No se modifica gold ni se llama al proveedor.

## Extensión explícita y límites

`distributed_reference.rs` introduce una ruta versionada de evaluación. Mantiene la frase nominal original y permite citar por separado cada vínculo con su propietario. Owners siguen outermost-first; el holder de cada eslabón debe ser el siguiente owner interno o subject para el propietario inmediato. Cada nodo/índice es local: nombres repetidos no prueban un ciclo semántico ni correferencia. El orden forzado de holders evita ciclos estructurales en la cadena declarada, sin inferir propiedad transitiva desde proximidad.

ownership_support conserva quote/bearer_quote/owner_quote, polaridad, rol, basis, modalidad de relación, aplicabilidad y context/configuration. Solo una relación de estado afirmada como fixed_identity, con fundamento descrito/observado/asignado, puede incorporarse a esa identidad. Un vínculo negado, normativo, planificado o corregible no se descarta silenciosamente: esta ruta lo rechaza como identidad fija; su contenido tendría que conservarse como hecho, premisa o lectura completa, o la operación sería unsupported. El adaptador no implementa por sí solo esa extracción ni certifica que el modelo etiquetó bien el texto.

La modalidad state del vínculo no se iguala a la modalidad occurred de una observación. Aplicabilidad bounded exige identity_interval y todos sus contextos/configuraciones como subconjunto de los contextos de uso. Una diferencia o una pérdida de condición se rechaza. identifier_only declara una restricción de nombre/referencia, no un derecho de propiedad válido para siempre. unknown/ausencia no equivalen a vigencia ilimitada. La pregunta tiene el contexto global del esquema existente; se conserva cada solicitud, sin seleccionar una favorable. Esta estructura no demuestra por sí sola la validez temporal de todos los usos ni expresa cualquier pregunta con intervalos divergentes.

## Clase literal frente a equivalencia afirmada

El registro versionado de clases se liga por SHA-256 antes de usarlo. No contiene entidades, IDs de casos ni alias de gold. Una vinculación class_binding declara equivalence=exact con una cita exactamente igual a toda type_quote; no acepta subclass, ID desconocido o cita de solo la cabeza. Una comprobación de borde impide recortar sensor dentro de sensores; no es un analizador lingüístico general. Clase semántica, clase léxica y clase ausente tienen namespaces disjuntos; null mantiene la clase léxica exacta y las clases fuera del registro no se convierten por conveniencia.

El registro contiene definiciones, no una prueba de que la cita sea equivalente. Una clase restringida de presión puede vincularse falsamente a sensor con forma/cita válidas. Esta falsedad se conserva como contraejemplo, no se etiqueta como rechazo automático. class_binding no se copia a ontology_type del dato: tipos de valor y filtros siguen su ruta separada. No se aplica stemming, borrado de artículos, case folding o tabla nueva de sinónimos a los textos.

## Implementación y conservación

La función antigua grounded_references::translate conserva su reescritor original. Se extrae un punto de extensión ReferenceRewriter para compartir la conversión de cláusulas, filtros y tipos de valor. Es un refactor del archivo compartido, no una afirmación de que su fuente quedó sin tocar: los resultados antiguos deben reproducirse íntegros. La ruta nueva entrega legacy y un sidecar con el raw original, registro y semantic_assertions_verified=false. El parser y resolver de hechos permanecen sin cambio.

`audit-distributed-reference <controles.json> <registro.json> <salida-nueva.json>` materializa y prueba dos lotes completos: ocho controles originales adaptados y ocho aclarados prospectivamente. Conserva además 13 guards con roster fijo y tres contraejemplos semánticos. Cada etapa queda visible incluso si falla la siguiente. Los errores/unsupported conservan denominador y salida existente no se sobrescribe. No hay extractor/prompt de modelo, runner remoto o política de producto.

El lote original conserva sus textos/condiciones y ambigüedades. El nuevo lote explicita negación en LA02, relación de pertenencia en LA03, composición/observación en LA06 e inmediatez/búsqueda aguas arriba en LA07; no se compara su porcentaje como mejora sobre un banco idéntico. Una expectativa nueva de qualification_required=true para B fue adjudicada frente al contrato anterior: B resuelta ya comunica corrección y no exige ese flag de límites adicionales. Se elimina solo esa condición redundante del nuevo control, manteniendo negación explícita, selected y B, sin modificar resolver/V1. Se conservan fixture, informe y test inicial fallidos.

Un bug del generador compartía el diccionario de intervalo y modificaba a la vez uso y soporte a hoy, citando ayer. El guard corregido cambia únicamente el uso; el caso donde ambos metadatos mienten se conserva como contraejemplo semántico aparte. No se supone que acuerdo entre campos pruebe respaldo de sus valores.

## Puertas y revisión

Dos críticos nuevos sin historial revisan los 19 pares manuales (8+8+3), con IDs anónimos, registro y construcciones, sin condiciones/resultados/mapa. Un crítico estático separado revisa código. Declarar exposición, incertidumbre y diferencias sin cambiar etiquetas para aprobar. Este es un diagnóstico de representabilidad y conservación, no una medición de extracción ni validación humana.

El agregado necesario incluye también los contraejemplos: si una equivalencia o intervalo falsos producen selección, la puerta completa falla aunque los dos lotes y los guards pasen. No se ejecuta el extractor mientras esos riesgos sigan sin resolver. Probar integridad del roster, datos descartados, modalidad/intervalo, clases restringidas, slots/premisas/lecturas, Unicode/nombres, padres y namespaces. Reproducir cuatro bancos manuales, diez informes históricos y ambos auditores previos.

Cero llamadas y USD 0 nuevos; no se informa latencia de recall/end-to-end ni pase de recursos. Una tubería posterior tendría que contar todas sus etapas y cumplir las puertas originales, sin amortización supuesta. Se conservan V6, snapshots previos, defaults, etiquetas, README/gráficas, dependencias, SQL, dashboard y servicios. Goal activo hasta mejorar calidad demostrada, controles difíciles, recursos/estabilidad, cohorte nueva ciega y adopción verificada.
