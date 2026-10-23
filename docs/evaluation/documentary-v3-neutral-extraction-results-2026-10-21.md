# Extracción documental completa con atributos neutrales y guía corregida

Se ejecutaron las 116 entradas conocidas y sus dos revisiones separadas. Hay 112/116 aceptadas por ambos críticos (96,6%), frente a 72/116 (62,1%) en la tanda anterior. La puerta sigue fallida: una entrada es parcial, cuatro reciben alguna objeción y dos pierden aceptación previa. No es una medición de recall, verdad mundial o rendimiento de producción; no se adopta el mecanismo.

## Contrato y ejecución

La primera revisión de la guía V5 detectó una contradicción sobre constantes bajo cuantificación, ambigüedades de clase/sentido y modalidad, y una separación insuficiente entre dependencias y aplicación semántica. Se preservó ese dictamen y se creó la guía V6: conserva igualdad literal universal sin inventar testigos compartidos, clases literales sin resolver sentidos por conveniencia, bases de preguntas según su propio texto y cobertura visible de límites expresivos. Las instrucciones de entrega quedaron en el paquete, fuera de la guía general. La segunda revisión, en el mismo contexto, no identificó contradicciones que impidieran una extracción experimental; no certificó su fidelidad futura.

El vocabulario conserva el candidato de dos atributos neutrales y operates_mode con roles ref. No hubo cambios de Rust, producto o predeterminado. Guía V6 es versión de instrucciones de desarrollo; no es la cohorte reservada V6, que sigue sin abrirse.

Diez autores de contexto nuevo recibieron únicamente su propio paquete de fuente o pregunta, sin raws anteriores, etiquetas, mapas de pares, métricas ni feedback de validación. Entregaron una vez. El coordinador conservó los bytes originales, ensambló mecánicamente y auditó con --assembled-only. Cada lote recibió dos críticos nuevos y separados con idéntico texto/raw/validación/contrato, sin ver otro dictamen. Las veinte revisiones pasan integridad de hashes, IDs, orden, dimensiones, paths y citas. Son contextos de la misma familia, no validación humana ni independencia de familia; el banco es desarrollo conocido.

## Métricas y comparación íntegra

| Métrica, denominador fijo 116 | Tanda anterior | Tanda nueva |
| --- | ---: | ---: |
| Entradas entregadas | 80 | 116 |
| Estructuralmente completas | 80 | 115 |
| Parciales | 0 | 1 |
| Sin entrega | 36 | 0 |
| Completas y aceptadas por ambos críticos | 72 | 112 |
| Puerta completa superada | No | No |

Las cuatro dimensiones son fidelidad textual, completitud, atribución y preservación operativa. Crítico A: 460 fieles, dos defectuosas y dos inciertas sobre 464 dimensiones. Crítico B: 458 fieles, tres defectuosas y tres inciertas sobre 464. Hay dos desacuerdos de dimensión, ambos en Risco. No se sustituyeron por un consenso favorable.

La unión por tipo y SHA256 de texto conserva exactamente las mismas 116 entradas, sin duplicados. De las 72 aceptadas antes, 70 se conservan y dos se pierden. Siete antes no aceptadas pasan a aceptadas; otras 35 aceptadas nuevas eran entregas ausentes en la tanda interrumpida. Dos siguen sin aceptación. El saldo es +40, pero recuperar transporte no equivale a mejorar causalmente el modelo. Cambiaron guía y contextos de revisión; los raws, dictámenes y puntuaciones anteriores no se recalculan.

## Cuatro entradas pendientes

| Entrada nueva | Hallazgo conservado | Consecuencia |
| --- | --- | --- |
| s6-008, Risco | Un crítico acepta; el otro duda de capability como lectura única de «funciona» y señala la ambigüedad omitida. | No aceptación conjunta; cierre íntegro no resuelve por sí solo la modalidad. |
| s6-018, Iris | Se representa el horario, pero la recepción habitual queda unsupported; ambos críticos señalan pérdida de completitud. | Parcial; pierde la aceptación anterior de s4-068. |
| s6-051, Duna/Loma | La clase de Loma cita la ocurrencia de «impresora» de Duna; ambos críticos rechazan atribución. | Los offsets válidos no acreditan el referente correcto. |
| q6-020, expediente Delta | Ambos críticos dudan de part_of para «imágenes del expediente» frente a asociación neutral. | Pierde la aceptación anterior de q4-001; el intervalo correcto no resuelve el genitivo. |

Las objeciones son dictámenes de revisión, con sus incertidumbres explícitas; no una adjudicación semántica adicional. No se repararon salidas ni se cambiaron etiquetas para eliminar los cuatro casos. Los 15 controles manuales anteriores siguen como controles del mecanismo, no como prueba de fidelidad automática ni como ejemplos correctos enviados al autor.

## Latencia, coste y verificación

La ventana registrada de orquestación, desde el primer despacho de revisión de contrato hasta el último dictamen comprobado, fue de 1.801,818 s (30 min 1,8 s). Incluye lectura, coordinación, esperas y recepción; no es latencia por consulta ni p50/p95 de producción. No se midieron recuperación automática, utilidad contra la base ni latencia de integración en esta tanda: su puerta previa sigue fallida.

No hubo llamadas nuevas a OpenRouter u otras APIs externas de clasificación: USD 0 nuevos en ese ledger. Hubo 31 contextos Codex nuevos: uno de contrato con dos turnos, diez autores y veinte críticos; 32 turnos de subagente completados. Coste y tokens Codex, incluido el trabajo del coordinador, no están cuantificados y no se declaran gratuitos. El ledger externo conserva 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes históricos desconocidos, excluyendo Codex.

Se verificaron diez preflights mecánicos, diez recepciones reales y veinte integridades de revisión; comparación y agrupación de objeciones se ejecutaron sobre los informes completos. Cargo fmt --all --check pasa. El código Rust registrado y el binario coinciden por hash con 097. Las 664 pruebas, 24 omitidas y 226 replays son evidencia de 097, no ejecuciones repetidas ahora. Los 15.139 archivos congelados del manifiesto anterior se verificaron antes del cierre; snapshot 098 preserva la nueva tanda y documentos. No se iniciaron servicios, puertos o credenciales ni se alteraron procesos del usuario.

## Objetivo pendiente

Los cinco requisitos siguen vigentes. Contrato y banco tienen implementación y controles locales; el paso 3 aún no cumple la puerta automática completa y conserva dos pérdidas de aceptación. Corresponde resolver recepción sin origen declarado, elección de citas repetidas con contexto explícito, y ambigüedades de modalidad/genitivo en una versión prospectiva, sin rescatar estos raws ni reetiquetar sus dictámenes. Otra extracción completa y revisión separada deben demostrar la corrección antes de utilidad automática, contraste contemporáneo con base, recursos/estabilidad y casos nuevos ciegos. La adopción solo procede después de esas pruebas; el objetivo no está completado.

Artefactos principales: outputs/iromem/evaluation/documentary-v3-neutral-configurations-2026-10-21/next-extraction-v6/{report.json,fidelity-comparison.json,failure-summary.json}; execution-events.jsonl conserva los handles y etapas. Las preparaciones V5 no despachadas y todos los controles/manuales/negativos anteriores permanecen preservados.
