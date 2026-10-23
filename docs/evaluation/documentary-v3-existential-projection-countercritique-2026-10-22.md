# Contracrítica de la proyección existencial común

Fecha: 2026-10-22. Alcance: revisión de diseño y lectura estática; sin cambios Rust/Cargo, ejecución de pruebas, llamadas a proveedores ni adopción. Documento contrastado: [diseño de proyección](documentary-v3-existential-projection-design-2026-10-22.md), incluidas sus precisiones de cierre dirigido, componentes Same, descriptor nominal local, correlación interna y exclusión inicial de patrones anónimos/descripciones. Se revisaron `AGENTS.md`, reglas backend/testing, contrato documental local, límite nominal conservado y `tools/evaluation/src/documentary_v3_io/readings.rs`. No se consultaron contrapartes del benchmark, etiquetas, particiones reservadas ni dictámenes de otras revisiones.

## Dictamen

La introducción propuesta es lógicamente defendible: cada cruce puede tener su propio testigo del mismo cuerpo existencial sin acreditar identidad entre testigos. No encontré una falsedad lógica reproducida en el mecanismo nuevo porque todavía no existe implementación. Las objeciones siguientes son obligaciones de contrato y controles prospectivos, no fallos ejecutados.

No debe aprobarse una implementación por devolver únicamente contexto, por abstenerse en todos los casos o por sustituir la pregunta. El objetivo mínimo sigue siendo publicar una respuesta existencial estructurada y útil para el control Senda conservado, con el mismo soporte y las mismas dos lecturas, sin promover el tamaño a propiedad común. El diseño satisface esa dirección; el cierre del formato ejecutable y su uso por consumidores sigue pendiente.

## Lo que el diseño sí resuelve

`forall reading exists witness: Body(witness)` permite responder `exists variable: Body(variable)` bajo las lecturas representadas. No permite concluir que hay un individuo identificado que es el testigo de todas ellas. El diseño conserva esta diferencia y no exige igualdad accidental del inventario completo de witnesses: basta una proyección entera común con prueba completa en cada cruce.

El cierre dirigido distingue soporte de consumidores entrantes. En Senda, `possession → tray → paper` incluye el complemento nominal. `local_descriptor → tray/paper` no convierte por sí solo ese claim en premisa de posesión. Si posesión requiere el descriptor, directa o efectivamente, ya no es residual. Esta precisión es necesaria para evitar tanto la pérdida actual de respuesta como una relajación ilegítima.

El carrier `local_nominal_lexical_use` también es necesario. «Existe una bandeja de papel» no permite introducir una segunda variable física y afirmar que existe un lote/hoja de papel independiente. La conclusión debe conservar la construcción nominal bajo su propietario y el subtipo físico indeterminado.

## 1. Falta cerrar el portador de contenido, prueba y completitud

El sobre propuesto contiene metavariables para `body`, `request` y pruebas. Eso es una especificación de diseño legítima, pero aún no es un contrato de salida implementable sin decisiones semánticas adicionales. Reutilizar enums existentes no define por sí solo qué campos aceptan `bound_variable`, cómo se identifica su binder ni cómo se distingue un descriptor léxico de una referencia física.

Antes de habilitar el feature debe existir un esquema cerrado que fije al menos:

- Identidad local de conclusión y binder; alcance exacto; variable de respuesta; tipo y denotación de cada variable. Un uso fuera de ese alcance debe ser inválido aunque coincida la cadena `v0`.
- Lugares permitidos para `bound_variable`: todos los operandos correlacionados autorizados, nunca IDs de referencias, nombres, evidencia o cadenas opacas que se sustituyan por búsqueda textual.
- Cuerpo íntegro con átomo, juicio/estado/fundamento, roles, clases/tipos, scopes/contextos, qualifiers, restricciones, aristas y ausencias. El cuerpo debe poder inspeccionarse sin invertir un hash.
- Descriptor nominal local con propietario, tipo de enlace, incidencia y descriptor completo. Ningún `exists` propio ni conversión silenciosa a `ref` para ese descriptor.
- Prueba por celda con los dos IDs de lectura, hashes/textos originales, request/claim/witness exactos, componentes locales, mapa de orígenes e incidencias y resultado local. Su asociación a `request_id` y `conclusion_id` debe verificarse, no inferirse de posiciones de array.
- Residual por celda distinguido del cuerpo protegido y conservado con su procedencia. Un error de mapa no puede relegarse al residual.
- Versionado y discriminante obligatorio de la respuesta. `complete_answer=true` sin un tipo de conclusión reconocido por el consumidor no constituye una respuesta consumible.

