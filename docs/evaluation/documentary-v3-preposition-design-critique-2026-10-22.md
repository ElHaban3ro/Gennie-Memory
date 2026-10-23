# Crítica fuente-only del adjunto preposicional

Estado: diseño revisado críticamente, todavía bloqueado para implementación y aceptación. Este documento no contiene una instancia candidata ni prueba de ejecución. La propuesta `source_prepositional_attachment_v1` es reparable únicamente si deja de tratar el adjunto como metadata sin obligación semántica: debe conservar una **restricción contextual positiva de una ocurrencia**, aun cuando no se conozca su especialización o su adjunción interna.

## 1. Material y frontera de esta revisión

Se leyeron AGENTS.md, las reglas backend/testing, el contrato local documental V3, `nacar-source-design-v1/design.md`, su recibo y los textos/raw de los tres inputs que este refiere. Incidencia de alcance: una lectura inicial imprimió también el vocabulario embebido de esos inputs; después se restringió la lectura a text/raw. Por ello no se declara cumplimiento estricto de la restricción de leer únicamente esos dos campos. No se abrieron preguntas, contrapartes, gold, guías V6, otras revisiones, pruebas ni informes de matching. Los raws son antecedentes, no interpretación autorizada de las frases. Los enlaces históricos presentes en las reglas o el contrato no se siguieron. No se inspeccionó código para ampliar el diseño, ni se ejecutó Cargo, CLI, API o campaña.

Las tres fuentes autorizadas son:

| ID | Fuente literal |
| --- | --- |
| `d-38fb24e622a073a293ead090` | Cobre funcionó ochenta minutos en Nacar sin apagarse; no se midió su duración hasta apagarse. |
| `d-b7123caa690427c7c04d4c74` | Se reservó una batería nueva para Cobre en Nacar. |
| `d-5d8e6d45d1ee52311cdfb8b4` | El reproductor Zinc se apagó tras cien minutos en Nacar. |

## 2. Bloqueantes del diseño recibido

**B1. Guardar sintaxis no basta.** `preposition`, `complement` y una región de texto permiten reconstruir que aparecen palabras; el diseño no obliga a conservar que la situación descrita está restringida por ese complemento mediante «en». Llamarlo solo metadata documental y posponer toda semántica permite que una representación con el adjunto borrado tenga las mismas consecuencias publicables. Eso pierde información positiva de la fuente. La alternativa tampoco es afirmar ubicación física: «en» conserva un vínculo contextual restrictivo antes de decidir lugar, modo, soporte u otra especialización.

Contramodelo de conservación, sin constituir un caso de prueba: del primer texto no puede publicarse el registro de ese mismo funcionamiento de ochenta minutos como si no llevara «en Nacar». La inferencia existencial más débil de que hubo funcionamiento puede ser legítima bajo reglas expresas; presentarla como representación completa del episodio omitiendo una restricción no lo es. El nuevo carrier debe distinguir esas operaciones.

**B2. `unresolved` mezcla deuda del autor e indeterminación de fuente.** «Esta autoría no la acredita» describe un límite del trabajo realizado. No demuestra que el texto deje abierta la adjunción, ni autoriza a llamar completa su representación. Deben distinguirse el host compuesto que sí se conoce, la adjunción interna subespecificada por la fuente y una decisión de autoría pendiente. `interpretation:unresolved` en todas las filas no demuestra que todas tengan la misma apertura semántica. Tampoco la ausencia de clase para Nacar demuestra que no exista significado contextual.

**B3. La región como bolsa de nodos no conserva incidencia.** Un ID de claim, referencia o qualifier no identifica necesariamente su aparición dentro de una cláusula. En la primera fuente la región externa es la primera cláusula, con su funcionamiento, duración y ausencia de apagado. En la tercera es la cláusula de apagado con su intervalo anterior. Esa adscripción externa es positiva y debe quedar resuelta, aunque no se elija una proyección interna. En la reserva, la relación reserva–batería–beneficiario está afirmada; no debe deshacerse al dejar abierta la adscripción más estrecha de «en Nacar». Enumerar los nodos sin sus roles y correlación permitiría unir el complemento al objeto o evento equivocado.

**B4. Dependencia y sentido de aplicación están incompletos.** Decir que el adjunto depende del host no obliga al host a conservar su adjunto cuando se exporta o usa. Añadir la dependencia inversa como una arista ordinaria crearía un ciclo artificial. Se necesita composición estructural del host calificado, con propiedad de su restricción; los consumidores dependen de ese objeto íntegro. El pointer al dueño es identidad de ocurrencia, no una segunda arista recursiva de requisitos.

**B5. No se especifican cuantificación, guard y contexto correlacionado.** Ámbito y aliases son necesarios, pero no suficientes. Un adjunto dentro de negación, antecedente, alternativa, descripción o binder no se transforma en hecho positivo independiente. Tampoco «positiva» significa que se afirme existencia física del complemento o realización de un host hipotético. La restricción se evalúa bajo los operadores del host; su presencia afirmativa en la construcción no revierte la polaridad de este.

