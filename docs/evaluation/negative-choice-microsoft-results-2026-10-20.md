# Microsoft con motivos explícitos: mínimo fallido

Fecha UTC: 2026-10-20. La combinación de Microsoft con las nueve alternativas conserva las 17 evidencias útiles, pero acepta una fuente que no responde. **Falla el mínimo y no se amplía a casos difíciles.** El objetivo sigue activo en el paso 3; no cambia la configuración predeterminada ni se abre V6.

## Qué se probó

Las 16 peticiones se compararon antes de red contra las de Perplexity con motivos explícitos: solo cambia `model`. Se mantienen fuentes, preguntas, reglas, ocho analogías, criterios, orden, esquema y canonicalización de seis motivos negativos a `none`. Se validan alias y revisión servida de Microsoft; un modelo distinto, salida incompleta, namespace incorrecto o probabilidades inválidas se rechaza sin inventar una abstención.

Modelo `microsoft/microsoft-decision-1`, revisión `microsoft/microsoft-decision-1-20261009`. Es otra combinación de interfaz/modelo, no una reparación de la prueba Microsoft anterior con cuatro alternativas. Aquella falló 62/64 y sigue conservada. El preparador de dos brazos también rechaza esta variante antes de escribir una ampliación.

## Resultado completo

| Métrica mínima conocida | Resultado |
| --- | ---: |
| Respuestas válidas | 16/16 |
| Roles canónicos correctos | 63/64 |
| Escenarios de roles exactos | 15/16 |
| Evidencias útiles conservadas | 17/17 |
| Evidencias útiles perdidas | 0 |
| Selecciones ajenas | 1 |
| Coste informado | USD 0,001128330 |

El único desacuerdo es `cv3_01_00`: la pregunta pide el teléfono de contacto del pedido O7; la fuente dice que el formulario no lo exige. Microsoft lo llama refutación B. Que no sea obligatorio recoger un dato no demuestra que esté ausente ni refuta que se pregunte por su valor. La alternativa de ausencia real del registro sí se conserva correctamente como A.

El contraste incompatible Risco/Fino que Microsoft clasificó C en la tanda de cuatro alternativas recibe B aquí. Esa diferencia entre ejecuciones no se presenta como mejora causal estable; no son controles contemporáneos. Tampoco se compara esta corrida con el mínimo histórico de Perplexity para declarar superioridad de una familia. El mínimo conocido exige cero errores y este resultado no lo cumple.

No hubo fallos técnicos, reintentos, llamadas inciertas ni cargos nuevos desconocidos. Se completaron los 64 juicios y se conservaron todas las respuestas. No se midieron `recall`, p95/coste contra baseline, casos difíciles, estabilidad o generalización de esta combinación. El precio inferior al límite mínimo no compensa el error de calidad.

## Siguiente hipótesis preparada sin red

Las instrucciones actuales están en inglés y las preguntas/fuentes del banco en español. No se encontró un ensayo previo del idioma de instrucciones; los fallos no demuestran que esa mezcla sea la causa. La revisión crítica considera defendible una única ablación exploratoria de idioma, condicionada a equivalencia bilingüe comprobada antes de llamar.

Se preparó una tabla inglés/español de reglas, definiciones, plantilla de pregunta y criterios, sin fuentes del banco ni respuestas esperadas. La futura variante deberá conservar pregunta, textos incrustados, ocho analogías originales, IDs, claves, orden, modelo y parámetros. No basta traducir y añadir explicaciones: eso introduciría otro tratamiento. La primera revisión rechazó un desplazamiento semántico en A y el orden de criterios; se conservaron el borrador y ese rechazo. Tras corregir ambos puntos, el crítico aprobó la equivalencia de los 15 campos. Es revisión falible del mismo modelo, no humana ni conductual. Su revisión y estado se guardan aparte; en esta tanda no se implementa ni ejecuta esa variante. Incluso si se aprueba la traducción, faltarán igualdad estructural, mínimo propio, referencia contemporánea y todas las puertas originales.

## Verificación y coste

261 pruebas del workspace pasan con y sin HTTP/2; formato y compilación pasan. Se reprodujeron los 15 informes anteriores y este nuevo informe. Las pruebas comprueban que solo cambia el modelo, vinculan la revisión esperada y canonicalizan correctamente la salida Microsoft de nueve alternativas. La preparación inválida de dos brazos se rechazó antes de crear archivos.

Tanda: **16 operaciones/16 HTTP, USD 0,001128330**. Acumulado: **5.692 operaciones, 23 campañas completas, USD 0,573728358 conocidos +4 operaciones históricas de importe desconocido**. Codex de programación/revisión queda fuera; la preparación/revisión local de traducción no añade cargos API.

Se conservan bancos, gold, snapshots y todos los hashes de V6, con cero llamadas a esa cohorte. No hay política runtime, dependencia, SQL, credencial, servicio o proceso del usuario modificado. README y gráficas mantienen la ruta vigente.

[Protocolo](negative-choice-microsoft-v1-protocol-2026-10-20.md). Artefactos en `outputs/iromem/evaluation/contribution-local-microsoft-contrast-minimal-v1-2026-10-20`. Cierre de huellas y contabilidad: `quality-goal-2026-10-19/snapshots/042`.
