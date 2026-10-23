# Revisión estática intermedia de proyección existencial

Fecha: 2026-10-22. Estado: código en escritura, pendiente de freeze y ejecución por root. Revisión exclusivamente de lectura; no se ejecutaron Cargo, fmt, CLI, proveedores ni los controles/replays preparados. Este informe no aprueba una versión final.

Archivos revisados: `tools/evaluation/src/documentary_v3_io/existential_projection.rs`, integración de `readings.rs`, perfil contextual y presupuesto compartido. Contrato: [diseño con addendum normativo](documentary-v3-existential-projection-design-2026-10-22.md). No se consultaron etiquetas, contrapartes del banco, particiones reservadas ni otros dictámenes.

Snapshot intermedio observado:

| Archivo | SHA-256 |
| --- | --- |
| existential_projection.rs | fc8285ad15961dbd5b81f68a8dfcf435facc415474d2f18ea1a93382ba1663b5 |
| readings.rs | dc5033214f6f0928ce44eca9b4f54fac059f685dac7a5262e6a888731ebb92bd |

## Hallazgos que requieren corrección

### SR01 — Anclas externas comparadas sin su identidad acreditada

En el snapshot, `Graph::reference` obtiene la clave histórica y la guarda en NodeOrigin, pero `named_key` usa solamente `descriptor.to_string()`. La equivalencia entre candidatos compara `request` y `body`; no comprueba que las claves acreditadas de las anclas externas correspondientes coincidan. El consumidor reconstruye cada candidato y obtiene el mismo criterio incompleto.

Contraejemplo estático: dos hojas conservan un único nombre literal idéntico y la misma clase, pero seleccionan ocurrencias distintas de ese nombre dentro del mismo texto. La clave histórica incluye las anclas de evidencia y puede diferir; el descriptor nominal conserva solo el literal. La proyección puede considerar común un owner que la identidad histórica no acredita como el mismo. Un binding fuente→pregunta acreditado dentro de cada celda no sustituye la obligación de fijar anclas externas entre las proyecciones comparadas.

La corrección debe mantener la clave acreditada en la correspondencia de anclas usada para agrupar y consumir candidatos, junto con el descriptor íntegro y todos los enlaces. No debe introducir una identidad para la variable ni para el descriptor nominal. Un control con nombres iguales y anclas distintas debe demostrar el límite; no se afirma que ya se haya ejecutado.

### SR02 — RecordOrigin puede señalar una arista distinta de la consumida

La rematerialización de orígenes busca la primera arista por etiquetas parciales. Para dependencias compara kind, role, origin y consumidor, pero omite el destino; para reference_link compara únicamente kind/link_kind; para identity únicamente kind/identity_kind. Dos enlaces del mismo tipo a destinos diferentes pueden terminar apuntando al mismo edge_id. `consume` reconstruye ese mismo origen defectuoso y por eso su igualdad no detecta el error.

El mapa debe resolver la arista normalizada exacta por dirección y ambos extremos, además de todas las etiquetas y procedencia. Si una arista no existe debe fallar invalid_origin. El literal `same_component` no es un edge_id de ConclusionGraph: una fila Same que se representa por el cociente debe apuntar explícitamente al nodo/componente competente mediante un RecordOrigin compatible, sin inventar una arista.

Se requieren controles de origen cruzado con dos links del mismo kind y destinos distintos y dos dependencias con el mismo consumidor/rol y distinto destino. La corrección del body no basta si el certificado enlaza otra arista.

## Correspondencias estáticas favorables y sus límites

El feature está en el conjunto cerrado contextual con máximo siete. `upgrade` retorna antes de recorrer si no está permitido. La integración toma la matriz original y la restaura; la nueva salida V2 añade discriminante y arrays. El consumidor recorre exactamente una prueba por celda, rechaza índices repetidos y vuelve a construir candidato, body y origin map desde los documentos de la misma operación. Eso es una puerta de consumo concreta para EP19, pendiente de las correcciones de ancla/origen y de ejecución.

El componente de respuesta se establece antes de normalizar el átomo y todas sus incidencias resuelven mediante el mismo mapa. Solo ese componente genera bound_variable; un segundo anónimo exige carrier nominal competente o queda not_supported. El descriptor nominal mantiene owner_node_id y no posee binder físico independiente. La conclusión fija identity_proven=false. Estas decisiones corresponden al contrato de correlación y ausencia de identidad global; su corrección observada sigue pendiente de controles ejecutados.

