# Relación preposicional local sin adjunción decidida

Fecha: 2026-10-22. Estado: diseño y crítica documental fuente-only. No implementado ni probado. Propietario de esta revisión: agente de diseño/criticidad, misma familia del coordinador; no es revisión humana ni independencia de familia.

## Dictamen

Un enlace tipado `en(anchor)` con categoría semántica indeterminada es un mínimo no vacuo. Su adjunción puede permanecer como variable local, sin elegir lugar, modo o ajuste, y sin desplegar mundos completos. Pero **una variable libre dentro de la cláusula solo es fiel si su dominio y todas sus restricciones conservan exactamente los sitios que permite la fuente**. `host_claim + en + anchor + unknown_site` por sí solo es una representación más débil: retiene la existencia del complemento y pierde restricciones de adjunción. Declarar `complete` no repara esa pérdida.

El contrato propuesto abajo es viable para propietarios determinados y, condicionalmente, para un dominio local exacto ya justificado. No queda demostrado un dominio exacto para los tres textos autorizados. Por ello persiste un gap real para conceder completitud documental de la adjunción ambigua en reserva y Zinc. Esto no demuestra que la fuente sea intrínsecamente imposible de representar: identifica la obligación pendiente. No cambia ninguna puerta, original o dictamen anterior.

## Alcance y evidencia

Se leyeron `AGENTS.md`, las reglas backend y testing, los índices documentales de memoria y decisiones, [el análisis previo](documentary-v3-nacar-context-minimality-2026-10-22.md) y exclusivamente estos tres inputs propios de `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-author-corrections-v1/known116-bundle/`:

| ID | Fuente | Estado observado del complemento |
| --- | --- | --- |
| `d-38fb24e622a073a293ead090` | Cobre funcionó ochenta minutos en Nacar sin apagarse; no se midió su duración hasta apagarse. | `en Nacar` bytes UTF-8 `[32,40)`, ancla `[35,40)`, coverage unresolved. `c1` porta operación, `run_duration` y `no_shutdown`; `c2` es medición negada separada. |
| `d-b7123caa690427c7c04d4c74` | Se reservó una batería nueva para Cobre en Nacar. | Complemento `[42,50)`, ancla `[45,50)`, coverage unresolved. `c1` tiene `action_beneficiary(subject=battery, beneficiary=cobre)`. `nueva` permanece unsupported. |
| `d-5d8e6d45d1ee52311cdfb8b4` | El reproductor Zinc se apagó tras cien minutos en Nacar. | Complemento `[48,56)`, ancla `[51,56)`, coverage unresolved. `shutdown` contiene `shutdown_duration`: cantidad exacta 100 minutos, evidencia `tras cien minutos`. |

En todos, `nacar` existe sin links y con `semantic_type: null`. Eso preserva la mención, no su enlace local. Los vocabularios propios ofrecen configuración, localización y descripciones, pero estos raws no demuestran un carrier neutral equivalente con owner de argumento/intervalo y adjunción indeterminada. No se inspeccionaron contrapartes, preguntas, gold, V6, utility, otros raws, código o APIs. No se ejecutaron Cargo, binarios ni campañas. Este informe es el único archivo escrito por este agente.

## Qué puede quedar indeterminado

La categoría semántica de `en` puede ser `undetermined` sin enumerar lugar/modo/ajuste: esas no son necesariamente alternativas exhaustivas. Es una relación preposicional identificada cuyo significado fino no se decide; no un predicado libre que el consumidor pueda interpretar a voluntad. El referente de Nacar, su scope y su uso como complemento sí deben quedar enlazados.

La adjunción es otra dimensión. Reserva en Nacar y beneficiario Cobre en Nacar no tienen el mismo owner. Apagado en Nacar e intervalo en Nacar tampoco. Apuntar al ID global `cobre` o `zinc` inventa una propiedad transportable a otras ocurrencias. Apuntar siempre al claim cambia una interpretación nominal o temporal por una de evento.

Una declaración fuente puede estar íntegra aun dejando una variable sin asignación, cuando esa indeterminación pertenece a la fuente y su estructura queda íntegra. Completitud de declaración no exige verdad resuelta ni extracción de una respuesta. Sin embargo, la libertad de la variable debe ser exactamente la de la fuente. No se convierte una extracción deficiente en indeterminación original cambiando el motivo de coverage.

