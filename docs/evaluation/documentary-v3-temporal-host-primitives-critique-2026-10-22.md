# Crítica semántica de dos primitivas temporales de host

Fecha: 2026-10-22. Alcance: crítica documental de los dos núcleos fuente propuestos, sin aceptación de fuente completa ni auditoría de cuatro dimensiones. Las dos relaciones pueden expresar sus compromisos locales mediante un único átomo compuesto genérico. No se demuestra una necesidad de token de evento para estas incidencias aisladas. El uso ejecutable sigue sin estar autorizado: faltan garantías sobre la conservación del contexto fijo y no se ha ejecutado validación, comparación ni proyección.

## Material y frontera de lectura

Se examinaron únicamente `design.md`, `candidate-input.json` y `receipt.json` de cada uno de estos directorios:

- `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/relative-interval-host-design-v1/`.
- `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/bounded-operation-host-design-v1/`.

Se leyeron además `AGENTS.md`, las reglas backend/testing y el contrato `docs/evaluation/documentary-v3-local-contract-2026-10-21.md`. No se siguieron sus enlaces históricos. No se abrieron código de validación, matching o tests, informes de resultados, preguntas, contrapartes, gold, guías V6 ni revisiones. Las referencias a esos materiales dentro de los diseños/recibos son declaraciones de sus autores, no verificaciones de esta crítica. No se ejecutó código, Cargo, CLI del evaluador, APIs ni nuevas candidatas; tampoco se recalcularon hashes o citas.

El contrato admite filas dinámicas de predicado con roles tipados y separa validación estructural de fidelidad semántica. También advierte que las notas laterales no añaden reglas ejecutables. Esto permite discutir la representación propuesta sin atribuirle un cálculo temporal implementado.

## Zinc: acción y retraso pertenecen a la misma ocurrencia

La fuente conservada es «El reproductor Zinc se apagó tras cien minutos en Nacar.». El único claim `shutdown` contiene `action_undergoer_after_interval(subject=zinc, action_kind=switch_off, interval=100 minutos)`, positivo, `fact/occurred/description`. Su evidencia cubre el núcleo con «tras»; la cantidad conserva su propia cita. No hay una ocurrencia separada de apagado a la que se le agregue una duración por coincidencia de sujeto.

El predicado tiene una interpretación composicional legítima: participante afectado, acción y cantidad son operandos sustituibles; el operador temporal pertenece a la relación. Su ID no contiene Zinc, Nacar, el numeral ni una cláusula serializada. Es una **relación nueva declarada mediante formas existentes**, no una semántica temporal previamente operativa por haber registrado tres clases de argumento.

La lectura suficiente exige una sola ocurrencia local de la acción indicada, situada tras el intervalo expresado con respecto al origen contextual de esa incidencia. No exige duración de la acción ni funcionamiento durante el intervalo. El diseño prohíbe expresamente escoger como origen un encendido, reproducción o medición y prohíbe exportar ese origen como variable o slot.

Hay una precisión documental necesaria: «contextual no especificado» debe significar **fijo para esa incidencia aunque desconocido**, nunca un origen existencial elegible durante el consumo. Si se interpretara como «existe algún punto cien minutos antes del apagado», cualquier apagado permitiría fabricar ese punto y el operador perdería su contenido. La prohibición de selección ya está en el diseño y el recibo; conviene fijar explícitamente también la rigidez por incidencia. No hace falta convertir ese origen en evento físico ni resolverlo para representar el núcleo.

`bound:exact` está explicado adecuadamente como conservación del numeral afirmado sin cota explícita. No certifica una medición de precisión infinita, convierte unidades ni acredita cien minutos de funcionamiento. No corresponde degradar ese numeral a una cantidad desconocida por ausencia de metrología.

`en Nacar` sigue `unresolved`, con propietario `shutdown`; no hay un argumento temporal ni una relación espacial que lo resuelva. Esta deuda impide llamar completa a la fuente. Tampoco autoriza al consumidor a escoger una adjunción o eliminarla para publicar el núcleo como hecho libre de contexto.

## Cobre: funcionamiento y ausencia bajo el mismo tramo

La fuente conservada es «Cobre funcionó ochenta minutos en Nacar sin apagarse; no se midió su duración hasta apagarse.». El único claim `c1` contiene `action_interval_without_event(actor=cobre, action_kind=operate, duration=80 minutos, absent_subject=cobre, absent_action_kind=switch_off)`, positivo, `fact/occurred/description`.