Los enlaces se recorren sobre todos los miembros Same y se procesan antes de dependencias referenciales, de modo que el target nominal ya tiene carrier. Las dependencias explícitas preservan role enum V3 y origin=explicit; los operandos de los argumentos generan implicit_operand/implicit sin sustituir las explícitas. Una dependencia hacia otro claim/qualifier/request no se ignora: produce not_supported. Qualifiers, context bindings, premisas y otros carriers no competentes se rechazan de forma conservadora en este primer subset; esa ausencia de capacidad no cuenta como éxito semántico.

El límite de operación es compartido y su agotamiento se comprueba nuevamente al salir de with_operation, evitando publicar como válido un resultado parcial construido antes del agotamiento. Hay cargos de visitas, copias y construcción de body/origins. No se ha medido coste local ni demostrado un límite real de memoria: existen ordenaciones y serializaciones auxiliares cuyo coste no se identifica con una visita de grafo. Se debe describir el contador como presupuesto operativo, sin equipararlo a bytes máximos residentes o latencia medida.

## Estado de entrega

Los seis controles y los replays anunciados por root no fueron ejecutados por este revisor. El positivo obligatorio sigue siendo Senda con misma fuente/pregunta y expectativa, modificando únicamente el feature optativo y los hashes consecuentes; un rechazo universal o un booleano verde sin conclusión consumible no lo satisface. Tras las correcciones y el freeze deberá vincularse la revisión a hashes nuevos y registrar por separado compilación, controles observables y replays. Hasta entonces SR01 y SR02 son hallazgos estáticos intermedios pendientes de reconciliar.


## Reconciliación estática del snapshot congelado por root

Root confirmó freeze de implementación y fmt pasado; la prueba focal seguía en curso al solicitar esta segunda revisión. No se atribuyen esos comandos a este revisor ni se anticipan sus resultados.

| Archivo | SHA-256 del segundo snapshot |
| --- | --- |
| existential_projection.rs | fb0f9fb99496c4b5551c0ccd254a6cfadff0ff16240465dbd8a82c90a32e1687 |
| readings.rs | dc5033214f6f0928ce44eca9b4f54fac059f685dac7a5262e6a888731ebb92bd |
| request_context_profile.rs | 0d6e3a4fac37cb1fa4a688f085602cbcad06a318320c1b7f4a7e19b39116a51a |

**SR01 cerrado estáticamente.** La asignación de named nodes incluye lado, descriptor y clave histórica. La fusión fuente/pregunta necesita un binding Same acreditado y descriptor compatible. La comparación de candidatos incluye `anchor_keys`, y el consumidor vuelve a reconstruir candidatos y exige igualdad de esas claves en todos los OriginMap. La clave sigue siendo prueba de ancla nombrada, no sustituto del descriptor ni identidad de la variable anónima.

**SR02 cerrado estáticamente.** El origen de links comprueba propietario y target; dependencias comprueban consumidor, target, role y origin explícito; identidad comprueba sus dos extremos. Un origen sin arista competente produce invalid_origin. Ya no existe fallback a una dirección ficticia `same_component`.

El código materializa una arista Same reflexiva por nodo del cociente local. Esa arista expresa x=x y no una igualdad entre testigos de lecturas distintas. Una fila Same original se asigna a ella solo después de que ambos extremos resuelvan al mismo componente; NodeOrigin conserva todos sus miembros y el pointer mantiene la evidencia raw. Es una representación válida del axioma de igualdad local, independiente del número de aliases, y no una nueva clave global. Su deduplicación evita que aliases redundantes creen una falsa diferencia de contenido.

**Puerta de vocabulario y versión.** `enabled` exige un perfil contextual válido y el feature permitido, no una coincidencia textual. No revalida por sí solo todos los campos del vocabulario, pero los validadores previos preservan sus errores y la elegibilidad resultante impide construir conclusiones ante vocabulario global inválido. La envoltura puede ser V2 con estado ineligible y errores originales conservados. Esto no convierte un error en respuesta o negativo fiable; imponer además V1 para todo vocabulario inválido sería otra decisión de formato, no una reparación necesaria de la solidez identificada aquí.

