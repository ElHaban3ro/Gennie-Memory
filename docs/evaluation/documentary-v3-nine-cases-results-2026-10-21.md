# Cierre de nueve carencias de representación V3

Se corrigieron las nueve representaciones pendientes y se revisaron por agentes distintos de sus autores actuales. No quedan defectos textuales señalados en esa revisión acotada. **Este cierre corresponde a representaciones manuales conocidas; no cierra nueve casos de calidad de extremo a extremo, M5 ni el goal.** Los controles todavía muestran pérdidas de utilidad y respuestas incompletas.

[Informe estructurado](../../outputs/iromem/evaluation/documentary-v3-nine-cases-2026-10-21/report.json), [banco seleccionado](../../outputs/iromem/evaluation/documentary-v3-nine-cases-2026-10-21/selected-frames.json), [revisión consolidada](../../outputs/iromem/evaluation/documentary-v3-nine-cases-2026-10-21/fidelity-review.json) y [ejecución](../../outputs/iromem/evaluation/documentary-v3-nine-cases-2026-10-21/execution-provenance.json).

## Qué se corrigió

| Entradas | Representación y comportamiento |
| --- | --- |
| Fuente005 y pregunta001 | Conservan eliminación de obligación, suficiencia de «su credencial de alumna», beneficiario, ámbito y propósito explícito solo donde aparece. La pregunta pide el tutor, no una lista genérica de requisitos. La premisa es corregible. |
| Preguntas004,005,006,030 | El origen solicitado tiene clase tubería y vínculo literal `part_of` con un patrón de red. No se fabrica una tubería existente ni identidad entre redes anónimas. |
| Pregunta017 | Dominio universal de archivos crudos de Q4, plazo solicitado ausente y exigencia de plazo común. La fuente universal con cantidad literal constante satisface esa condición; el archivo B3 individual no la satisface. |
| Preguntas018,019 | Cardinalidad seis del grupo de parejas propietarias, sin atribuir seis o doce faros ni repartir una medición entre propietarios. Se conserva el contexto ambiguo de Salvia. |

El vocabulario añade únicamente `class_suffices(subject,class)` y mantiene las filas anteriores. Esta proposición conserva suficiencia sin inferir matrícula, permiso ni regla de acción. Todos los textos anteriores, propuestas fallidas, informes y etiquetas permanecen intactos.

El filtro nuevo `reference_pattern` compara restricciones de la respuesta, no identidad global. Falta de vínculo o clase diferente sin exclusividad declarada significa desconocimiento. Un candidato desconocido no se selecciona como respuesta acreditada; una corrección de premisa o un enlace explícito de contexto parcial conserva su vía separada. Una lista mixta conserva sus miembros acreditados como parciales, sin declarar completo el conjunto.

La corrección normativa exige coincidencia del átomo central y conserva incertidumbre de ámbito, contexto, ramas y dependencias. En el control real005→001 ahora se selecciona la evidencia parcial que antes se perdía; no se inventa tutor ni se publica una refutación completa. Los controles de otra persona y otro curso no se seleccionan.

## Evidencia y límites

- Las nueve revisiones validan completas. El banco total queda en **114 completas, dos parciales y cero inválidas**, frente a105/11/0 anteriormente. Las dos fuentes parciales066/067 conservan señales conservadoras sobre el genitivo ambiguo.
- La revisión textual separada cubre las nueve revisiones actuales. Para las otras107 se reutilizan juicios previos tras comprobar igualdad del contenido raw salvo el hash de la extensión monotónica del vocabulario. No es una nueva revisión ciega de116 textos ni una medición automática.
- **585 pruebas pasan,24 omitidas,cero fallos**;65 pruebas V3, ocho nuevas. Incluyen12 variantes de las cuatro preguntas de red, conservación de un miembro válido en conjunto mixto, persona/curso/guard normativo y límites de cantidad común universal.
- Los116 informes de validación anteriores se reproducen idénticos. Los116 informes nuevos y17 pares originales se reproducen idénticos con el binario final. Cinco entradas mal formadas del filtro no pasan como completas. Compilación y formato pasan.
- Los17 pares diagnósticos originales producen **cero respuestas completas**. No es un porcentaje de error: incluyen negativos y parciales. La política anónima y los grupos de seis propietarios siguen sin identidad acreditada. Fuente020→pregunta005 y fuente024→pregunta006 siguen sin selección, por lo que todavía no se acredita conservación global de evidencia útil. Se mantienen visibles para la siguiente evaluación.

La última medición automática sigue en **102/116 DTO válidos y93/116 aceptados por ambos críticos originales**. No se reemplaza por los juicios sobre correcciones manuales. No se modifica baseline, producto, README ni gráficas de rendimiento; tampoco se abre V6.

## Coste y continuación

Esta tanda añade cero peticiones a proveedores y **USD0 de API**. Coste y tokens de Codex desconocidos. Acumulado conservado:6129 operaciones lógicas,23 campañas emparejadas completas,USD4,565416508 conocidos y20 operaciones de coste desconocido, excluyendo Codex.

El siguiente trabajo es congelar una extracción automática nueva que use este contrato real, revisarla por separado y comprobar utilidad en los126 pares conocidos, incluidas las pérdidas observadas. Después corresponden baseline emparejada, recursos y estabilidad; solo con esas puertas aprobadas, casos nuevos y revisión ciega. El goal permanece activo en el paso3.
