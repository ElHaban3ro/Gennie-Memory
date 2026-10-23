# Revisión semántica ciega de pregunta/documento

El revisor recibe únicamente id,family_id,pool_id,split,query,document. No recibe etiquetas, razones, resultados, cuotas, cardinalidades esperadas o archivos de autor. No leer protocolo de autoría, author/final/reconciliation, bancos previos,closedoV6. No inferir respuestas por IDs/posición/tamaño/negación o por balance supuesto. Evaluar cada documento por sí mismo respecto de la pregunta: un pool puede no tener útiles o contener varios, sin imponer cuota.

Juicio útil (`label=useful`):

- A: aporta valor o estado explícito del atributo solicitado en el mismo caso. Que ese campo no conste o esté ilegible puede ser información útil; no convierte otro campo, otro objeto o una mera regla de plantilla en estado del campo preguntado.
- B: incompatible explícitamente con una premisa necesaria de la pregunta para el mismo caso, sin asumir exclusividad, hechos no escritos o identidad distinta.
- C: aporta parte, límite o contraste pertinente de la configuración asumida, sin afirmar respuesta completa o exclusividad entre opciones compatibles.

Juicio ajeno (`label=irrelevant`,`role=none`): otro referente o atributo; asociación temática; silencio que no niega; opcionalidad de plantilla que no determina el dato ni su ausencia real; perfil ajeno aislado sin contraste que incluya el caso consultado. Un documento no se vuelve útil por vocabulario compartido o porque otro documento del pool sea pertinente.

No confundir una fuente que dice «campo X no registrado» con que su formulario no obligue a registrar X: esta última no determina el estado o valor del registro. Tampoco confundir ausencia del campo Y con estado del Xsolicitado. Conservar dudas de rol y de utilidad, sin resolverlas para cuadrar distribución. Ningún conocimiento externo o política inventada completa el texto.

Salida por fila: id,label(useful|irrelevant),role(A|B|C|none),rationale,ambiguity(boolean). Razón breve fundada en texto. Marca ambiguitycuando la fuente/rúbrica no permita un juicio único; no elimines casos difíciles. No edites datos. Revisión sintética entre agentes de la misma familia, sin afirmar revisión humana o independencia de familia de modelos. Sin inferencia de modelo,API,entrenamiento,Cargo o instalaciones.