El borrador deja opcional la decisión de `completion_kind=identity_answer` y el versionado final. Se acepta como asunto pendiente de diseño, no como garantía de compatibilidad. Una implementación debe elegir una forma concreta y probar un consumidor que presenta la conclusión; solo añadir un booleano verde con listas históricas vacías no cierra Senda.

## 2. La correlación debe sobrevivir dentro de un solo átomo

Separar binders entre conclusiones evita joins ilícitos, pero no cubre por sí solo la correlación interna. Para un predicado sintético registrado, considérese `relates(owner=A, left=x, right=x)` frente a `relates(owner=A, left=x, right=y)`, con `x Different y` en el segundo. Cada objeto puede tener la misma clase y descripción. No son el mismo cuerpo bajo alpha-renombrado. Sustituir cada aparición o rol de forma independiente destruye la igualdad interna.

El mapa debe construirse primero sobre componentes Same locales; después se aplica una única sustitución coherente a todas las incidencias. No se exige biyección entre raw IDs: dos aliases compatibles de un componente pueden corresponder a uno del otro. Sí se exige que el cociente conserve la partición de ocurrencias y las restricciones Different. Un ciclo o contradicción en ese componente bloquea, aunque la referencia elegida como raíz no contenga el problema directamente.

En el subset actual los demás roles referenciales deben tener correspondencia acreditada. El segundo átomo con `y` anónimo puede quedar fuera del subset; ese rechazo es correcto, pero debe comprobarse por la obligación de correspondencia y no porque el fixture sea estructuralmente inválido. Debe haber también un positivo con repetición permitida del mismo componente y un negativo que sustituya esa segunda incidencia por un referente nombrado distinto. Así se comprueba correlación dentro de una forma que sí puede llegar al constructor.

La realización debe usar el mismo cuerpo: una variable repetida sigue siendo «ese mismo ítem» dentro de la conclusión; dos conclusiones independientes no pueden realizarse como «un ítem que reúne ambas propiedades». Una superficie textual que suprima esa correlación puede ser falsa aunque el JSON sea correcto. En Senda, una realización fiel es «Senda tiene al menos una bandeja de papel» con tamaño abierto. No debe decir «la bandeja grande», «el papel es grande» ni identificar una bandeja entre lecturas.

## 3. Residual no significa todo lo que no se haya seleccionado

Un claim no seleccionado puede ser irrelevante, una restricción efectiva o una contradicción relevante. La clasificación debe distinguirlos antes de formar el residual. El cierre no incorpora todos los consumidores entrantes, pero la búsqueda de conflictos no puede limitarse a dependencias salientes.

Contraejemplo: el soporte positivo `has_item(Senda,x)` y su negación explícita sobre el mismo componente/contexto. Si la negación no es dependencia de posesión, un cierre exclusivamente saliente no la descubrirá. Debe bloquearla la comprobación de conflicto relevante ya exigida por el diseño. En cambio, `has_item(OtraPersona,y)` negado en otro contexto no debe vetar Senda.

Una diferencia en metadata de tamaño es residual solo mientras no sea filtro, restricción del término, guard, contexto ni dependencia efectiva del soporte. Clasificar por el nombre del atributo o por coincidencia de cita sería insuficiente. El caso positivo y su mutación a dependencia requieren el mismo descriptor y la misma evidencia, cambiando únicamente la relación semántica que lo consume.

## 4. El punto de integración posterior no cubre todas las puertas anteriores

`readings.rs` calcula elegibilidad y estabilidad del inventario antes de comparar respuestas. Con perfil contextual, la rama de inventario usa `context_owner_key` para cada request; este canonicaliza contenido referencial a través de la identidad histórica. Un `class_filter=reference_pattern` anónimo puede devolver `None` allí, antes de que un módulo posterior construya conclusiones existenciales. Es un riesgo estático concreto, no una ejecución que demuestre un fallo nuevo.

El control Senda conservado usa `class_filter:null` y owner nombrado; no exige aflojar esa puerta para cerrar su objetivo. La revisión del diseño excluye expresamente `reference_pattern` y `described_ref`, conserva filtros nulos/literales comprobados y mantiene la puerta temprana intacta. Con esa precisión la objeción queda resuelta para el primer subset sin retirar Senda. Sigue siendo un control de frontera: no se debe simplemente forzar `eligible=true` tras fallar el inventario ni presentar esta limitación como capacidad general de patrones anónimos.