## Contrato prospectivo acotado

Nombre de perfil experimental: `local_prepositional_attachment_v1`, solo evaluación. Campos desconocidos o duplicados se rechazan. El perfil no se activa por encontrar `en`, `Nacar` o una key de configuración. Su ausencia conserva el contrato histórico.

Un documento que lo declare podría añadir `prepositional_relations`, colección de registros con los siguientes campos obligatorios y cerrados:

| Campo | Forma y significado |
| --- | --- |
| `id` | ID local único, namespace propio del documento. |
| `scope_id` | Scope existente de la relación, compatible con host y ancla. |
| `host_claim_id` | Claim existente que delimita la cláusula contenedora; no es automáticamente el owner. |
| `marker` | Enum cerrado `en`; no texto libre. |
| `semantic_category` | Enum de esta versión con único valor `undetermined`. No equivale a location/mode/configuration. |
| `anchor_ref_id` | Referencia nominal local existente y con evidencia propia. Sin identidad por nombre desnudo. |
| `attachment` | Una de las formas descritas abajo; no null comodín. |
| `marker_evidence` | Span UTF-8 exacto del token preposicional. |
| `complement_evidence` | Span exacto de `en` más el complemento nominal. |
| `clause_evidence` | Span de la cláusula local contenedora. Delimita evidencia, no ejecuta semántica. |

`attachment` tiene tres formas excluyentes:

1. `fixed`, con `site` obligatorio: una adjunción decidida por la fuente y revisada como tal.
2. `local_variable`, con `variable_id` local y `domain` obligatorio: relación operativa cuyo owner no se decide. `domain` contiene un conjunto no vacío de `sites` y un estado `exact` o `open`. `exact` significa que esos sitios, sin otros, constituyen el dominio semántico; no una lista de candidatos convenientes. No lleva asignación ni se resuelve por matching favorable.
3. No existe forma `unknown_site` sin dominio. Si no pueden expresarse los límites de la adjunción, se conserva una incidencia pendiente fuera del carrier completo; no se fabrica un owner.

`site` es una unión cerrada de selectores locales:

- `claim_occurrence { claim_id }`: la ocurrencia expresada por el claim, sin afirmar nuevas ocurrencias.
- `argument_use { claim_id, role }`: el uso singular de un argumento `ref` de ese átomo. El perfil requiere roles únicos; si se admitieran roles repetidos, necesitaría un selector de ocurrencia estable y sería otro cambio. Se valida que el rol existe y su valor completo se conserva. No acepta el ID de una entidad global como sustituto.
- `qualifier_interval { claim_id, qualifier_id }`: intervalo explícito ya adjunto al claim, de dimensión duración temporal; se preservan operador, cantidad, unidad y condición terminal del qualifier. No admite cualquier número, qualifier compartido sin incidencia local ni intervalo inventado. No convierte los 100 minutos en nuevo evento medido.

Todos los sitios de un dominio pertenecen a `host_claim_id` y su cláusula, al mismo entorno de scope/binder/rama. No se admite apuntar al host de otra cláusula ni a cualquier antecedente con el mismo nombre. No hay campos de predicados libres, quotes como contenido sustituto, categoría inferida o propiedad permanente. Una restricción de adjunción adicional que la fuente exprese y esta forma no pueda codificar deja el registro incompleto: no se omite para que el conjunto parezca exacto.

Para Cobre, el owner de operación `c1` permite estudiar la forma fixed sin transferir el PP a `c2`; esto sigue necesitando crítica de esa adjunción y no cierra el documento entero. Para reserva, la distinción local relevante es claim frente al uso `beneficiary`; para Zinc, claim frente a la incidencia `shutdown_duration`. Esos contrastes muestran por qué existen los selectores, sin convertirlos en inventarios exhaustivos de lecturas.

Los estados `exact/open` son declaraciones semánticas sujetas a crítica, nunca certificados de fidelidad. El validador puede comprobar forma y referencias; no puede demostrar desde ese flag que la lista coincide con el lenguaje natural. Para los tres textos no se propone escribir listas exactas en los raws ni cambiar sus coverage. El diseño permite listar candidatos con estado open para diagnóstico, pero esa lista no representa exhaustivamente la fuente ni concede complete.

