# Auditoría de unión semántica y cierre de lecturas — 2026-10-22

## Conclusión preliminar

La objeción matemática es correcta: una disyunción inclusiva no necesita enumerar cada combinación de hechos compatible. No está demostrado que V1 confunda esos mundos con interpretaciones lingüísticas: el código no impone XOR a las hojas. Sí existe una diferencia contractual real entre declarar exhaustivo un inventario de análisis completos y certificar una fórmula de restricciones cuyo conjunto de modelos coincide con el significado íntegro de la fuente. Un mecanismo separado podría expresar esa segunda obligación, pero cambiar el nombre del cierre no demuestra equivalencia ni arregla pérdidas independientes.

Para bandeja, exigir una tercera hoja solo porque bandeja y papel pueden ser físicamente grandes carece de fundamento. Para Lino, la absorción de una lectura actualizante depende de demostrar que la lectura sin actualización es semánticamente legítima; los paquetes y críticas no lo resuelven. No hay base para aprobar el cierre de Lino por álgebra ni para declarar imposible toda representación fiel. Tampoco hay evidencia que autorice promover ahora un mecanismo nuevo o repetir raws hasta obtener aprobación. La puerta original de 116 entradas completas y fieles permanece íntegra.

## Alcance y evidencia

Auditoría documental, sin código modificado, Cargo, API, consultas web ni utilidad. Se leyeron AGENTS.md, backend.md y testing.md; sus referencias históricas no se usan como prueba de fidelidad. La búsqueda de memoria no encontró evidencia pertinente y no se utilizó memoria como fuente.