También `validate_inner` exige cierre completo y ausencia de `unresolved_dimensions` para un envelope completo. La ambigüedad de adjunción del control está representada por dos hojas cerradas; puede mantenerse residual sin cambiar esa condición. No se puede extrapolar el éxito a envelopes semánticamente abiertos. Cualquier excepción futura por dimensión «no relevante» necesitaría su propia prueba de independencia, no solo una etiqueta favorable.

## 5. Existencia, unicidad y múltiples soportes son cuestiones distintas

El diseño acierta al interpretar `one_value` como un testigo suficiente, no como exactamente un individuo. Dos testigos posibles no falsan `exists x: Body(x)` ni autorizan cardinalidad dos. El límite tampoco debe aprobarse por sustituir esa respuesta por «hay un objeto de alguna clase» si el cuerpo completo conserva bandeja y construcción nominal.

Para varias pruebas locales, generar proyecciones enteras y después buscar una clase de equivalencia común es defendible. Buscar primero la mayor intersección de atributos no lo es. El constructor debe conservar el conjunto de candidatos examinados y cada prueba elegida. Si una celda no apoya K, no puede compensarse con dos apoyos de otra celda.

La selección local puede tener reglas de desambiguación más conservadoras que la lógica existencial. Un constructor agregado posterior no debe rescatar un witness que el clasificador dejó incompleto ni alterar matching para pasar el caso. Debe documentarse la limitación si impide un control de varias pruebas; el positivo principal sigue siendo el control nominal con dos hojas localmente completas.

## Controles mínimos de falsación, todavía no ejecutados

Los controles semánticos requieren validación estructural completa de ambos lados cuando se pretende probar matching o agregación. Los controles deliberadamente inválidos se registran aparte. No son casos de extracción ni una muestra ciega.

| ID | Cambio controlado | Observación exigida |
| --- | --- | --- |
| EP01 | Senda conservado, mismo texto/pregunta/expectativa; solo perfil ampliado y hash consecuente | Dos hojas completas y agregado nuevo completo por conclusión existencial; cuerpo bandeja/de/papel; mapas por cruce; tamaño residual por hoja. |
| EP02 | Mismo input sin feature nuevo | Mismo informe histórico por bytes; agregado sigue fallido; cero recorrido nuevo. |
| EP03 | Renombrar IDs, permutar filas sin orden semántico y añadir alias Same compatible | Mismo contenido lógico; orígenes correctos; no exige igual cantidad de raw aliases. |
| EP04 | Cambiar owner/posición de rol, conservando clase y cita del ítem | Ninguna conclusión común ni identidad superficial. |
| EP05 | Un átomo con dos incidencias del mismo componente; mutar una a otra raíz acreditada | Preservar correlación en positivo; no equiparar los cuerpos mutados. |
| EP06 | Different interno, alias con tipo/denotación incompatible o Different a tercero alcanzable | Bloqueo o cuerpo distinto con restricción íntegra; nunca borrar la desigualdad. |
| EP07 | Tamaño residual cambia de bandeja a papel | Conclusión común estable; residual diferente; sin tamaño común nuevo. |
| EP08 | Mismo descriptor pasa a dependencia efectiva o restricción consumida | Ya no es residual; discrepancia bloquea el candidato. |
| EP09 | Negación compatible del mismo soporte sin dependencia explícita; control negativo ajeno | Bloquear conflicto relevante; conservar respuesta ante negación ajena. |
| EP10 | Link nominal ausente, kind invertido/genérico, descriptor diferente o enlace adicional en alias | No publicar cuerpo simplificado; mismo criterio para cada componente. |
| EP11 | Reutilizar descriptor nominal entre propietarios o introducirlo como variable física | Rechazo del carrier; ninguna existencia independiente ni transferencia de propiedades. |
| EP12 | Qualifier con distinto owner, scope, configuración o dependencia implícita | Rechazo de equivalencia aunque texto del qualifier sea igual. |
| EP13 | Request con patrón anónimo de respuesta | Queda fuera del subset y no fuerza elegibilidad; Senda con filtro nulo sigue siendo positivo obligatorio. |
| EP14 | Una celda parcial/sin apoyo/refutatoria o inventario abierto | No completar eliminando la celda; matriz original conservada. |
| EP15 | Dos lecturas de fuente y dos de pregunta | Exactamente cuatro pruebas para K; falta/duplicado de una celda invalida su certificación. |
| EP16 | Cada celda tiene prueba entera de K y witnesses adicionales distintos | K común sin exigir igualdad accidental de todos los soportes; no mezclar trozos. |
| EP17 | Dos conclusiones de claims diferentes con v0 visual idéntico | Binders independientes; sin conjunción o identidad entre sus testigos. |
| EP18 | Corromper mapa, hash, celda, request_id, conclusion_id o binder de origen | Prueba inválida; no basta contenido equivalente sin procedencia correcta. |
| EP19 | Ejecución de consumidor con complete_answer nuevo y canales históricos vacíos | Presenta cuerpo existencial reconocido; versión histórica no lo interpreta como respuesta identificada. |
| EP20 | Límite compartido cerca del umbral con aliases, DAG compartido y residual grande | Cargo acumulado de visitas/copia/serialización; fallo explícito sin subconjunto favorable. |
| EP21 | Request independiente respondido junto a candidato rechazado | Respuesta independiente y procedencia preservadas, salvo agotamiento global de operación. |