## Crítica del carrier de cláusula con variable libre

Sea `D_source` el conjunto de sitios locales realmente permitido por la fuente, incluidas sus restricciones. La declaración abstracta es `EN(x, nacar), x ∈ D_source`; `x` representa el owner aún indeterminado, no una ubicación física desconocida.

Si se reemplaza por `x ∈ all_local_sites(host)`, el carrier permite adjunciones extra siempre que ese conjunto sea mayor. Si se reemplaza por los sitios que el autor alcanzó a enumerar, puede excluir adjunciones legítimas. Si se omite el dominio, ni siquiera conserva la diferencia entre cláusulas, roles e intervalo. Solo hay equivalencia cuando se demuestra igualdad de dominios y se conservan los demás enlaces y ámbitos.

La prueba puede ser intensional en un futuro: un bosque sintáctico local tipado y sus restricciones podrían denotar exactamente `D_source`, sin enumerar interpretaciones del documento entero. Pero almacenar la cláusula original y decir «aplicar la gramática correcta» difiere el trabajo al lector; no es una relación computable preservada. Tampoco basta declarar `all_grammatically_possible` sin especificar y validar esa gramática y el análisis local. Esta versión no introduce un parser/forest ni afirma que los owners de los raws existentes sean ese análisis.

Con estas fuentes, `{reserva, beneficiario}` y `{apagado, intervalo}` son hipótesis útiles tomadas del análisis anterior, no demostraciones de dominio exacto. Un universo de tipos que admite además usos de actor/sujeto no autoriza añadirlos al dominio de esta fuente. A la inversa, no se puede omitir un sitio permitido porque falte un selector cómodo. La nueva unión de tipos resuelve direccionamiento; no demuestra exhaustividad lingüística.

La opción de key neutral en `Configuration` solo sería equivalente si conservara la misma relación, selector de incidencia, variable y dominio exacto, con igual efecto en todos los consumidores. Los inputs y el análisis anterior no prueban esa equivalencia. Una key en el claim resuelve únicamente la forma fixed de owner claim; no cubre por sí sola las otras formas. Por eso no se recomienda reutilizar ese contenedor como si el nombre de la key eliminara el problema de owner.

## Validación, grafo y consumo necesarios

La validación prospectiva exige evidencia exacta, unicidad de IDs, perfil habilitado, referencias existentes, marker contenido en el complemento, complemento y sitios pertenecientes a la cláusula y scopes compatibles. En Zinc, la evidencia actual de `shutdown` termina antes del complemento; un registro nuevo debe aportar la cláusula contenedora completa sin reescribir la evidencia histórica ni asumir que el span previo ya la incluye. El ancla no recibe tipo ontológico por capitalización. Un fixed no puede declarar indeterminación de adjunción en coverage como si ya estuviera decidida.

En el grafo, host y sus sitios relevantes requieren el registro relacional para consumirse como declaración completa. El registro requiere el ancla, scope y todos los operandos de sus selectores; un selector de intervalo requiere la incidencia y el qualifier completos. Esas referencias implícitas participan en detección de ciclos y deudas. No se crea un ciclo artificial al duplicar host→relación→host: la incidencia del selector identifica su sitio y se valida por separado; los requisitos semánticos se recorren mediante claves tipadas con detección de ciclos reales. Si se admite una serialización de dependencias explícitas redundantes, se compara su consistencia y se recorre una sola arista efectiva.

Un consumidor debe transportar el carrier entero o declarar qué proyección preserva sus restricciones. Una solicitud de la misma relación preposicional neutral solo puede satisfacerse por una correspondencia local demostrada. Si exige owner fixed, una variable con varios sitios no prueba que sea ese owner. Que uno de sus candidatos coincida nunca basta. Dos variables no se igualan solo por compartir el host o un elemento: se comparan dominios y restricciones mediante un mapa de ocurrencias válido. Un dominio open bloquea una derivación completa que dependa de cerrar esa adjunción; no se omiten las alternativas aún desconocidas.