El diseño declara un testigo de intervalo local común a la acción efectiva, su duración y la ausencia del evento. Su fórmula explicativa conserva ese mismo intervalo en los tres componentes. Por ello el átomo no equivale a una negación global de apagado ni a tres claims independientes. `absent_subject=cobre` liga la ausencia al sujeto reflexivo de esta incidencia, sin afirmar una clase física para el nombre.

La fila es genérica y composicional: ambos participantes, ambas acciones y la duración son argumentos separados. No contiene ochenta, Nacar ni el apagado concreto en el ID. Que su definición incluya un testigo interno y una ausencia acotada no la convierte por sí solo en un escape opaco. La legitimidad depende de mantener esa definición completa; la aridad sola no demuestra su significado.

Los ochenta minutos caracterizan un tramo afirmado de funcionamiento efectivo. No caracterizan el episodio total ni su máximo, una capacidad, autonomía o una duración hasta apagarse. «Un único testigo» sirve para correlación interna; no significa que Cobre solo haya funcionado una vez. No debe reinterpretarse el tramo como suma de fragmentos separados ni como un intervalo meramente contenedor de alguna actividad. El diseño habla de tramo de funcionamiento: esa condición debe mantenerse al formalizar `effective_action`.

Conviene incorporar aquí la misma precisión cuantitativa de Zinc: `exact` conserva el numeral literal afirmado sin cota explícita, y no certifica precisión metrológica. El recibo dice «exact interval duration» y la fórmula usa igualdad; ambas expresiones deben leerse en ese sentido documental, sin inventar medición o tolerancia. Es una aclaración de la declaración, no una nueva cantidad.

No se afirma apagado en el extremo del tramo ni fuera de él. La negación interna permanece limitada al mismo tramo operativo. La relación no permite resolver por sí sola una convención física sobre extremos de intervalos ni derivar un apagado terminal de esa falta de resolución.

`en Nacar` conserva `pending_host_context` ligado a `c1`. La segunda cláusula completa permanece `unsupported`; `su` y `hasta apagarse` conservan deudas expresas, sin antecedente elegido, medición negada global ni apagado afirmado. Estas deudas no obligan a rechazar el núcleo como propuesta parcial, y tampoco quedan resueltas por él. Esta crítica no pide resolver la preposición ni la segunda cláusula.

## Contramodelos lógicos y suficiencia local

Los siguientes son contramodelos de interpretaciones debilitadas, no casos ejecutados ni resultados experimentales. Se pregunta en cada uno si la representación debilitada podría ser verdadera mientras el compromiso fuente local fuera falso.

| Interpretación examinada | Contramodelo lógico | Consecuencia para los diseños entregados |
| --- | --- | --- |
| Zinc descompuesto en ocurrencia de apagado y cantidad temporal sin vínculo | La ocurrencia de apagado existe; la cantidad caracteriza otra relación u ocurrencia. Ambos átomos serían verdaderos, pero ese apagado no sucede tras el intervalo contextual afirmado. | El átomo compuesto excluye esa separación en su semántica declarada. No es una brecha demostrada del núcleo entregado. |
| Zinc con origen elegible | El apagado no mantiene la relación de cien minutos respecto del origen contextual fijo. Puede fabricarse otro punto anterior que haga verdadero un existencial libre. | Es una debilidad de esa interpretación del origen. La prohibición de selección la excluye; debe quedar inequívoca la rigidez por incidencia. |
| Zinc convertido en funcionamiento de cien minutos | El apagado ocurre tras el intervalo contextual, pero no hay funcionamiento durante todo ese intervalo. | El átomo entregado puede seguir siendo verdadero; la conclusión de funcionamiento sería falsa. Esa proyección está prohibida, no falta un hecho fuente. |
| Cobre separado en funcionamiento de ochenta minutos y ausencia de apagado sin intervalo compartido | La actividad caracteriza un intervalo I; la ausencia caracteriza otro J. Cada átomo sería verdadero y el no-apagado durante I podría ser falso. | El testigo común del compuesto excluye I/J distintos. No se necesita un token externo para correlacionar sus componentes internos. |
| Cobre interpretado como duración total o máxima | Existe el tramo afirmado sin apagado, pero el episodio se prolonga fuera de él. | El núcleo entregado sigue siendo verdadero; total o máximo igual a ochenta puede ser falso. Ninguna de esas proyecciones está autorizada. |
| Cobre proyectado a no-apagado global o apagado terminal | El tramo satisface la ausencia local; fuera de él puede haber un apagado, o puede no haberlo al extremo. | El átomo fuente sigue siendo verdadero y la conclusión global o terminal puede ser falsa. Mantener la ausencia dentro del testigo evita el salto. |
| Cualquiera de los hosts liberado de su contexto fuente fijo | La relación nuclear se cumple bajo otro contexto K1, pero no bajo el contexto fuente K0 cuya adjunción sigue pendiente. | El átomo aislado sin envoltura restrictiva sería insuficiente. La deuda registrada reconoce esta brecha de consumo; no demuestra que el runtime la bloquee. |

