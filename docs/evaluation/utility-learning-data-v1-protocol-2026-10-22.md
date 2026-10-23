# Datos nuevos para un piloto de aprendizaje de utilidad

Estado: preparación de datos, sin entrenamiento autorizado por un pase de datos ni pesos aprendidos. Fecha2026-10-22. La ejecuciónMLXconprefijo exacto supera el filtro físico preliminar y conserva los11erroresV1. Se permite investigar discriminación aprendida: modificar el orden equivocado requiere cambiar parámetros, no calibrar scores fijos. Esto no garantiza mejora ni sustituye comparación contemporánea/coste/estabilidad/V6.

Banco prospectivo96pares nuevos de pregunta/documento,12familias semánticas×8pares,4útiles/4irrelevantesporfamilia. Particiones por familia, no por nombres:6familiastrain(48pares),3development(24),3closed(24). Toda paráfrasis/inversión/derivado de una construcción queda en su familia y partición. Los64conocidos yV6no se usan como entrenamiento. Autores generan textos nuevos sin leer bancos/resultados anteriores niV6; la reservaV6ni se inspecciona para deduplicar textos. Raíz puede comprobar duplicados contra los64conocidos. Este banco es sintético y no establece generalización poblacional.

Familias fijadas antes de redactar:

| Split | ID | Estructura |
| --- | --- | --- |
| train | f01 | Referente aislado correcto frente a otro objeto con mismo atributo/valor |
| train | f02 | Atributo solicitado frente a propiedad asociada del mismo objeto |
| train | f03 | Hecho realizado frente a plan futuro o ejecución explícitamente no ocurrida |
| train | f04 | Valor observado y estado/límite de medición solicitados frente a accesorio |
| train | f05 | Corrección explícita de atribución del mismo artefacto frente a otro |
| train | f06 | Pregunta de varias partes y aporte concreto de una parte/condición frente a dato tangencial |
| development | f07 | Política de clase frente a hecho individual sin universalización |
| development | f08 | Uso efectivo frente a disponibilidad/capacidad/condición compatible |
| development | f09 | Contraste de configuración asumida en dos perfiles sin exclusividad frente a otro perfil aislado |
| closed | f10 | Estado de un mismo referente en periodos temporales diferenciados |
| closed | f11 | Necesidad/suficiencia de requisitos con actor/acción/scope preservados |
| closed | f12 | Vínculos de sujeto/objeto/roles entre varias entidades en una oración |

Cada fila: id/family_id/split/query/document/label/role/rationale. label=useful|irrelevant, role=A|B|C|none; useful agrupaA/B/Csin hacerlos equivalentes. Aaporta dato solicitado o estado afirmado pertinente; Bcorrige una premisa necesaria mediante incompatibilidad explícita del mismo caso; Caporta una parte/límite/condición/estado solicitado o contraste explícito del mismo atributo en dos contextos sin inferir exclusividad. noneincluye otro referente, atributo no solicitado, asociación temática, silencio que no niega, plan que no acredita ejecución, instancia que no acredita política y condición compatible que no excluye otras. Una fuente ajena aislada no se vuelve contrasteC; debe aportar explícitamente el contraste pertinente al caso consultado.

Usar español natural, nombres/valores nuevos, sin etiquetas o IDs dentro de query/document. Incluir mínimos contrastivos que cambien pertinencia manteniendo vocabulario/negación/longitud/posición; no permitir que «no», tamaño, nombre o orden prediga label. Preguntas multipartes conservan utilidad de una parte real sin presentarla como respuesta completa. No redactar frases ambiguas que exijan una regla nueva del gold tras medir. Mezclar orden de etiquetas dentro de familia.

Autores y crítico separados: el crítico recibe únicamente id/family_id/split/query/document, sin etiquetas/rationale/resultados del autor; publica juicio independiente por fila y dudas. Se preservan originales; todo desacuerdo se resuelve/documenta antes de congelar datos y antes de entrenar o puntuar. Los agentes son de la misma familia, no revisión humana ni independencia de familia. Autoría no puede presentarse como revisión independiente. No excluir filas difíciles para conseguir pase; corregir una ambigüedad prospectiva debe conservar original/motivo y mismo tamaño/familia.

Raíz no leerá textos/etiquetasclosedpara diseñar o escoger entrenamiento. Autores/crítico/adjudicador separado pueden reconciliarclosedantes de fijar gold y solo comunicar conteos/hashes. El modelo y raíz abriráncloseduna vez tras congelar pesos e hiperparámetros y superar development/regresiones. Los metadatos de familia no se introducen en el modelo. Las fuentes query/document son el único contenido semántico del input. Si no se puede mantener esa separación, registrarclosedcomo observado y no declararlo ciego nuevo.

Un pase de datos no permite entrenar todavía: falta protocolo independiente con checkpoint/control sin entrenar, arquitectura de adaptador, una corrida y presupuesto/hyperparámetros predefinidos, selección por developmenty puerta previa aclosed. No probar instrucciones/umbrales hasta acertar estos textos. Mantener0,5y el scorer de utilidad congelados. No convertir fallos ennegativos correctos por abstención. Los resultados de evaluación deberán conservar todas laspérdidas/extras y exigir ganancia, sin compensarlas.

Sin nuevas llamadasAPIdeinferencia o datos reales;costeCodexdeautoría/revisión desconocido. Archivos bajo `outputs/iromem/evaluation/utility-learning-data-v1-2026-10-22/`; no cambian producto/DB/secreto/runtimepolicy/Rust/ramas. V6sigue cerrada y se abrirá solo tras los pases originales del objetivo completo.
