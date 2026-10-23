# Descripciones locales de referencia: mecanismo implementado, calidad automática pendiente

Se añadió solo al evaluador una variante de término que conserva masa, procedencia y otras restricciones predicativas dentro de su propia ocurrencia. Resuelve la carencia de representación general detectada en miembros de conjuntos cerrados sin añadir propiedades globales a Reference ni afirmar restricciones como hechos independientes. Es una implementación nueva sobre controles manuales; no demuestra extracción fiel ni mejora de recuperación automática. El paso 3 sigue activo.

## Contrato y operación

El vocabulario debe habilitar `term_description_profile: {version: "local_reference_descriptions_v1"}`. El término tiene exclusivamente `kind: "described_ref"`, `id`, `constraints`, `resolution` y `evidence`. Admite entre una y 16 Proposition no recursivas, con juicio fact, fundamento description/specification y normative_scope null; cada una incluye directamente la referencia raíz. Se comprueban forma, citas, vocabulario, referencias, ámbito y dependencias con los validadores existentes. Se rechazan repeticiones semánticas exactas y uso en slots escalares: solo roles ref y miembros de restricciones con rol ref declarado admiten este término.

La descripción es una conjunción local al operando, sin compromiso existencial adicional. La modalidad y polaridad permanecen en el host. El comparador exige ancla compatible y equivalencia estructurada de todas las restricciones, sin depender del orden ni del nombre técnico del ID. Reutiliza mapas explícitos sin eliminar los descriptores. No hay subsunción general ni equivalencia por parecido textual.

Dos cantidades exactas distintas, en la misma unidad literal, atributo, objeto y contexto, prueban incompatibilidad entre descripciones. Un valor solo se excluye de un cierre si además pertenece positivamente a su dominio y es incompatible con todos los miembros. Nombres diferentes, procedencias diferentes sin exclusividad, ausencia de una masa o unidades no convertibles no acreditan esa incompatibilidad. Coincidir con un miembro no prueba admisión, existencia ni uso efectivo.

Las restricciones internas nunca ingresan al inventario de claims que responde procedencia o descarga guards. Sus calificadores se comparan dentro de la descripción; los externos se conservan como condiciones del host. El perfil soporta contexto interno temporal/configuración; un guard interno queda unsupported y pendiente, sin inventar su satisfacción. No se promete matching general de calificadores alpha-renombrados, instanciación de cuantificadores a individuos ni inferencia entre propiedades. Las diferencias de status/fundamento/contexto mantienen el resultado pendiente.

Los testigos conservan el host íntegro como source_constraint y los términos descritos completos. Si faltan restricciones, los answer_terms se vacían; no aparece un ref desnudo como respuesta completa. Otros claims independientes conservan su elegibilidad. Los consumidores existentes que exigen ref simple no se convierten en proyecciones implícitas de propiedades.

## Evidencia ejecutada

| Verificación | Resultado |
| --- | --- |
| Suite offline completa | 639 pasan, 24 omitidas, 0 fallan |
| Pruebas nuevas de descripción / total motor V3 | 12 / 103 pasan |
| Casos CLI manuales fijados antes de ejecutar | 12/12 |
| Informes históricos idénticos en bytes | 226/226 |
| Compilación y formato | pasan |

Las pruebas incluyen cierre con masa distinta e igual, pertenencia positiva, conservación del miembro completo, ausencia de promoción de procedencia desde negación, guard no descargado, múltiples miembros sin intercambiar propiedades, contexto histórico interno/externo, orden/renombrado de IDs, comparación con mapa explícito, cantidades internas incompatibles, descripción desconocida junto a un hecho independiente y errores de perfil/forma/cita/raíz/ámbito/slot. El control de mapa prueba la operación local; no acredita toda la ruta de cuantificación y bindings.

Los 12 casos CLI se guardaron con expectativas y hash antes de ejecutarse. Son anotaciones manuales sintéticas, incluidas propuestas incorrectas deliberadas; no extracción ni revisión ciega. Las 226 reproducciones corresponden a 100 pares manuales, 116 validaciones y diez recibos congelados. No se cambió ninguna etiqueta ni raw histórico.

Se conservan fallos de desarrollo: visibilidad de métodos del nuevo módulo corregida tras error de compilación; un control del mapa carecía del namespace y produjo Unknown, corregido restaurando contexto y hashes del fixture; el primer driver CLI leyó el sobre externo como resultado interno, corregido sin cambiar producto ni expectativas. Los intentos y salidas originales permanecen disponibles. No se ejecutaron agentes nuevos tras su cierre; no hay revisión independiente nueva del código o de su semántica.

## Métricas vigentes, coste y próximo trabajo

No se repitió extracción automática: siguen 101/116 completas estructurales y 98/116 aceptadas por ambos críticos, puerta fallida. Las 12 pruebas/12 casos manuales no sustituyen esas métricas. La utilidad manual histórica sigue en 36/39 no ambiguas, cero none seleccionadas de 60 y tres respuestas completas. No se ejecutó utilidad sobre raws automáticos, baseline, recursos/estabilidad de producción ni V6. No hay adopción ni cambio de política predeterminada.

Cero llamadas nuevas de proveedor, USD 0 de API, cero subagentes nuevos. Coste/tokens de trabajo Codex desconocidos; no se declara gratuito. Sin medición de latencia del producto ni ahorro de tokens. Acumulado de proveedor intacto: 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos, excluyendo Codex.

El siguiente trabajo es revisar por separado esta extensión y su guía prospectiva junto con las demás causas de extracción pendientes: Atom frente a Proposition, atribución local de clase, procedencia explícita y condiciones de eventos pasados. Después, preparar otra versión congelada para los 116 textos completos con entrega semántica separada del ensamblado y revisión separada completa. Mantener guards internos o bindings no soportados pendientes, nunca borrarlos para pasar. La puerta de fidelidad completa precede a utilidad automática y comparación de calidad/recursos; V6 sigue reservada.

Artefactos: `outputs/iromem/evaluation/documentary-v3-local-descriptors-2026-10-21/verification.json`, fixture `tests/fixtures/documentary-v3-local-descriptions-v1.json`, código previo, logs de intentos, controles, reproducciones y manifiesto.