Bajo las definiciones completas declaradas, los primeros componentes se cuantifican juntos: no se encontró un modelo donde el compuesto sea verdadero y falle precisamente la correlación local que declara. Exigir un token visible para volver a probar esa misma conjunción interna añadiría una obligación no necesaria para estas dos fuentes aisladas.

Sí existe una brecha diferente si se pretende **correlación entre átomos**: dos relaciones existenciales sobre el mismo participante pueden ser verdaderas con testigos distintos, mientras la afirmación de que pertenecen a una misma ocurrencia es falsa. Compartir nombre, scope o duración no elimina ese contramodelo. Para tal extensión se necesitaría un portador de ocurrencia compartido o una adjunción estructurada equivalente con reglas de identidad. Los diseños no intentan esa extensión y no deben reclamarla. El ID local de claim identifica una fila documental; no acredita por sí solo identidad de episodio entre claims.

## Proyectabilidad actual y reparaciones antes del uso

La representación puede proponerse dentro de las formas dinámicas actuales sin que eso acredite validez efectiva del candidato. Esta crítica no inspeccionó el código operativo ni ejecutó el validador. Los recibos declaran validación y matching no ejecutados; sus hashes documentan una vinculación declarada, que no se verificó aquí.

Las notas semánticas y `coverage` no constituyen por sí solas una regla ejecutable que conserve el origen de Zinc o la restricción `en Nacar`. En ambos JSON, los argumentos del átomo no incluyen esa adjunción, y las dependencias declaradas de los claims cubren scope y participante. Cobre reconoce expresamente que su cobertura pendiente no acredita gating. Por tanto, no se puede afirmar que los hosts actuales estén listos para consumo seguro, aunque sus núcleos compuestos sean semánticamente suficientes bajo la declaración íntegra.

Antes de usarlos se requieren estas condiciones, sin implementarlas en este encargo:

1. Precisar documentalmente el origen rígido por incidencia en Zinc y la exactitud literal en Cobre. Conservar el mismo testigo operativo durante cualquier formalización de Cobre.
2. Mantener los hosts completos con texto, evidencia, estado, fundamento, ámbito y todas las deudas. Definir y verificar por separado cómo una restricción fuente pendiente impide descargar, sustituir o borrar su contexto en cada canal de consumo. Una anotación de cobertura sola no basta como evidencia de ese bloqueo.
3. Validar estructura, citas y hashes mediante un encargo posterior autorizado; un pase solo acreditaría esas comprobaciones. Revisar semántica por separado sin reemplazar los candidatos históricos para obtener aceptación.
4. No proyectar cantidades a duración operativa en Zinc, ni a duración total/máxima/autonomía en Cobre; no extraer negación global, apagado terminal, medición, origen concreto o localización. Una proyección distinta necesita contrato y comprobación propios.
5. Si un uso futuro requiere unir ocurrencias entre átomos, resolver antes la brecha de testigos separados mediante identidad de ocurrencia demostrada. No introducir un token nuevo para resolver una necesidad que estas incidencias internas ya cubren.

No se autoriza mediante este documento proyección, matching, extracción nueva ni activación en producto. El resultado es una crítica de dos primitivas fuente parciales: su composicionalidad local es defendible; su consumo ejecutable y el cierre de las deudas siguen pendientes. No acredita aceptación de las fuentes completas, fidelidad automática, utilidad medida, recursos ni latencia.