**B6. Conservar y diagnosticar como única operación perpetua no es progreso semántico.** Puede ser un archivo fiel de material pendiente, pero no cierra por sí solo una pérdida expresiva ni acredita utilidad. Antes de implementar hay que fijar qué obligación positiva aporta el operador y qué transformaciones prohíbe. No se acepta el mecanismo solo porque corte todas las rutas o produzca `not_supported` sin respuestas indebidas.

## 3. Reparación semántica mínima

La forma mínima es una **calificación contextual de una ocurrencia estructurada**. Su lectura es: «este host, con todos sus operadores, está restringido por este complemento a través de la relación contextual expresada por esta ocurrencia de la preposición». La relación es léxicamente anclada y subespecificada; no es asociación genérica, ubicación física ni una relación vacía. Su obligación operacional mínima es que el host calificado no equivalga al desnudo, y que el complemento no pueda cambiarse, transferirse o eliminarse silenciosamente. No se inventa una lista cerrada de sentidos.

No hace falta un parser universal, una ontología nueva ni un catálogo abierto de clases de host. Hace falta poder poseer un fragmento semántico compuesto y señalar sus incidencias existentes. Si el contrato todavía no lo permite, esa es una brecha concreta y el diseño sigue sin ser implementable.

La siguiente es una forma conceptual cerrada, no JSON válido actual ni autorización para añadirla al contrato:

| Campo | Forma y obligación exactas |
| --- | --- |
| `version` | Literal `source_prepositional_attachment_v1`. Habilitación optativa, incluida en el contrato/hash de vocabulario. |
| `id`, `provenance` | Identidad local y pareja `(input_sha256, domain_namespace)`. No se fusionan las tres fuentes. |
| `evidence` | Evidencia UTF-8 del sintagma completo; `preposition_evidence` y `complement_evidence` son subintervalos exactos no solapados que identifican ambos operandos. |
| `complement` | Término referencial completo, con sus restricciones y ámbito. Ausencia de clase/tipo no se rellena por el nombre. La mención nominal no añade existencia física por sí sola. |
| `host` | Una ocurrencia estructurada poseída: raíz y cierre efectivo local, con los operandos correlacionados, calificadores y ámbitos originales. No una referencia inventada que reifique la cláusula. |
| `owner_occurrence` | Localizador estable compuesto por documento, nodo raíz y camino tipado a la ocurrencia. Los segmentos pertenecen a una enumeración cerrada de campos/roles del contrato; deben resolverse de forma única. Un índice ordinal accidental o nombre compartido no basta. |
| `attachment` | Unión de `resolved` —localizador de esa incidencia— y `source_underspecified` —región estructurada poseída y evidencia de la adscripción externa, con frontera interna abierta—. La segunda no contiene una lista supuestamente exhaustiva de hosts. |
| `relation` | `lexically_anchored_context`: conserva la preposición exacta como operador restrictivo. `specialization` es ausente o un predicado estructurado expresamente acreditado por la fuente; el primer alcance solo admite ausente. Ausencia no significa carencia de restricción. |
| `author_status` | `complete_for_declared_form` o `pending`, este último con causa/evidencia de deuda. No decide la indeterminación de fuente ni cambia la aceptación por sí mismo. |
| `source_context` | Referencias a toda la cadena de scopes/binders/ramas, modalidad, polaridad, fundamento, guards y calificadores del host y complemento. Sus cuerpos y restricciones se conservan por el cierre efectivo; no se duplican manualmente como valores sueltos. |

El primer alcance contiene un adjunto por ocurrencia y contexto documental sin binder, rama ni guard. Esa restricción de implementación no permite borrar esas estructuras: se conservan y la forma queda fuera del primer alcance si aparecen. El diseño general explica dónde pertenecen; no promete su ejecución. «Cierre estructural local» no afirma inventario cerrado de lecturas o de contextos del mundo.

### Adscripción necesaria en estas fuentes

1. En Cobre debe resolverse el host externo como primera cláusula completa. La duración pertenece al funcionamiento observado y «sin apagarse» restringe ese mismo episodio. La apertura interna no convierte todo el host en desconocido. La medición negada de la segunda cláusula queda separada; no hereda Nacar por compartir sujeto. La reserva de una interpretación más estrecha no debe fragmentar un solo episodio en tres hechos sin correlación.
2. En la reserva puede conservarse la adscripción interna pendiente entre constituyentes de la cláusula, pero solo como subespecificación justificada mediante esas incidencias concretas. La fuente afirma una reserva con batería y beneficiario vinculados; «en Nacar» los restringe en esa construcción, sin afirmar que modifique simultáneamente a todos. La crítica no certifica que ya se haya demostrado un inventario de lecturas ni que la región propuesta sea mínima. Hasta esa revisión de fuente, `author_status=pending` y no aceptación completa.
3. En Zinc debe resolverse el host externo completo de apagado e intervalo anterior. «Tras cien minutos» mantiene orden temporal y origen de cómputo no dado; no se convierte en cien minutos de funcionamiento ni en autonomía. La apertura interna entre intervalo y apagado no autoriza a conservar solamente el nombre Nacar como contexto huérfano.

