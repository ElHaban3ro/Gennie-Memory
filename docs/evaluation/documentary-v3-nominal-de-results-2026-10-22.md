# Complemento nominal local y preservación de evidencia independiente

El conjunto mixto conocido pasa de **109/116 a 110/116 representaciones completas y fieles para ambos críticos**. Quedan seis. Se implementó una relación nominal optativa y se corrigió una interferencia detectada en los controles. Pasan **784 pruebas**, cero fallos y 24 ignoradas; **216 informes históricos idénticos** y revalidación de los 116 documentos. El objetivo de cinco pasos sigue activo en el paso 3, sin adopción ni apertura de V6. Una nueva prueba de agregación de respuesta anónima permanece fallida.

## Cambio implementado

`nominal_complement_de` conserva la construcción dirigida núcleo→complemento de «bandeja de papel»; una asociación incidental no puede sustituirla. La evidencia debe cubrir exactamente el núcleo, el marcador «de» y el complemento. La comparación usa la descripción del complemento dentro de esa incidencia local y las restricciones de todos los aliases. No crea identidad global de referencias anónimas ni infiere material, uso, ubicación, posesión o una hoja de papel.

El descriptor léxico «grande» sobre el término papel se conserva en el raw completo como metadata local. Se retira de la vista ejecutable ordinaria junto con consumidores explícitos, implícitos y por aliases; no puede satisfacer propiedades universales ni reglas por otra ruta. Una petición local compatible puede conservar contexto parcial con propietario y evidencia íntegros, pero sin cifras, proposiciones de respuesta o bindings inventados. Los hechos independientes conservan su respuesta completa. El matching cuantificado no incorpora este enlace.

La canonicalización recorre el complemento bajo su propietario, sin asignarle una clave global al anónimo. Los recorridos y las copias se cargan al presupuesto acumulado. El feature está limitado al evaluador y su ausencia no activa estos recorridos. Diseño, contrato y revisión estática están enlazados en los documentos de esta tanda.

## Corrección conocida y revisión

Se conserva la misma fuente de Senda y sus dos adjunciones posibles del adjetivo. El delta semántico real sustituye `associated_with` por el nuevo vínculo nominal, con evidencia exacta «bandeja de papel»; no cambia el dueño del descriptor ni añade una resolución física. El candidato anterior y su desacuerdo de críticos permanecen intactos. No se obtuvo aceptación repitiendo el mismo significado hasta encontrar revisores favorables.

Dos críticos nuevos recibieron únicamente vocabulario, guía, fuente y candidato congelados; no vieron preguntas contraparte, gold, código, validaciones o dictámenes anteriores. Ambos dieron faithful en las cuatro dimensiones: **8/8**. Se verificaron hashes, citas UTF-8 y punteros. Son contextos separados de la misma familia, sin revisión humana ni independencia de familia. La aceptación semántica no acredita por sí sola ejecución.

El raw nuevo valida completo. Una consulta manual por el ítem que tiene Senda conserva apoyo completo por separado en ambas lecturas, pero **su agregación sigue siendo condicional**: no hay clave de identidad global para la bandeja anónima. Este control mantiene expectativa de respuesta común completa y **falla**; no se cambió la expectativa ni se inventó identidad para pasarlo. El control usa una petición tipada de desarrollo, no una nueva extracción automática ni la contrapregunta del banco.

| Métrica del conjunto mixto conocido | Antes | Ahora |
| --- | ---: | ---: |
| Estructuralmente completo | 111/116 | 111/116 |
| Fiel para ambos críticos | 109/116 | 110/116 |
| Completo y fiel para ambos | 109/116 | 110/116 |
| Pendientes combinados | 7 | 6 |

Los 115 documentos que no cambian producen informes idénticos. Este conjunto mezcla correcciones dirigidas y vocabularios por documento. **La extracción autónoma original sigue en 99/116 y la utilidad histórica en 31/39, cero ajenos/60**. No se presentan 110/116 ni los controles como rendimiento autónomo de producto.