Pedir lugar, modo, ajuste o propiedad global no se satisface por `semantic_category=undetermined`. El resultado compatible de un carrier neutral puede ser únicamente contexto tipado con su incertidumbre íntegra; no promueve por sí solo bindings, términos de respuesta, cantidades ni hechos específicos. No hay fallback a ruta histórica que descarte el registro, lectura semántica de la cita o coacción por nombre. Los hechos realmente independientes se conservan; no se borra globalmente todo el documento por una deuda local.

La firma canónica incluye namespace semántico, scope/binders/ramas, predicado/ocurrencia del host, marker, categoría, referente ancla canónico, forma de attachment y selector completo o conjunto de sitios con su estado. Usa renombrado alfa consistente; ordenar conjuntos no cambia la firma. No colapsa argumento con claim, ni intervalo con claim, ni open con exact. Evidencias conservan su propia identidad de origen y se validan aunque no se usen sus bytes como sustituto del contenido semántico. Los IDs locales desnudos no son la firma.

Todas las expansiones de alias, owners, dominios y recorridos de dependencias cargan un presupuesto compartido por operación, también en caminos históricos alcanzados desde el carrier. Se cuenta por llave tipada más entorno de correspondencia; no se reinicia por sitio, hoja ni solicitud. El agotamiento conserva payload y deudas y bloquea la conclusión completa; nunca recorta favorablemente el dominio. Se fijarán límites medidos antes de implementación; este documento no inventa una cifra ni promete complejidad o latencia comprobada.

## Controles prospectivos que harían falsable el diseño

No se ejecutaron. Los controles semánticos deben partir de documentos plenamente válidos; un rechazo de schema no acredita protección de matching.

| Control | Resultado necesario |
| --- | --- |
| Retirar relación conservando nombre, quotes y claim | Se pierde preservación del complemento; no complete ni reconstrucción desde texto. |
| Sustituir solo el ancla por otro referente en control coherente de fuente/evidencia | Cambian restricción y firma; no consume como Nacar. Renombrar IDs del mismo referente mantiene resultado. |
| Mover owner entre claim/beneficiary o claim/interval | Cambia firma y consumo; no produce la misma restricción. Otra ocurrencia de Cobre no hereda nada. |
| Exigir ubicación, modo o configuración | Neutral no prueba esa categoría; ni alias léxico ni fallback lo rescatan. |
| Variable con dos sitios, pedido fixed coincidente con uno | No conclusión fixed por coincidencia existencial. Conservar ambos sitios y la variable. |
| Añadir sitio indebido / quitar sitio permitido | Crítica de fidelidad detecta sobre/subrepresentación; validación estructural sola no concede equivalencia. |
| Open con lista aparentemente suficiente / quitar una alternativa | Sin cierre favorable; no universalizar sobre el prefijo recibido. |
| Borrar referencia, evidencia, incidencia o scope | Rechazo estructural o deuda explícita; null nunca comodín. |
| Relación en otra cláusula, rol o intervalo numéricamente igual | No transferencia por nombre/cantidad. `c2` de Cobre no hereda `en Nacar`. |
| Renombrar IDs y reordenar conjuntos/argumentos únicos | Firma y semántica invariantes mediante mapa completo. |
| Ciclo, alias con deuda, límite global agotado | Rechazo/deuda conservada, sin conclusión completa y sin eliminar hechos independientes. |
| Perfil ausente y consumidores históricos | Comportamiento histórico idéntico; perfil presente no puede ignorarse por esa ruta. |

En las fuentes propias se conservarían además las fronteras ya expresas: operación sin apagado no es duración hasta apagado ni máximo; reserva no es uso; apagado tras intervalo no es medición. `nueva` y la estructura de `hasta apagarse` conservan sus obligaciones propias. Resolver un PP no repara otros elementos por asociación.

## Decisión propuesta

Mantener la puerta histórica y los tres dictámenes intactos. Aceptar el enlace neutral como objetivo de diseño, con propietarios locales tipados y consumo obligatorio. No aprobar todavía la promesa de que `host + variable` baste para complete en las tres fuentes. Antes de implementar la forma local_variable hace falta una crítica fuente-only que establezca el dominio exacto y sus restricciones, o un mecanismo tipado de dominio intensional con equivalencia comprobable. Si no se obtiene esa prueba, queda adjunción abierta y fidelidad completa pendiente. Ningún flag, cobertura textual o inventario parcial sustituye esa obligación.
