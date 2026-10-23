# Atributos neutrales y configuraciones completas: candidato prospectivo

El candidato añade únicamente dos atributos (`frequency`, `delivery_location`) y un predicado con roles de referencia (`operates_mode`) al vocabulario de evaluación. No cambia Rust, producto, política vigente ni resultados automáticos anteriores. La guía V5 está preparada, pero todavía no tiene revisión semántica separada. El snapshot ejecutado vigente sigue siendo 097; esta preparación no constituye un cierre 098.

## Problema y representación

Una unidad de frecuencia no prueba que el proceso sea muestreo. Una entrega no permite resolver por conveniencia una expresión ambigua como inscripción. Los atributos neutrales conservan lo explícito; los especializados siguen disponibles cuando existe evidencia. Las relaciones de uso afirmadas se conservan por separado con su configuración.

`operates_mode(subject: ref, mode: ref)` permite miembros descritos íntegros en restricciones de modos. Capacidad, admisión y ejecución efectiva conservan fundamentos distintos. Una configuración exclusiva debe limitar cada miembro con sus propios parámetros: trasladarlos a una condición exterior debilita el cierre. Un límite necesario no prueba existencia ni suficiencia; si el texto también afirma capacidad, esa afirmación positiva se representa separadamente. Ninguna descripción local se convierte en una propiedad mundial independiente.

La guía incorpora además atribución de citas a la cláusula y referente correctos, propiedad efectiva de operandos de calificadores y notación temporal prospectiva corregida. No añade conversión de unidades, subsunción de predicados ni una tabla completa de perfiles dimensionales.

## Revisiones y resultados negativos

Las versiones originales se conservan. Los primeros controles tenían una pregunta de dominio incompleta; otro intento empleaba `property.value`, cuyo rol genérico no admite el dominio de miembros descritos requerido. Se añadió el predicado tipado al vocabulario, sin alterar el validador para forzar aceptación.

Un contexto crítico revisó únicamente la versión 5, sin expectativas ni resultados. Detectó un cierre debilitado, una pregunta que imponía capacidad sin decirlo, la omisión de «física» y la posible pérdida de la afirmación positiva del texto. Se conserva íntegro su dictamen. El cierre debilitado era una propuesta incorrecta deliberada y permanece como control negativo.

La versión 6 cambia explícitamente los textos manuales cuando procede: distingue un límite condicional de una capacidad afirmada, conserva esta última como proposición adicional y añade la pregunta de ejecución efectiva. El antiguo control de pertenencia insuficiente no se reetiqueta: se conserva y se introduce otro caso con la afirmación positiva añadida. Esto es revisión de un banco manual, no rescate de raws automáticos ni mejora retrospectiva de una puntuación congelada.

## Comprobación ejecutada en esta continuación

- 15/15 controles CLI manuales cumplen sus expectativas; fuente y pregunta validan completas.
- Los 15 informes reejecutados son idénticos en bytes a los de la versión 6.
- Los controles incluyen dos respuestas completas de refutación; no son apoyos positivos.
- Se verifican hashes del código evaluador registrado y del binario contra la verificación 097: no cambiaron.
- El delta del vocabulario es exactamente versión, dos atributos y un predicado; las filas y perfiles anteriores permanecen iguales.
- Guía V5: cobertura exacta de todos los IDs de atributos y predicados.
- Se preparan diez paquetes nuevos con las mismas 83 fuentes y 33 preguntas de desarrollo, con IDs nuevos y sin raws/etiquetas/métricas anteriores en los paquetes de autor.
- Los diez preflights Rust de ensamblado pasan con `raw: null`: prueban transporte, nunca extracción ni calidad semántica.

La suite workspace de 664 pruebas y 24 omitidas, y los 226 replays históricos, pertenecen a la verificación 097; no se repitieron en esta continuación de datos/guía. Las comprobaciones por hash no amplían su alcance. No se midieron latencia de producción ni rendimiento de extracción. V6 permanece sin abrir.

## Puerta, siguiente ejecución y coste

La última extracción automática conserva 80/116 entradas completas, 36 sin salida y 72/116 aceptadas por ambos críticos. La puerta sigue fallida. Los 15 controles manuales no tienen ese denominador y no prueban mejora automática.

Los subagentes permanecen detenidos por petición del usuario. La revisión semántica separada de la versión 6 y de la guía V5 está pendiente; no se presenta la crítica de la versión 5 como aprobación de ellas. Los diez paquetes nuevos están preparados y no despachados. Antes de utilidad automática, comparación con la base, recursos/estabilidad y cohorte nueva, se exige la extracción completa y su puerta original: 116 entradas completas y 464 dimensiones fieles por cada uno de dos críticos, sin reducir a un subconjunto favorable. Es desarrollo conocido, no cohorte ciega nueva. No hay adopción en producto.

Cero llamadas nuevas a proveedores y USD 0 de API en esta continuación. No se iniciaron contextos ni turnos críticos nuevos. La preparación previa conserva un contexto crítico y un turno completado; tokens/coste Codex desconocidos. Acumulado histórico sin cambios: 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos; excluye Codex. No hay medición nueva de coste o latencia del mecanismo automático.

El objetivo permanece activo en el paso 3, con sus cinco requisitos intactos. Artefactos: `outputs/iromem/evaluation/documentary-v3-neutral-configurations-2026-10-21/`; el vocabulario candidato final está en `revision6/vocabulary.json`, no en el archivo inicial de la raíz. `next-extraction-preparation/` conserva protocolo, inventario, paquetes y preflights; `revision6-recheck-verification.json` registra la reejecución real.