## Condición para dar por cerrado el diseño

Puede continuarse el diseño porque no se ha demostrado una imposibilidad ni una regla intrínsecamente falsa. Antes de activar una implementación deben cerrarse el esquema del cuerpo/prueba/binder y el discriminante de completitud; mantener la exclusión explícita de patrones anónimos en el inventario; y conservar la correlación por componentes e incidencias como obligación verificable. Los controles deben demostrar un positivo útil y sus mutaciones, especialmente EP01, EP05, EP07–09 y EP19.

Esta contracrítica no cambia resultados históricos ni acredita que el fallo actual esté resuelto. La capacidad se demostrará únicamente cuando el mismo control produzca una conclusión estructurada inspeccionable, con cada prueba local conservada y sin identidad global anónima. Ninguna de estas comprobaciones, por sí sola, demuestra extracción autónoma, fidelidad general, rendimiento o adopción de producto.


## Revisión del addendum normativo de DTOs

El addendum posterior del diseño sustituye sus opciones abiertas: fija reporte V2 con feature válido, V1 por bytes sin feature, discriminante de completitud, arrays siempre presentes, ConclusionGraph cerrado, tres kinds de nodo, sustitución conjunta de incidencias y un OriginMap por celda con punteros internos comprobados. El residual autoritativo es la matriz/documentos íntegros; no exige duplicarlo. Con ello quedan cerradas las objeciones anteriores sobre el formato ejecutable y la forma del canal EP19 para el subset definido. La comprobación de un consumidor real y el positivo Senda siguen siendo verificaciones de implementación pendientes.

Se detectó una laguna concreta en el primer addendum: la fila `kind=dependency` de ConclusionEdge exige `role=null`, aunque el contrato Dependency V3 posee `role` con valores `identity|guard|configuration|temporal|binding|premise|normative_target|branch_membership` (`validation.rs`, comprobación del enum Dependency). Conservar solo extremos y kind elimina una etiqueta tipada existente y permite igualar grafos cuyo cierre se declaró con roles distintos. No se afirma que esa diferencia baste por sí sola para falsar una respuesta particular; sí invalida la obligación declarada de comparar dependencias completas.

La reparación mínima consiste en conservar ese rol para aristas dependency mediante su enum vigente, reutilizando `role` con unión discriminada por kind o añadiendo un campo exclusivo. Las aristas efectivas implícitas también necesitan una regla tipada explícita para conservar su incidencia sin inventar un rol declarado ausente. Ningún consumidor puede reconstruir la etiqueta por extremos o por nombre del registro. Una mutación con mismos extremos y distinto rol tipado debe conservar esa diferencia o quedar fuera del subset, nunca borrarla al normalizar. Esta corrección no requiere ampliar predicados, tipos de nodo ni alcance.

El protocolo de seis controles preparado por root —Senda positivo conservado, renombrado de IDs, enlace restrictivo adicional, dependencia hacia metadata, envelope abierto y hoja sin apoyo— es un inicio proporcional a la capacidad autorizada. Esta revisión no lo ha ejecutado ni lo convierte en cobertura de los 21 controles prospectivos. El mapa por cada celda y el canal completo deben comprobarse en sus salidas, no deducirse de un booleano final.


La revisión final del addendum corrige esa laguna: `dependency_role` y `dependency_origin` son campos obligatorios de ConclusionEdge. Los explícitos preservan enum V3 y `explicit`; los implícitos usan `implicit_operand` y `implicit`, con operando íntegro normalizado y procedencia. Ambos campos forman parte del contenido semántico comparado; una implícita no sustituye ni deduplica una explícita por compartir extremos. Con esta regla no se colapsan roles explícitos distintos ni se confunde una arista declarada con un operando efectivo. La objeción queda cerrada en diseño. No queda una laguna material identificada que impida implementar este subset; permanece pendiente demostrar su comportamiento y consumo del canal mediante las comprobaciones autorizadas.
