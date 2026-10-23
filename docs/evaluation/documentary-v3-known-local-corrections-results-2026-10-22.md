# Cuatro correcciones conocidas: contexto, descripción y requisito

El conjunto conocido corregido pasa de **103/116 a 107/116 documentos completos y fieles para ambos críticos**. Quedan nueve pendientes. Las cuatro correcciones tienen validación completa y doble revisión nueva: **32/32 juicios faithful**. No se cambiaron etiquetas ni resultados anteriores; la extracción autónoma original conserva **99/116**. El objetivo de cinco pasos sigue activo en el paso 3, sin adopción ni V6.

## Cambios y evidencia

| Documento conocido | Corrección | Comprobación |
| --- | --- | --- |
| Pregunta de autonomía máxima durante una ruta | `nominal_during` ligado al request y a la ruta existente; máximo, magnitud y demás campos intactos | Validador complete y 8/8 juicios fieles |
| Ensayo en sala baja | Propiedad lexical de la sala mediante `nominal_qualitative_descriptor`; conserva ensayo y ubicación sin inventar planta, altura o nombre oficial | Validador complete y 8/8 juicios fieles |
| Confirmación de viento sigue pendiente | `action_progress_state`, valor literal `sigue pendiente`; conserva asociación a viento y diez músicos de cuerda como afirmación independiente | Validador complete y 8/8 juicios fieles |
| El curso exige tutor para matricularse | Propiedad normativa del curso `enrollment_required_class`, valor de clase `tutor`; no inventa alumno, actor, acompañamiento, matrícula realizada ni suficiencia | Validador complete y 8/8 juicios fieles |

La primera corrección usa el mecanismo ya verificado en la tanda anterior. Para las otras tres, una [auditoría de minimalidad](documentary-v3-local-carrier-minimality-2026-10-22.md) y una crítica separada mostraron que bastan `property`, `Proposition` y referencias actuales: se añadieron tres atributos al vocabulario experimental y sus definiciones/guards a una guía versionada. No se añadieron DTO, crates, dependencias, predicados de producto ni reglas de equivalencia. La excepción de precedencia para la propiedad normativa impersonal se incluyó expresamente en el paquete autoral.

Los atributos no son certificados de fidelidad por pertenecer a la tabla. El primero conserva atribución lexical sin decidir dimensión física; el segundo conserva el estado y aspecto contiguo del proceso nominalizado sin crear ocurrencia o calendario; el tercero conserva la necesidad de clase para matrícula del curso explícito sin asignar participantes. La crítica debe rechazar portador incorrecto, frase proposicional arbitraria como valor, pérdida de condiciones o inferencias más fuertes.

## Procedimiento y fallos conservados

Se congelaron protocolos y entradas conocidas antes de recibir las entregas. La autoría de fuente y pregunta se mantuvo separada. Los dos revisores de la pregunta solo vieron esa entrega y el contrato; los dos revisores de fuentes solo vieron el paquete de tres fuentes con vocabulario y guía. No recibieron preguntas/contrapartes, gold, selecciones, diagnósticos, resultados del validador, razones del autor ni dictámenes del otro revisor. Son contextos separados de la misma familia, sin validación humana ni independencia de familia.

La autoría de fuentes preparó lectura mientras se finalizaba la crítica de diseño, pero esperó el cierre formal antes de escribir los raws. Los originales, crítica y hashes permanecen en `local-attribute-semantics-v1/design-freeze.json`.

La primera entrega de las tres fuentes calculó `vocabulary_sha256` sobre el archivo pretty JSON en lugar del valor JSON canónico que exige el contrato. Las tres validaciones originales fallaron por ese campo; se conservan en `validation/`. La única reparación mecánica autorizada cambia exclusivamente ese hash, con parche inverso comprobado, sin reescribir significado ni pedir otra tentativa semántica. Las tres versiones reparadas validan complete y fueron las entregadas a los críticos. La pregunta no necesitó reparación.

El receptor de revisión verificó hashes de paquetes y entregas, exactamente los cuatro IDs esperados y las cuatro dimensiones por revisor, citas UTF-8 y pointers resolubles. No se trataron rutas de archivos como punteros JSON. Las cuatro entregas fueron aceptadas; no se filtró una fracción favorable del lote.

