# Parciales cuantificados y coherencia de alias V3

Se corrigieron pérdidas y aceptaciones indebidas del mecanismo local, solo en evaluación. Las seis pruebas nuevas incluyen documentos validados y controles positivos y negativos; las etiquetas y resultados anteriores permanecen intactos. Esto permite continuar el paso 3; no acredita todavía calidad autónoma ni adopción.

## Causas comprobadas

1. El parcial descrito bajo un cuantificador verificado se descartaba cuando quedaba una deuda externa de fundamento. Ahora se reutiliza únicamente el mapa alpha de la proposición completa y se conserva el host íntegro, sin completar la respuesta ni publicar answer_terms. El individuo sin ancla y la fuente existencial no reciben ese permiso.
2. Un alias sin clase podía esconder clases divergentes entre raíces. Ahora se exige coherencia de clases declaradas, tipos y denotaciones de los componentes Same completos, incluidos compañeros del átomo y dominios/cardinalidad del mapa. Una discrepancia queda pendiente; no se inventa que las clases son disjuntas.
3. El mapa dependía del ID elegido como representante de una variable. El resolutor acredita una única pareja de raíces enlazadas explícitamente por Same a las variables verificadas. Sustituye solo las raíces de las restricciones locales y conserva el mapa original y alpha_alias_roots en el testigo. Sin Same no hay rescate.
4. La primera corrección de alias exigía duplicar sus enlaces. Un control positivo válido falló antes del arreglo: el alias heredaba Faro de su variable, pero tenía links vacío. Ahora se comparan todos los vínculos efectivos del componente, deduplicados por tipo y objetivo localmente Same. Se conserva correspondencia uno a uno con objetivos verificados. Otro contexto, un vínculo adicional o la ausencia de Same siguen sin seleccionar el parcial. No se borran condiciones para conseguir el positivo.

Los logs anteriores al arreglo conservan los fallos de parcial, clase, dominio tipado, representante y asociación heredada. La primera revisión crítica encontró dos bloqueos; la revisión siguiente detectó además el fallo de vínculos heredados. Ambas revisiones estáticas permanecen originales. La última corrección de vínculos fue ejecutada y verificada por el coordinador, sin atribuir un nuevo dictamen al crítico. Los subagentes siguen cerrados.

## Verificación ejecutada

| Alcance | Resultado |
| --- | --- |
| Suite offline del workspace | 656 pasan, 24 omitidas, cero fallos |
| Descripciones locales | 29 pasan; seis pruebas nuevas desde snapshot 092 |
| Controles CLI manuales previos | 12 + 6 pasan |
| Informes históricos reproducidos | 226/226 idénticos en bytes |
| Build offline y formato | pasan |

Los casos nuevos se comprueban en el motor Rust con documentos validados; los 18 CLI son reejecuciones de controles anteriores, no casos nuevos ni extracción. No se midió latencia de producción ni una mejora porcentual de recuperación. Los tiempos del harness no reemplazan esas mediciones.

## Límites y siguiente puerta

La desigualdad con un tercero conserva su veto anterior en el patrón cuantificado. No puede eliminarse sin demostrar correspondencia de las restricciones y sus extremos o irrelevancia acotada. El matching general de calificadores alpha-renombrados y otros contextos no acreditados sigue pendiente. La corrección no promete equivalencia arbitraria de nombres ni inferencia de identidades a partir de descripciones.

Preparar la guía prospectiva de representación con estas reglas y congelar una nueva extracción completa de los 116 textos conocidos. Se necesitan todos los documentos y la revisión separada de las cuatro dimensiones antes de utilidad automática, comparación contra la base, recursos/estabilidad y casos nuevos ciegos. No reusar ni reparar raws fallidos. V6 permanece reservada.

La última extracción automática sigue con 101/116 completas y 98/116 aceptadas por ambos críticos; puerta fallida. La utilidad manual histórica sigue en 36/39 no ambiguas, cero selecciones en 60 none y tres respuestas completas. No hay nueva medición automática de esas cifras.

## Coste y trazabilidad

Cero llamadas de proveedor y USD 0 de API nuevos. La preparación de esta tanda incluyó un contexto crítico y dos turnos estáticos antes del cierre solicitado; la continuación posterior no abrió agentes. Coste y tokens Codex desconocidos, fuera del ledger de proveedor. Acumulado sin cambios: 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos.

Artefactos: outputs/iromem/evaluation/documentary-v3-description-boundaries-2026-10-21/, con revisiones, fallos originales, logs completos, comprobaciones CLI, hashes y snapshot 093 del objetivo. Configuración vigente conservada; objetivo activo en paso 3, sin promoción.