**Dictamen del segundo snapshot.** No se identifica una nueva falsedad material ni laguna de procedencia que impida continuar con las verificaciones autorizadas del subset. El consumidor reconstruye body/request/origin y claves por cada celda; la variable repetida conserva su nodo y el canal completo V2 tiene discriminante y conclusión inspeccionable. Sigue pendiente confirmar resultados de pruebas/controles/replays, así como consumo observado de ese canal. La revisión estática no se presenta como prueba ejecutada de EP19 ni como validación de recursos, extracción o adopción.


## Tercer snapshot: payload íntegro, descriptor léxico y ordenaciones acotadas

Root comunicó freeze posterior a fmt y cinco pruebas focales pasadas; build seguía en curso. Este revisor no ejecutó esas verificaciones. Hash observado de existential_projection.rs: `9848f0e0813edea2c1b81eade5198faf21ae26fa6f7e5b0431bcb5a34c6f6392`; readings.rs y request_context_profile.rs conservaron los hashes del segundo snapshot.

El canal V2 incorpora textos y raws fuente/pregunta originales mediante copias con cargo previo. Los mapas de prueba tienen así un soporte íntegro transportado en el mismo reporte. El término nominal local se rechaza como argumento físico y como objetivo de Different; su tratamiento no se convierte en una variable física. El consumidor conserva la comparación de anclas entre todos los mapas. Las ordenaciones reservan comparaciones y precomputan una clave por valor, evitando serialización en cada llamada al comparator. No se deduce un límite de memoria residente de esos contadores.

La prueba focal de correlación usa ahora documentos fuente/pregunta originales validados y comprueba directamente el primitivo de normalización: una referencia repetida conserva nodo y otra referencia competente no se colapsa. No afirma probar una solicitud end-to-end con dos slots omitidos. Esta distinción es coherente con el alcance real de la prueba revisada.

### SR03 — Un cap local devuelve limit sin marcar agotamiento global

En este snapshot `comparisons(count)` retorna Err(limit) cuando count>4096 sin activar el flag shared-budget exhausted. La profundidad `visiting.len()>64` usa el mismo patrón. `upgrade` captura fallos por witness como diagnósticos y continúa examinando candidatos. Por tanto, un cap local puede descartar un candidato mientras otro completa, sin que with_operation transforme el resultado en el sobre global context_limit_v1. Es una discrepancia concreta con el requisito de no conservar un subconjunto favorable tras un límite de operación.

La corrección mínima es hacer que la superación de cada cap marque agotamiento del presupuesto compartido antes de retornar limit, o propagar ese estado hasta el error global. No debe añadirse un nuevo fallback ni cambiar expectativas. La rama global debe conservar los inputs íntegros y reportar ineligible/requires_review, nunca éxito semántico. Este hallazgo es estático; no se ejecutó un contraejemplo en CLI ni se afirma que los seis controles existentes cubran ese cap.


### Reconciliación final de SR03

Root centralizó las salidas limit mediante `exhaust()`: marca agotamiento compartido con charge(usize::MAX,0) antes de devolver limit. La lectura final confirma su uso en todos los retornos limit del módulo, incluidos cap de ordenación, profundidad y fallo de contabilización de bytes. De ese modo el wrapper de operación convierte incluso un error capturado localmente en request_context_limit; no puede persistir un candidato favorable como salida ordinaria válida.

Hash final revisado de existential_projection.rs: `12c1062a977be7e24dcec225cbcefcba39e3a875f959ce1a0446b78b7c7491eb`. Los hashes de readings.rs y request_context_profile.rs permanecen `dc5033214f6f0928ce44eca9b4f54fac059f685dac7a5262e6a888731ebb92bd` y `0d6e3a4fac37cb1fa4a688f085602cbcad06a318320c1b7f4a7e19b39116a51a` respectivamente.

La nueva prueba operation_cap_tests::local_sort_cap_exhausts_entire_operation verifica explícitamente cap4097, flag exhausted, rechazo de charge posterior y error global de with_operation. Se inspeccionó su contenido; no se ejecutó por este revisor ni se atribuye un resultado antes de que root lo confirme. SR03 queda cerrado estáticamente. SR01/SR02 siguen cerrados; no queda un fallo material identificado en el alcance revisado. El dictamen es de código y contrato en estos hashes, separado de compilación, pruebas, controles CLI, replays y medición de coste que root debe registrar con su evidencia efectiva.