Fuentes permitidas bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`:

- `lexical-attachment-corrections-v1/review-packet.json`, SHA256 comprobado localmente y coincidente con ambas críticas `7ce52998bf5c9278e00bf176a814f687e947d4977bc54ca1e744bfeb780a013b`; sus guías y `blind-a.md`/`blind-b.md`.
- `relative-bound-contract-v1/known-source-candidate/review-packet.json`, SHA256 comprobado localmente y coincidente con ambas críticas `0179701bfdd264e27b1c17ca294431abb107c729a1e356dc41f0bf7aac5394f1`; sus guías y `blind-a.md`/`blind-b.md`.
- Solo los cuatro `*-input.json` autorizados en `consolidated-author-corrections-v1/known116-bundle`: `d-5b4860f3cf848e9dcbf9985a`, `d-38fb24e622a073a293ead090`, `d-b7123caa690427c7c04d4c74`, `d-5d8e6d45d1ee52311cdfb8b4`, con texto y raw de fuente. No se utilizan sus configuraciones de relevancia para decidir qué significa el texto.
- Contrato ejecutable actual: `tools/evaluation/src/documentary_v3_io/readings.rs`, especialmente `validate_inner` y `classify_with_context_inner`.

Las críticas son objeciones de agentes de la misma familia, no gold ni adjudicación lingüística. Se conservan todos los desacuerdos. No se leyeron preguntas contraparte, QFaro, gold, V6 ni resultados de utilidad. La pregunta restante sigue siendo una obligación general no adjudicada aquí. Esta exposición limitada no permite certificar las 116 entradas.

## Qué exige V1 y qué no demuestra

`documentary_reading_set_v1` admite de una a ocho hojas V3, comparte identidad del documento y exige validación de cada hoja. El estado se vuelve partial cuando el sobre o una ambigüedad están abiertos, hay dimensiones pendientes o alguna hoja no está complete. El resultado declara siempre `semantic_fidelity_proven:false`.

La guía léxica exige preservar la fuente completa en cada hoja y permite cerrar solo interpretaciones semánticas exhaustivas. El código comprueba las declaraciones y estructura; no decide exhaustividad lingüística. No agrega negaciones de claims ausentes ni exclusividad a las hojas. Por tanto no se encontró un operador ejecutado equivalente a «solo A y no B» al almacenar A.

El comparador recorre el producto cartesiano fuente×pregunta, exige solicitudes estables y solo da respuesta completa cuando todas las celdas entregan una firma coincidente. Esto comprueba consenso entre documentos representados, no verdad en todos los modelos de cada documento ni cobertura de todas las interpretaciones del español. Tampoco significa que cada hoja sea una descripción física total del mundo.

Hay tres niveles distintos: análisis lingüísticos posibles; fórmulas tipadas que expresan sus compromisos; modelos que satisfacen esas fórmulas. Pueden existir varios análisis con la misma fórmula, y una sola fórmula puede admitir infinitos modelos. Contar análisis, disyuntos o modelos no es intercambiable. La necesidad de un mecanismo nuevo solo quedaría demostrada si una representación exacta de restricciones no puede expresar o certificar su semántica bajo el contrato actual sin afirmar un cierre de análisis injustificado. Este audit no demuestra esa imposibilidad de V1.

## Condición de equivalencia, no excepción a fidelidad

Sea S el significado documental íntegro, con identidad, predicados, ámbitos, modalidad, presuposiciones pertinentes y compromisos de actualización explícitamente definidos. Sea E una representación candidata y τ su interpretación tipada. El requisito es `Models(τ(E)) = Models(S)`, bajo el mismo contexto admisible y la misma interfaz semántica. No basta `S ⇒ τ(E)`: eso permite borrar contenido. No basta `τ(E) ⇒ S`: eso permite escoger una lectura y excluir las demás.

Una proyección π que olvida distinciones solo permite igualdad `π(Models(S)) = Models(τ(E))` en el vocabulario retenido. Eso no acredita fidelidad completa si π ha eliminado una distinción de la fuente. Para conservar la puerta original, debe probarse que la proyección es sin pérdida respecto de todo compromiso documental exigible; si oculta uno, la representación pierde completitud aunque resulte útil para una pregunta concreta. Preservar la cita o el raw original como acompañamiento no convierte una restricción omitida en operativa.

Con `I` como hechos independientes obligatorios, la forma admisible sería `I ∧ (F1 ∨ ... ∨ Fn)`, con disyunción inclusiva, ámbitos y bindings internos explícitos, y sin negación por ausencia. Los Fi pueden ser parciales respecto del mundo físico; el conjunto E debe ser completo respecto de la fuente. Una rama `true`, un átomo opaco que copie la oración o un resto «cualquier otro sentido» sin semántica tipada no demuestra la igualdad. Marcar lo desconocido honestamente evita una falsa aprobación, pero no satisface automáticamente complete.

`A ∨ B ∨ (A ∧ B) ≡ A ∨ B` y `B ∨ (B ∧ C) ≡ B` son identidades. Solo prueban redundancia una vez fijado el significado de A, B y C y demostrada la legitimidad fuente de los disyuntos. Si S exige C en la rama B, sustituir `B ∧ C` por B añade mundos `B ∧ ¬C`; la absorción no autoriza introducir esa rama débil. «Compatible con el texto» es condición más débil que «interpretación de la fuente completa».

## Los siete textos fuente inspeccionados

### 1. Matrículas de Faro — d-b0eb2f1b48f42710c9cf6480

«Las matrículas de Faro se entregan en la oficina de la esquina.»

El raw conserva `delivery_location(records, office)`, grupo léxico matrículas, asociación records→Faro y clase literal office=«oficina de la esquina». El vínculo office→corner es `associated_with`. A critica el debilitamiento locativo; B considera operativa la clase nominal completa. No procede convertir ninguno en gold.

Una unión de acepciones de matrículas no repara una ubicación perdida. Debe demostrarse que la interpretación de la clase literal exige el destino restringido por la esquina, o representarse expresamente esa relación con la semántica correcta. Contraejemplo: una oficina relacionada administrativamente con una esquina pero ubicada en otra parte satisface una asociación genérica; no satisface la descripción locativa ordinaria. Si el nominal completo excluye ese modelo de forma operativa, el contraejemplo no vale para ese portador. Esa es precisamente la prueba pendiente, no una votación entre críticos. No se debe inventar inscripción, licencia o entrega ocurrida.

### 2. Bandeja de Senda — d-f4152dc4613a39eccea79714

«Senda tiene una bandeja de papel grande.»

Las dos hojas conservan posesión, bandeja→papel y `nominal_qualitative_descriptor` con propietario tray o paper. La guía define ese atributo como modificador léxico atribuido, sin elegir dimensión física. No se permite traducirlo silenciosamente a tamaño medido ni a «ambos objetos son físicamente grandes».

Si A y B son los compromisos de las dos adjunciones, con el mismo I, la unión inclusiva `I ∧ (A ∨ B)` admite modelos que satisfagan ambos. No requiere una tercera hoja por coincidencia de propiedades físicas. B argumenta que solo hay dos núcleos disponibles; A deja uncertain el cierre porque «de papel» puede tener sentidos diferentes y las hojas genéricas podrían o no cubrirlos. Ambos puntos pueden coexistir: exhaustividad de sitios de adjunción no demuestra suficiencia semántica del complemento.

La condición pendiente es si `associated_with(tray,paper)` más nominales locales denota exactamente la relación indeterminada «de papel», sin afirmar una hoja individual existente ni admitir una asociación ajena al complemento. Una asociación arbitraria con una empresa de papel puede ser demasiado débil. Un portador léxico tipado de complemento podría evitar especializar material/contenido/finalidad, pero debe tener semántica y restricciones de identidad propias. Conservar el rótulo «de papel» sin ellas no basta.

Dictamen: no hay omisión demostrada de una tercera predicación grande; no hay equivalencia completa demostrada del sobre actual. Es un candidato para aclaración semántica, no aprobación inmediata ni prueba de que un nuevo envelope sea necesario.

### 3. Lino — d-eeb265867d41ca3c7ebc7f3b

«No se midió la duración del reproductor Lino hasta apagarse.»

A=negación de medir la magnitud duration_until_shutdown. B=no ocurrencia de medir duration acotada por until/switch_off de Lino. El raw distingue ambos. Su guía deja inicio e inclusión exacta del límite sin especificar y no afirma ni niega medición en/después del límite. C=actualización positiva de medición al alcanzar el límite es una candidata interpretativa, no un hecho adjudicado.

Si B es por sí sola una lectura semántica legítima de toda la construcción, `A ∨ B ∨ (B ∧ C)` tiene los mismos modelos que `A ∨ B`. No falta cobertura de mundos actualizantes y no corresponde exigir C como hecho universal. Pero si la lectura temporal del español exige C, la rama correcta es `B ∧ C`; incluir B permite el mundo «no se midió antes y tampoco nunca después», que esa lectura excluiría. La fuente puede tener además la rama A, por lo que para un contraejemplo decisivo se necesita un mundo B∧¬C que tampoco satisfaga A. Deben fijarse magnitudes, eventos y tiempos para demostrarlo; no basta señalar ¬C.

La cuestión es lingüística y aspectual: entrañamiento, presuposición/proyección y enriquecimiento pragmático no son equivalentes. Las dos críticas la dejan uncertain; ninguna demuestra C obligatorio ni B suficiente. Tampoco cabe inferir apagado ocurrido solo por el infinitivo, inventar número o método, o tratar la frontera como tiempo conocido sin definir su denotación cuando no ocurre. Un envelope de restricciones no elimina esta obligación; puede expresarla con mayor honestidad, pero no aprobarla mientras siga indecidida.

### 4. Faro/Ina — d-5b4860f3cf848e9dcbf9985a

«En Faro se eliminó la obligación de tutor para Ina; basta su credencial de alumna.»

El raw distingue eliminación ocurrida, exención normativa de Ina en Faro y suficiencia de «su credencial de alumna», pero marca ese último segmento unsupported. Una nueva unión no puede borrar el segundo miembro ni conservarlo solo como clase textual. Debe fijar operativamente el posesivo y la condición de suficiencia en el mismo ámbito, sin afirmar presentación, posesión efectiva, matrícula ocurrida o prohibición de tutor. La exención no equivale a que nadie pueda acompañar a Ina, y suficiencia no significa requisito necesario. No se inspeccionó QFaro ni se propone resolver una pregunta desconocida para aprobar esta fuente.

### 5. Cobre ochenta minutos — d-38fb24e622a073a293ead090

«Cobre funcionó ochenta minutos en Nacar sin apagarse; no se midió su duración hasta apagarse.»

El raw conserva operación ocurrida, 80 minutos y sin apagarse, y una negación de medición dimensional; «en Nacar» queda unresolved. La primera cláusula I no puede perderse al modelar la segunda ambigüedad. El tiempo de operación no demuestra autonomía máxima ni resultado de medición. La ausencia de apagado está ligada al episodio de operación, no a todo el futuro. El posesivo su remite al mismo Cobre, sin fusionar los eventos de operar y medir.

La relación con Nacar debe permanecer operativa. No se puede declarar que Nacar sea lugar o modo solo para llenar el schema: si la fuente deja esa categoría indeterminada se necesita una relación contextual tipada exactamente adecuada, con dueño y ámbito conservados. Omitirla en todos los disyuntos sigue siendo incompleto. La primera cláusula restringe la interpretación local de la segunda; copiar las hojas de Lino sin examinar su compatibilidad con I no acredita fidelidad.

### 6. Batería nueva — d-b7123caa690427c7c04d4c74

«Se reservó una batería nueva para Cobre en Nacar.»

El raw conserva reserva ocurrida y beneficiario Cobre; nueva está unsupported y Nacar unresolved. Hace falta un descriptor de batería con atribución exacta, sin convertir nueva en fecha, carga, capacidad o instalación, más el vínculo contextual íntegro. Reserva no afirma posesión, uso ni entrega. Una disyunción no puede absorber `new(battery)` mediante la rama débil `reserve(battery,Cobre)` si toda lectura legítima exige nueva. Este es un contraejemplo directo al abuso de absorción, separado de la ambigüedad de adjunción.

### 7. Zinc — d-5d8e6d45d1ee52311cdfb8b4

«El reproductor Zinc se apagó tras cien minutos en Nacar.»

El raw conserva apagado ocurrido de Zinc y duración 100 minutos, con Nacar unresolved. Debe conservarse el vínculo temporal del apagado y su contexto. No se afirma que alguien midiera, ni resultado de autonomía máxima, ni relación con Cobre por compartir Nacar. Cambiar la incertidumbre de relación por una asociación genérica solo es fiel si la semántica de ese portador coincide con el compromiso contextual de la fuente, no porque acepte más modelos.

## Obligaciones de un mecanismo separado, si se justifica

1. **Contrato nuevo e inmutable.** Otra versión y otra semántica; no reinterpretar V1, hojas, labels, dictámenes ni raws previos. Distinguir completitud de representación de exhaustividad de análisis. No trasladar automáticamente complete de un contrato al otro.
2. **Semántica composicional.** Definir predicados, nominales locales, descripciones, existencia, negación, modalidad, fundamento y límites. Explicar cuáles son restricciones del discurso y cuáles hechos del mundo. Citas y cobertura sirven de trazabilidad, no de significado sustituto.
3. **Equivalencia de fuente completa.** Demostrar ambos sentidos, con testigos de inclusión y contraejemplos de exclusión por cada fuente. El modelo debe excluir mundos sin nueva en batería y sin relación locativa en oficina cuando el portador afirma esa relación. No admitir `true`, ramas de escape o proyección que pierda contenido explícito.
4. **Correlación.** `I ∧ ((A∧B)∨(C∧D))` no se reemplaza por `I∧(A∨C)∧(B∨D)` sin prueba de independencia: aparecen cruces A∧D y C∧B ajenos al texto. La correlación de eventos, propietarios y fundamentos integra la fórmula.
5. **Identidad y cuantificación.** IDs locales no establecen correferencia global. Compartir un testigo existencial y permitir uno diferente por disyunto produce fórmulas distintas cuando hay dependencias externas. `∀x(P(x)∨Q(x))` y `(∀xP(x))∨(∀xQ(x))` tampoco son equivalentes. Preservar grupos, masa, cardinalidad y bindings; no convertir papel en hoja ni exención de Ina en regla universal.
6. **Ámbito y condiciones.** No separar un guard de su claim ni promover contenido condicionado, normativo o de capacidad a hecho realizado. Conservar la unión de hechos independientes, contexto, tiempo, restricciones negativas y datos ausentes; ausencia de claim no equivale a falsedad ni desconocimiento afirmado.
7. **Invariantes en todos los modelos.** Una conclusión ψ requiere `E ⇒ ψ`, con E satisfacible y sin explosión desde ramas contradictorias. Tener el mismo ID o la misma cifra en cada hoja no prueba igualdad de referente/contexto. No se deben publicar bindings, números o respuestas completas obtenidos solo en alguna rama.
8. **Contexto parcial sin completar ausencias.** Puede transportarse E íntegra o una consecuencia tipada con procedencia y deudas claras, sin presentarla como fuente completa si se proyectó. Que un fragmento sea contextual no elimina el requisito de fidelidad del documento almacenado. La relevancia para consultas no vistas requiere contrato propio; no se ajusta E a una pregunta ausente.
9. **Verificación proporcional posterior.** Antes de implementar, adjudicar el lenguaje y el coste de un motor de restricciones: equivalencia, satisfacibilidad, cuantificadores, identidad y presupuestos pueden ser costosos o incompletos. Un timeout/unsupported debe conservar el denominador y no ser aprobación. Esta auditoría no ejecuta esos controles ni afirma que estén implementados.

## Decisión y puerta de salida

Hay una vía conceptualmente coherente: restricciones inclusivas exactas, sin enumeración de cada modelo ni falsa exhaustividad de análisis. No es automáticamente un mecanismo más fácil: requiere demostrar la equivalencia íntegra que V1 no verifica lingüísticamente. No existe aquí una demostración de necesidad técnica ni una certificación de factibilidad para Lino y los siete textos. Si no puede definirse una semántica operativa exacta para sus complementos, posesivos y actualización temporal, ese candidato no sirve para estas fuentes, aunque pase un schema.

La siguiente decisión debe apoyarse en esas obligaciones y sus contraejemplos, no en otra repetición de los mismos raws. Una adjudicación puede confirmar que dos disyuntos ya cubren un caso; también puede mostrar una pérdida real que exija otro portador. El resultado no cambia dictámenes anteriores. Solo una representación nueva, revisada con el mismo criterio de fuente completa y después comprobada sobre las 116 entradas sin excluir ninguna, podría abrir la puerta de fidelidad. Este documento no abre esa puerta ni mide extracción autónoma, utilidad o producción.