## Fallos conservados y verificación

La primera suite pasó 781 pruebas. La crítica estática detectó recorridos adicionales bajo perfiles sin el feature y copias grandes sin cargo previo; se corrigieron con permiso explícito de operación y reserva antes de materializar. La segunda pasada añadió el cargo de IDs/contenedores de aliases. Los tests comprueban ausencia de consumo adicional sin permiso, agotamiento compartido e input preservado. La revisión estática no se presenta como reproducción dinámica de una respuesta incorrecta.

Los controles CLI iniciales tenían un atributo `power` ajeno al vocabulario: seis pares quedaron parciales y no se contaron como rechazos seguros. Se preservaron y se reparó únicamente el símbolo a `mass`, sin cambiar el motor o expectativas. Así quedaron nueve de diez controles válidos pasando y un fallo real: metadata ajena añadía unresolved a una petición independiente de Lino y degradaba su respuesta completa. La consulta manual del ítem común también reprodujo esa interferencia en una hoja.

La reparación limita esos pendientes a peticiones pertinentes y no habilita sus consumidores bloqueados. En el binario final pasan **10/10 controles nominales válidos**, incluida conservación de la respuesta completa independiente, mientras que el control del ítem común continúa fallando en agregación. Otros **8/8 controles de unión/adjunción** pasan y se reproducen idénticos con el binario final; las tablas booleanas no prueban exhaustividad lingüística ni calidad del extractor. Total: **18 controles pasan de 19 distintos; uno sigue fallando**. Los replays y reparaciones no se cuentan como casos nuevos adicionales. La aceptación fuente→raw de Senda y la validación estructural se registran por separado de esta puerta de utilidad, que sigue fallando y no habilita adopción.

Suite final: **784 aprobadas, cero fallos, 24 ignoradas**; formato comprobado. Binario SHA-256 `363d8a50c67cac13ffa2da93f95b1d58ddd66fa0771a1c08b2c4e231d9b90d84`. Replay de 216 informes históricos sin perfil idéntico; es compatibilidad con esos inputs, no cobertura universal del límite. Todos los originales y primeras capturas se conservan separados.

## Puertas, próximos pendientes y coste

Quedan QFaro (finalidad/control), Lino (alcance de «no…hasta»), credencial suficiente/posesivo y tres fuentes con «en Nacar». El [análisis fuente-only de las seis entradas](documentary-v3-remaining-six-source-design-2026-10-22.md) contiene hipótesis y un siguiente candidato sobre credencial; no cierra ninguna fila. Los análisis de unión lógica y cobertura de adjunciones distinguen indeterminación fiel de omisión, sin debilitar fuente o inventar un inventario cerrado.

Además de los seis pendientes de fidelidad queda la agregación de respuesta anónima en el control nuevo; se conserva como fallo del mecanismo, no como extracción aceptada de utilidad. Después se mantienen las puertas originales: guía y vocabulario coherentes, extracción completa nueva de las 116 entradas con doble revisión, utilidad sin pérdidas ni ajenos, comparación con producto, recursos/estabilidad, casos nuevos reservados y adopción solo si pasan todas. No se midió latencia de producto ni recuperación nueva y no se renuevan gráficas con controles de otro alcance.

**Cero llamadas externas nuevas; USD 0 nuevos de proveedor.** Ocho contextos Codex nuevos; coste y tokens Codex desconocidos. El ledger externo conserva 6129 operaciones, 23 campañas, USD 4,565416508 conocidos y veinte importes desconocidos, excluyendo Codex. La contabilidad local exacta se guarda por separado. Sin servicio, credenciales, dependencias, commit, push o cambio predeterminado del producto.

Artefactos: `ambiguity-closure-audit-v1/`, `nominal-de-contract-v1/` y `known-nominal-correction-v1/` bajo la campaña documental V3. La instantánea 114 conserva entradas, dictámenes, fallos, código y verificación. El objetivo sigue activo y no está logrado.
