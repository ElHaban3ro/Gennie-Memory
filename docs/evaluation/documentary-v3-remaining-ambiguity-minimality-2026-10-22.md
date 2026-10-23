# Minimalidad de las tres ambigüedades restantes

Fecha: 2026-10-22. Estado: propuesta documental retrospectiva, fuente aislada; no nueva autoría, validación, ejecución ni puntuación.

## Dictamen

1. **Matrículas:** puede conservarse íntegramente la atribución lexical y la relación neutral de entrega sin escoger acepción ni inventariar sentidos. Los carriers necesarios ya existen. Esto no acredita fidelidad de la entrega histórica ni resuelve su `unresolved` retrospectivamente.
2. **Bandeja/papel grande:** el atributo experimental `nominal_qualitative_descriptor` permite transportar las dos adjunciones de «grande». `associated_with` ya permite conservar el complemento nominal «de papel» sin elegir material o uso. Son dos lecturas prospectivamente defendibles, si se conserva el carácter local del papel descrito; no hay justificación para añadir `qualitative_size`, un atributo material, un uso o un DTO solo para este caso. No se ha demostrado cierre semántico mediante nuevas críticas.
3. **Lino:** distinguir aspecto de la duración y alcance temporal de la no medición requiere una arista operativa adicional. Ni el descriptor local ni un `time` literal ni dos hojas que repitan el mismo qualifier conservan esa diferencia. La propuesta mínima es un calificador cerrado de límite relativo de no ocurrencia, sin evento límite afirmado.

La puerta histórica de **116 completos y fieles continúa fallida**. Ninguna conclusión autoriza cambiar dictámenes, raws, cobertura, denominador ni significado de `complete` en entregas existentes. La auditoría de incertidumbre citada sigue vigente: cita, flag y honestidad sobre deuda no equivalen a conservación.

## Insumos y método

Se leyeron `AGENTS.md`, `docs/rules/testing.md`, los índices de memoria/decisiones y `documentary-v3-uncertainty-gate-audit-2026-10-22.md`. Los únicos documentos experimentales de fuente inspeccionados son los tres `*-input.json` siguientes, bajo `outputs/iromem/evaluation/quality-goal-2026-10-19/snapshots/109/attempts/consolidated-author-corrections-v1/known116-bundle/`:

| ID | Texto propio | SHA-256 del archivo de entrada |
| --- | --- | --- |
| `d-b0eb2f1b48f42710c9cf6480` | Las matrículas de Faro se entregan en la oficina de la esquina. | `908f12930e415a22b6d06d55e926a4687906715742fce52623ed1c53a0f766ff` |
| `d-f4152dc4613a39eccea79714` | Senda tiene una bandeja de papel grande. | `53913aad811b4233b7de74110fb88051ab89e1f4d99b2e6a030737851f1ad32a` |
| `d-eeb265867d41ca3c7ebc7f3b` | No se midió la duración del reproductor Lino hasta apagarse. | `71c7da97b1a38b58e4e6b70e7e605126ed361a872b30f4aebc6b2e9544da4815` |

Del diagnóstico `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-failure-diagnosis-v1/source-diagnosis.json` se extrajeron únicamente esos tres ítems de `items`. Se contrastaron la guía contractual `consolidated-extraction-contract-v1/author-guide.json`, `tools/evaluation/src/documentary_v3_io/readings.rs`, el esquema vigente en `documentary_v3/validation.rs` y la definición de asociación en `documentary-v3-local-contract-2026-10-21.md`. Se leyó la guía `snapshots/111/attempts/local-attribute-semantics-v1/semantic-guide.json` y su crítica de diseño; el apartado de bandeja del informe `documentary-v3-local-carrier-minimality-2026-10-22.md` aporta el candidato anterior, no una validación.

No se abrieron preguntas experimentales, contrapartes, gold, resultados de utilidad ni V6. Las búsquedas estáticas en código incluyeron accidentalmente líneas de tests que mencionan variables de pregunta; no se usaron como evidencia semántica de estos casos. No hubo Cargo, binario evaluador, API, servicio ni nueva extracción. Solo se escribe este informe. La verificación local consistió en lectura, hashes y offsets UTF-8; no demuestra comportamiento ejecutado del contrato propuesto. El coste de herramienta/proveedor externo es cero llamadas nuevas; el consumo/coste Codex carece de recibo y no se declara cero.

## 1. Matrículas: conservar la acepción abierta sin enumerarla

Evidencia propia: `matrículas` bytes `[4,15)`, `de Faro` `[16,23)`, `oficina de la esquina` `[42,63)`. La entrada ya contiene:

- `/raw/references/1`: grupo `records`, `declared_class="matrículas"`, `semantic_type=null`, asociación dirigida hacia `faro`.
- `/raw/claims/0/body/value/atom`: `property(records,delivery_location,office)` con `fact/state/description`.
- `/raw/references/3`: oficina con asociación a esquina; no se nombra un edificio identificado globalmente.
- `/raw/coverage/1`: sentido de matrículas marcado `unresolved`, sin propietario ni `issue_id`.

El diagnóstico conserva `partial` y dice que no falta carrier para el núcleo neutral. La guía de `delivery_location` exige explícitamente no especializar el objeto como inscripción, placa u otra acepción. NF07 permite `associated_with` para un genitivo de relación no especificada. Por eso **la falta de desambiguación lexical no exige por sí misma una lista finita de sentidos**: el referente local lleva exactamente la clase citada y las restricciones expresadas. No se está representando una intersección provisional entre sentidos inventados, sino el nominal aún no especializado.

La entrega es la relación de entrega localizada, preservada por `delivery_location`; no afirmar `located_at(records,office)`, custodia, posesión, entrega consumada de una matrícula individual ni un agente. El presente impersonal no obliga a crear un evento ocurrido. «De la esquina» conserva el descriptor nominal de oficina mediante el link débil autorizado; no permite deducir coordenadas, una planta o identidad global.

**Mínimo prospectivo:** precisar en la guía la distinción entre clase lexical completa y acepción ontológica no resuelta; reautoría y crítica de cobertura/propietarios. Cero atributos, DTO o código nuevos. El flag se retira únicamente de una futura autoría si la crítica confirma que toda restricción está operativamente conservada; no constituye el motivo para conceder completitud. Si una operación exige una acepción específica, esta representación no la proporciona.

## 2. Bandeja: dos propietarios del descriptor, un complemento neutral

Evidencia propia: `bandeja` `[16,23)`, `de papel` `[24,32)`, `papel` `[27,32)`, `grande` `[33,39)`. La entrada tiene una sola proposición `has_item(Senda,described_ref tray)`. Su restricción `/raw/claims/0/body/value/atom/arguments/1/value/constraints/0` atribuye `paper_tray_size="grande"` a tray. La clase de tray es `bandeja de papel`; no hay referencia separada a papel. Cobertura única `represented` oculta la segunda adjunción.

La nueva guía define `nominal_qualitative_descriptor` como modificador lexical explícitamente atribuido a un referente nominal, sin seleccionar dimensión física. Prohíbe transferir el modificador de papel a bandeja y viceversa. Esta es exactamente la capacidad que faltaba para «grande» sobre papel; no hace falta inventar tamaño, formato de papel o capacidad.

**Candidata A:** el mismo `has_item` y la misma descripción nominal neutral, con `property(tray,nominal_qualitative_descriptor,"grande")` local a tray. El `paper_tray_size` anterior puede conservar esa lectura bajo su selección específica; usar el descriptor lexical en ambas hojas evita atribuir una dimensión más fuerte por simetría del esquema.

**Candidata B:** `has_item(Senda,tray)`, referencias locales a tray y al nominal paper, link dirigido tray→paper `associated_with` por «de papel» y `property(paper,nominal_qualitative_descriptor,"grande")`. Paper es el referente nominal contextual del complemento, sin inventar hoja individual, papel almacenado, cantidad o suministro. El atributo describe ese nominal local; no afirma que todo papel del mundo sea grande. Las dependencias ligan descriptor→paper y tray→paper en el mismo ámbito de la lectura. No basta añadir paper sin esa conexión.

No es obligatorio anidar un `described_ref` de paper dentro del de tray. La oración positiva permite conservar la descripción local y su asociación como afirmaciones ligadas del mismo documento/lectura. La definición del atributo es lexical, no una propiedad física global de una clase; si la autoría no conserva esta localización y proyecta una afirmación universal o existencia de papel físico, se rechaza. No se propone ampliar recursión de `described_ref`.

**¿Material o uso obligan a más hojas?** No por el texto propio: la relación nominal no se especifica. NF07 y la definición vigente de `associated_with` ya conservan ese genitivo como relación dirigida y débil. Enumerar composición, contenido, admisión y destino de uso fabricaría un inventario que el texto no cierra. Las dos hojas solo separan la adjunción de «grande», conservando en ambas la relación de papel no especializada. Esto puede ser completo en el nivel lexical del contrato sin decidir material/uso. No se usaría la asociación débil si la fuente sí dijera explícitamente «fabricada con» o «para contener», pues se perdería una relación fuerte expresada.

**Mínimo prospectivo:** reutilizar el atributo experimental ya definido, dos hojas `reading_set` con la dimensión existente `reference`, mismo texto/hash, propietario alternativo correcto y vínculo nominal conservado. Cero atributos nuevos respecto de esa guía, cero DTO/código para la hipótesis de conservación. Exige reautoría y crítica separada de ambas hojas; dos hojas válidas no prueban inventario completo. Las etiquetas históricas `complete` y las críticas permanecen intactas.