## Métricas y alcance

| Métrica del conjunto mixto conocido | Antes | Ahora |
| --- | ---: | ---: |
| Documentos estructuralmente completos | 106/116 | 109/116 |
| Aceptados por ambos críticos | 104/116 | 108/116 |
| Completos y aceptados por ambos | 103/116 | 107/116 |
| Pendientes combinados | 13 | 9 |

El inventario y los textos siguen siendo los mismos 116. Se validaron de nuevo los cuatro documentos modificados. Los otros 112 conservan entradas, validaciones y dictámenes exactos, ligados por hash al replay completo verificado en la tanda anterior y al **mismo binario**; no se afirma que se hayan ejecutado 116 validaciones nuevas. El lote combina vocabularios por documento: no constituye una campaña nueva con vocabulario compartido ni una extracción autónoma.

Pasaron **9/9 controles manuales del mecanismo** sobre textos sintéticos nuevos: tres consultas de propiedades exactas con portador nombrado y seis variantes con portador o atributo distintos. Ambos documentos de cada par validan completos; los tres positivos responden al escalar explícito y los seis negativos no transfieren el dato. Las variantes negativas son controles tipados, no extracciones fieles de preguntas naturales. Esto prueba que el carrier es consumido por la comparación existente; no demuestra utilidad en el benchmark, respuestas de tutor individual ni razonamiento físico/aspectual general.

Rust y el binario no cambiaron: SHA-256 `9eed0999a3d210f09f5cc46587cd6d6d9a9c28e3fea61c7616866f20bc59581d`. Se comprobó igualdad de todas las fuentes de evaluación contra el snapshot 110. Las **757 pruebas aprobadas y 24 ignoradas** corresponden a esa verificación anterior, sin repetir Cargo en esta tanda. Aquí se ejecutaron 16 comandos CLI locales: cuatro validaciones aceptadas, tres intentos originales inválidos por hash y nueve controles.

No se ejecutó comparación nueva de recuperación, clasificación remota, latencia de producto, estabilidad o V6. La utilidad conocida anterior 31/39, cero ajenos/60, permanece histórica. No se suman los tres positivos de los controles a ese benchmark, ni se actualizan gráficas de rendimiento con una métrica diferente.

## Siguiente cierre

Quedan: control/finalidad ambiguos de matrícula en una pregunta; sentido de matrículas; alcance de hasta apagarse; suficiencia referencial y posesivo; tres fuentes con en Nacar; tarea no especificada de páginas pendientes; y adjunción de grande en bandeja de papel. La lista exacta de nueve IDs está en `known-local-corrections-v1/report.json`.

La siguiente iteración debe conservar explícitamente atribución y límite de información sin forzar tarea, tipo de contexto, actor ni ocurrencia. Donde el formato existente sea suficiente, se corregirá su uso; donde no lo sea, se criticará el operador cerrado mínimo antes de autoría. Después se congelará una guía y vocabulario compartidos para la extracción completa con doble revisión, todavía obligatoria antes del contraste de producto/recursos/estabilidad y de la partición reservada. No se cambia la puerta 116 ni se considera que una cita o un flag unresolved reconstruya significado perdido.

También se retiraron del archivo operativo de próxima iteración flags heredados y contradictorios de campañas antiguas. El estado actual queda breve y explícito, con enlaces a los snapshots históricos intactos; ningún pase de una variante antigua sirve como autorización para saltar las puertas actuales.

## Coste y trazabilidad

**Cero llamadas externas nuevas; USD 0 nuevos de proveedor.** Ocho contextos nativos de subagentes, sin turnos adicionales sobre contextos previos. Coste y tokens Codex desconocidos. El ledger externo se conserva: 6129 operaciones, 23 campañas, USD 4,565416508 conocidos y veinte importes desconocidos, excluyendo Codex.

Artefactos bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`: `known-request-context-correction-v1/`, `local-attribute-semantics-v1/` y `known-local-corrections-v1/`. El último contiene consolidación e integridad de las revisiones. No hubo servicios, cambios de credenciales, publicación ni cambio de política predeterminada.
