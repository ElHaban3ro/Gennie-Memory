# Definiciones de evidencia: avance parcial, sin variante aprobada

Goal activo, producto y configuración predeterminada conservados. Se ensayaron tres formulaciones sobre las mismas18preguntas conocidas de V4/V5 y sus prefijos completos de siete recuerdos. Cada ensayo compara18controles Scoped Anchor V1 con18candidatas, alternando el orden, con un intento por operación. Son108HTTP sobre18preguntas conocidas, no108casos nuevos. V6 sigue congelada, sin llamadas.

| Diagnóstico | V2 compartida | V3 directa | V4 presuposiciones |
| --- | ---: | ---: | ---: |
| Regresiones iniciales sin resolver, de5 | 4 | 0 | 0 |
| IDs no permitidos de la candidata | 1 | 0 | 1 |
| IDs útiles perdidos frente a control contemporáneo | 4 | 0 | 0 |
| IDs útiles históricos perdidos | 5 | 1 | 0 |
| IDs útiles ganados frente a control contemporáneo | 0 | 3 | 4 |
| Coste candidata/control | 1,010× | 1,047× | 1,065× |
| Pasa calidad | No | No | No |
| Pasa recursos del piloto | Sí | No | No |
| Incorporar a producto | No | No | No |

Las ganancias no compensan ningún ID ajeno o perdido. Se exigió coste≤1,03×control por el pequeño margen del pase V5 anterior, y p50/p95≤1,5×; no se elevó el límite después del resultado. Todas las nuevas operaciones terminaron sin error técnico y con recibos completos. Un test manual que termina correctamente acredita ejecución y parser, no calidad semántica.

V2 añade definiciones compartidas y pregunta mediante referencias a ellas; no recupera las partes perdidas y añade pérdidas en controles. V3 pone partes, límites y estados de medición directamente en cada pregunta: corrige los cinco fallos originales, elimina los extras en este ensayo y conserva al control contemporáneo. Sin embargo, pierde el recuerdo06106 en la pregunta histórica06101: el perfil Silente supuesto es incorrecto para M4 y la memoria conecta explícitamente los perfiles de M4/M8. Se mantiene el gold y la pérdida, aunque el control contemporáneo también se abstenga.

V4 cambia únicamente la instrucción de corrección de V3 para incluir presuposiciones y el mismo target. Recupera esa refutación y conserva los IDs útiles históricos, pero ante «¿Qué avalista debo llevar para reservar Olmo?» también acepta «Para reservar Olmo se presenta el carné vecinal vigente en el mostrador». Exigir un carné no excluye exigir un avalista. El recuerdo permitido que niega el avalista también se conserva; añadir un recuerdo permitido no vuelve válido el extra. El testigo literal comprueba Olmo en ambos y no resuelve esta relación semántica.

Estas son diferencias observadas entre instrucciones, no una demostración causal general. Los18payloads de control V1 son idénticos entre los tres ensayos; los resultados anteriores no se sustituyen. Se conservan todas las omisiones de gold dentro del prefijo en los informes, además de la puerta de preservación. No se corrigieron etiquetas para rescatar una candidata.

| Consumo y tiempo HTTP del arnés | V2 | V3 | V4 |
| --- | ---: | ---: | ---: |
| Coste nuevo USD, control+candidata | 0,002942184 | 0,002995860 | 0,003022320 |
| Mediana control/candidata, ms | 310,79 / 296,56 | 294,07 / 299,06 | 309,05 / 314,73 |
| p95 control/candidata, ms | 365,12 / 396,59 | 410,13 / 337,13 | 408,00 / 381,93 |

Coste de la tanda: **USD0,008960364**,108HTTP, cero costes nuevos desconocidos; excluye Codex de programación/revisión. Los tiempos son de llamadas del arnés sobre candidatos ya recuperados: no son recall integrado, respuesta final ni garantía poblacional. Acumulado del objetivo:4966operaciones/23campañas completas, USD0,528274692conocidos y tres costes históricos desconocidos. Los tres diagnósticos no añaden campañas completas al contador.

198pruebas workspace pasan, build offline y formato correctos. La reversión del campo/instrucciones V2 restaura V1; V3 conserva state/namespace; V4 cambia solo corrección respecto aV3. El auditor Rust vuelve a calcular probabilidades, elecciones, testigo, OR de vías, pérdidas y recibos desde respuestas raw. Las fuentes ejecutadas se archivan antes de cambiarlas; la revalidación de V2 conserva todas las métricas/casos y distingue únicamente metadatos del esquema nuevo, sin reescribir su informe. No se archivaron los binarios antiguos completos: se conservan sus hashes. No hay política runtime nueva, dependencias, SQL, puerto, servidor o datos del usuario modificados.

Siguiente hipótesis: una elección por recuerdo que distinga aporte solicitado, refutación explícitamente incompatible y ausencia de evidencia, con partes/límites/estados en sus criterios. Investigar si elimina el salto de requisito a exclusividad y reduce el coste de dos choices por recuerdo; no se presume ninguna de las dos mejoras. Debe conservar estos18casos y las puertas antes de implementación real, campañas V5/V4/V2, alias, estabilidad y V6 ciega. No se abre V6 con estas variantes fallidas.

[Protocolo V2](scoped-anchor-definitions-v2-protocol-2026-10-19.md), [V3](scoped-anchor-inline-v3-protocol-2026-10-19.md), [V4](scoped-anchor-presupposition-v4-protocol-2026-10-19.md). [Resultados V2](../../outputs/iromem/evaluation/scoped-anchor-definitions-v2-2026-10-19/report.json), [V3](../../outputs/iromem/evaluation/scoped-anchor-inline-v3-2026-10-19/report.json), [V4](../../outputs/iromem/evaluation/scoped-anchor-presupposition-v4-2026-10-19/report.json).