## 3. Lino: el límite del aspecto no es el límite de la negación

Evidencia propia: `No se midió` `[0,12)`, `duración` `[16,25)`, `hasta apagarse` `[47,61)`. El raw tiene `event_kind=measurement`, `status=negated_occurrence`, `subject=lino`, `aspect=duration`, método no especificado, resultado null y `terminal_condition="hasta apagarse"` en `/raw/qualifiers/0`. No hay segunda lectura.

**Lectura A:** no se midió la magnitud «duración hasta apagado». El límite pertenece al aspecto medido. La guía de `duration_until_shutdown` declara esa dimensión sin inferir autonomía o duración operativa distinta; usar ese aspecto permite distinguirla explícitamente, sin resultado numérico ni apagado afirmado.

**Lectura B:** el intervalo sin medición de duración queda limitado por apagarse. El aspecto sigue siendo `duration`; el límite modifica el alcance de la no ocurrencia. No se concluye medición al llegar al límite ni después, ni valor de duración, ni fecha, ni apagado efectivamente ocurrido. Conservar `terminal_condition` idéntico en ambas hojas no distingue estos alcances. `Time` solo ofrece role/value; `Qualifier.kind` admite `guard/configuration/time`. Una cadena en un tiempo no especifica qué negación limita ni qué participante se apaga.

### Delta mínimo cerrado a criticar

Proponer exclusivamente `Qualifier.kind=relative_nonoccurrence_bound` con body exacto `{relation:"until", boundary_subject:<reference_id>, boundary_action:"switch_off"}`. El qualifier conserva los campos externos existentes de ID, ámbito, resolución y evidencia. No admite campos adicionales, `before`, `since`, texto libre como relación, fechas inferidas ni cualquier acción fuera de `switch_off` en este perfil inicial.

Su semántica es: **el evento negado al que se adjunta no ocurre dentro del intervalo previo al límite designado por el apagado del sujeto citado**. La fuente no fija inicio ni inclusión exacta del punto límite; ambos permanecen sin especificar y ninguna operación puede inventarlos. El boundary designa una condición de límite, no una ocurrencia afirmada; no es un segundo evento. Por tanto no necesita fabricar un claim de apagado ni un actor.

Guards mínimos: un único claim propietario explícito por adjunción; cuerpo `event/measurement/negated_occurrence`; sujeto de límite existente y ligado al mismo contexto; evidencia literal del límite; dependencia owner→qualifier y qualifier→boundary_subject; prohibir aplicación a un hecho positivo, al aspecto como dimensión o a otro claim. Una incompatibilidad o ausencia mantiene deuda, sin fallback favorable. Cualquier propagación o comparación futura debe transportar esta arista; no equipararla a `terminal_condition` ni descartarla de una firma.

**Coste:** un nuevo caso de qualifier y su validación/recorrido de dependencias, definición versionada y reglas conservadoras de propagación; dos hojas prospectivas de `temporal_aspect`. Es un cambio real de contrato/código si se implementa, aquí solo descrito. No se justifica un motor temporal general. Hasta implementarlo y someterlo a crítica, la alternativa B sigue sin carrier y el inventario permanece abierto.

## Controles mínimos prospectivos, no ejecutados

1. **Matrículas:** mantener clase lexical literal y destino, cambiar solo el atributo a `enrollment_delivery_location`: debe rechazarse la especialización no acreditada. Quitar `delivery_location` o los vínculos nominales dejando la cita debe perder conservación. La ausencia de acepción no autoriza borrar el núcleo ni inventar sentidos.
2. **Bandeja:** intercambiar únicamente el propietario del descriptor debe producir lecturas diferentes; eliminarlo de una hoja bloquea completitud. Quitar tray→paper debe romper la conservación del complemento. Reemplazarlo por composición/uso sin respaldo debe fallar fidelidad. Una copia de dos hojas con el descriptor en tray no conserva la alternativa paper.
3. **Lino:** sustituir el límite de la no ocurrencia por dimensión `duration_until_shutdown`, o moverlo a otro claim/sujeto, debe cambiar la representación y bloquear cualquier equivalencia de alcance. Eliminar la arista dejando «hasta apagarse» en evidencia debe perder conservación. Ninguna variante autoriza resultado, medición al final ni apagado ocurrido.

Son controles de representación aislada, no pares con preguntas ni resultados de utilidad. Reautoría, validación estructural y críticas futuras siguen separadas. No se afirma extracción autónoma, mejora de calidad, matching nuevo ni adopción.