No hay que resolver lugar o modo para conservar esos tres compromisos externos. Tampoco hay que fabricar una variable de operación para ofrecer host al tercer texto.

## 4. Guards, cierres y presupuesto exigibles

- Evidencia y ownership: coincidencia UTF-8 exacta, sintagma/partes dentro de evidencia de host acreditada, ámbito alcanzable y localizador único. La simple inclusión de bytes no prueba adjunción semántica; necesita revisión fuente. Un complemento homónimo en otra ocurrencia no es intercambiable.
- Restricciones: host y complemento conservan todos sus vínculos, aliases, descripciones, tiempos, fundamentos y scopes. Cambiar preposición, complemento, incidencia, adjunción o contexto cambia la estructura documental. Un nombre repetido no une hosts ni episodios; la firma debe tolerar renombrado de IDs, sin usar el literal Nacar como excepción.
- Composición: el host posee el adjunto como restricción; exportarlo incluye ese adjunto. El cierre sigue dependencias explícitas e implícitas salientes y aliases pertinentes. El backpointer de ownership se verifica sin recursión de requisitos. Un ciclo semántico real queda inválido o pendiente según el contrato, nunca se corta silenciosamente para obtener respuesta.
- Alcance: no mover el adjunto fuera de binder, rama, negación o guard; no distribuirlo entre miembros; no convertir una variable en individuo ni un término nominal en objeto físico. Hosts fuera del alcance inicial quedan conservados como pendientes. Los hechos independientes acreditados siguen disponibles; compartir sujeto no les transmite ni elimina deudas.
- Consumo: ninguna ruta ordinaria puede tratar el host calificado como si su restricción no existiera. Las rutas que no representen el operador deben retener el host completo como candidato pendiente. Una proyección deliberada más débil debe declararse como tal y no contar como cobertura íntegra del texto. No se define aquí matching nuevo.
- Presupuesto: cobrar antes de visitar, resolver localizadores, expandir aliases, comparar, canonicalizar y copiar evidencia, IDs y cierres. Un único contador acumulado por operación, con visitas por documento/nodo/ocurrencia y guard de profundidad. Un DAG compartido puede reutilizar validación, pero cada materialización paga su copia. No reiniciar por host o lectura. Todo límite local agota la operación; no hay truncamiento favorable, aceptación parcial técnica ni exclusión del denominador. Cotas numéricas pendientes impiden implementar el diseño como contrato cerrado: deben fijarse antes, proporcionales a los límites operativos existentes, y medirse después.

Los contramodelos de diseño son claros: transferir Nacar del funcionamiento a la medición negada; llevarlo de la reserva al Cobre global; separar los cien minutos del apagado; equiparar host con y sin adjunto; resolver dos incidencias por el primer ID homónimo; desplazar una restricción bajo cuantificación hacia el documento global. Son transformaciones que esta forma debe impedir. Son razonamientos estáticos, no controles ejecutados ni evidencia nueva sobre otras fuentes.

## 5. Condición de aceptación y siguiente decisión

La aceptación fuente requiere una revisión fuente-only que acredite conjuntamente: restricción positiva conservada; host externo e incidencias correctos; incertidumbre de fuente distinguida de deuda autoral; modalidad/cuántificación/contexto íntegros; ninguna especialización no dicha; y cobertura de las otras cláusulas. Un carrier válido con deuda del autor no satisface esas condiciones. La indeterminación real puede representarse completamente como indeterminación solo cuando la fuente y su frontera están acreditadas, sin usar `open` como comodín para evitar interpretarla.

Persisten deudas independientes: semántica de «nueva», ausencia de apagado durante el funcionamiento, límite relativo de la medición negada y origen no dado del intervalo de Zinc. La última ausencia debe conservarse sin rellenarla; las demás necesitan su representación acreditada. El carrier de «en» no las resuelve. Los raws recibidos tampoco prueban su corrección por marcar coverage como represented.

La decisión útil inmediata es devolver el diseño para reparar B1–B5 y revisar el compromiso externo de cada una de estas tres fuentes antes de programar. Si no se puede expresar el host compuesto y la restricción contextual con esos límites, se registra la brecha como tal; construir otro contenedor de texto y bloquearlo todo no completa el trabajo. No se propone nueva instancia, mecanismo de matching ni cambio de producto.

La puerta de los 116 textos sigue íntegra, con todos sus casos y pérdidas, sin sustituirlos por este piloto. Aceptación manual de una forma no acredita extracción autónoma, recuperación útil, recursos ni adopción. `not_supported`, errores técnicos y guardas conservadoras mantienen sus denominadores y fallos; no se cuentan como éxito. Esta entrega verifica únicamente la crítica documental y no afirma ejecución ni resolución de los bloqueantes.
