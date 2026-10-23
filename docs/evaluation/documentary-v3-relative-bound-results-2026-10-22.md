# Ubicación explícita y límite relativo de no medición

El banco mixto conocido pasa de **108/116 a 109/116 documentos completos y fieles para ambos críticos**; quedan siete. Se corrigió la descripción de la oficina y se implementó un operador temporal acotado en el evaluador. Pasan **774 pruebas**, cero fallos y 24 ignoradas; ocho controles CLI nuevos y 216 replays históricos idénticos. El objetivo de cinco pasos sigue activo en el paso 3, sin adopción ni V6.

## Cambio aceptado: la oficina de la esquina

La primera candidata mantenía «matrículas» como clase léxica no especializada y «oficina de la esquina» como clase nominal con una asociación débil. Un revisor consideró suficiente esa preservación; el otro detectó pérdida de la relación locativa. Ambos dictámenes y el candidato permanecen intactos. El desacuerdo no se convirtió en aceptación.

La segunda candidata representa el destino con el carrier existente `described_ref(office)` y una restricción explícita `located_at(object=office,place=corner)`. Mantiene el nominal «matrículas» y su vínculo a Faro sin inventar inscripción, placa, actor ni entrega consumada. Retira la asociación débil office→corner y conserva el sujeto de la ubicación dentro de la descripción del mismo destino. No añade predicados, atributos, DTO ni código para esta corrección.

Dos críticos nuevos fuente-only aceptaron las cuatro dimensiones: **8/8 faithful**. Recibieron únicamente un paquete congelado de vocabulario, guía, fuente y candidato, sin versiones anteriores, contraparte, gold, resultados, código o el otro dictamen. Son contextos separados de la misma familia, no validación humana ni independencia de familia. Se comprobaron hashes, citas UTF-8 y punteros.

Tres controles manuales tipados en textos sintéticos nuevos demuestran consumo de la ubicación: la coincidencia conserva apoyo completo; otra esquina u otro titular no reciben apoyo, respuesta completa ni refutación fabricada. Ambos documentos de cada par validan completos. No son preguntas del benchmark ni nueva extracción autónoma.

## Operador implementado: no ocurrencia con límite relativo

El feature optativo `relative_nonoccurrence_bound` del perfil contextual V1 añade un qualifier cerrado para un evento de medición negado. Su cuerpo conserva relación until, referente del límite y acción switch_off; el sujeto del límite coincide con el de la medición. La adjunción tiene propietario único, ámbito y dependencias explícitas. No afirma apagado ocurrido, valor, fecha ni medición posterior. Inicio e inclusión exacta del extremo quedan sin especificar. Se distingue de medir la dimensión `duration_until_shutdown`.

El [diseño y su revisión](documentary-v3-relative-bound-design-2026-10-22.md) explican forma, permisos y límites. Solo el evaluador admite este carrier, con su feature explícito; las entradas históricas no lo aceptan. `boundary_subject` participa en validación de referencias, dependencias y canonicalización semántica para que renombrar IDs no cambie el significado.

Antes de las rutas históricas se filtran el owner condicionado y los consumidores dependientes, preservando hechos independientes. Una ruta dedicada conserva contexto parcial útil solo con sujeto, aspecto y ámbito compatibles: transporta owner, límite y request íntegros, pero `answer_terms`, `answer_propositions` y `bindings` vacíos. El evento limitado no se transforma en negación global ni en respuesta numérica.

La revisión estática detectó un escape potencial por operandos implícitos de `normative_open_pattern`: una referencia normativa dependiente podía carecer de una arista redundante permitida por validación. El veto utiliza ahora el mismo lector de dependencias efectivas del validador y calcula el cierre inverso sobre aristas explícitas, implícitas, aliases y claims anidados. Una regresión con documentos completos comprueba referencia normativa directa y alias Same. El riesgo se detectó estáticamente; no se afirma haber reproducido una respuesta indebida antes del cambio. El crítico verificó estáticamente su cierre. El grafo nuevo también cobra sus aristas materializadas al presupuesto compartido.

Cinco controles CLI de textos nuevos pasan: dos consultas compatibles conservan únicamente contexto parcial, sin cifras ni conclusión inventada; otras tres con sujeto, atributo o aspecto distintos no seleccionan el owner. Los nueve tests focales incluyen propiedad, tipos, aristas, perfil, aislamiento, referencia canónica, hechos independientes, dependencia implícita y agotamiento compartido. Estos controles no constituyen un motor temporal general ni demuestran todos los recursos del recorrido de producto.

## Intentos que no se aceptaron

La candidata de «bandeja de papel grande» preserva las dos adjunciones propuestas con el descriptor léxico existente. Un crítico dio faithful en las cuatro dimensiones; el otro dejó incertidumbre sobre exhaustividad. No se adjudicó aceptación ni se cambió el dictamen. La versión previa del banco sigue vigente para ese ID.

La candidata de Lino distingue dimensión medida y alcance de la no ocurrencia en dos hojas estructuralmente completas. Ambos críticos aceptaron contenido y atribución, pero dejaron incierto el cierre del inventario, incluida la posible lectura actualizante de «no…hasta». No afirmaron como hecho obligatorio un apagado, medición positiva o resultado; tampoco acreditaron exhaustividad. **No se incorporó esa candidata a la calidad conocida**, aunque el operador y sus controles funcionan.

Esto distingue una representación que ya puede transportar un límite de una extracción fiel cuya cobertura interpretativa aún no se ha acreditado. El siguiente avance necesita cerrar esa deuda sin cambiar etiquetas, repetir juicios hasta obtener uno favorable ni confundir cita o forma válida con conservación. Los paquetes, candidatos, revisiones y desacuerdos permanecen congelados.

## Métricas y pruebas

| Métrica | Anterior | Actual |
| --- | ---: | ---: |
| Mixto conocido completo estructuralmente | 110/116 | 111/116 |
| Mixto conocido fiel para ambos críticos | 108/116 | 109/116 |
| Mixto conocido completo y fiel para ambos | 108/116 | 109/116 |
| Pendientes combinados | 8 | 7 |

Se revalidaron los 116 documentos del conjunto mixto con el binario actual. Los 115 que no cambian producen informes idénticos a sus versiones anteriores. La consolidación usa una sola corrección aceptada, no suma pilotos ni versiones de candidatos rechazados. Continúa siendo un conjunto conocido con vocabularios por documento, no una extracción nueva con guía/vocabulario comunes. La extracción autónoma original **99/116** permanece intacta, igual que la utilidad histórica **31/39**, cero ajenos/60.

La suite workspace actual pasa **774 pruebas**, cero fallos y 24 ignoradas; `cargo fmt --all --check` pasa. Los 216 informes históricos sin perfil se reproducen byte a byte. Binario final SHA-256 `e4408d91bf23030a44190e2a9764a0beecb3af14335ba48ce99ec88009caded0`.

Se preservan dos fallos de preparación/verificación. En la primera tanda focal pasaron dos tests y fallaron cuatro: el grafo histórico rechazaba qualifier como consumer de dependencia. Se habilitó únicamente para este carrier optativo. Después, la regresión nueva falló por dos dependencias ordinarias omitidas en la pregunta sintética; se reparó el fixture sin relajar validación. Las tandas focales posteriores pasaron 8/8 y 9/9. Los logs y el recibo del primer fallo están separados de la suite final. Se ejecutaron dos suites workspace porque después de la primera se añadió contabilidad de materialización del grafo.

No se midieron recuperación nueva, comparación contra producto, latencia de producto, estabilidad de extracción ni V6. No se renuevan gráficas del README con controles de otro alcance. Las pruebas existentes no acreditan integración automática ni extracción escalable.

## Estado del objetivo y coste

El turno anterior y este son avances verificables; no hubo espera por permisos. Siguen pendientes siete casos: pregunta de tutor/control/finalidad; Lino y alcance de hasta apagarse; credencial suficiente y posesivo; tres fuentes con en Nacar; y adjunción bandeja/papel. Después siguen vigentes las puertas originales de extracción completa con guía y vocabulario comunes/doble revisión, utilidad sin pérdidas, comparación de producto, recursos y estabilidad, V6 ciega y adopción solo si pasan todas.

**Cero llamadas externas nuevas; USD 0 nuevos de proveedor.** Nueve contextos de subagentes nuevos y tres turnos adicionales sobre diseño/implementación. Coste y tokens Codex desconocidos. No se equiparan esas revisiones con coste o latencia de extracción de producto. El ledger externo conserva 6129 operaciones, 23 campañas, USD 4,565416508 conocidos y veinte importes desconocidos, excluyendo Codex.

Hubo 348 operaciones CLI locales: cuatro validaciones iniciales de candidatos, tres controles de ubicación iniciales, ocho controles finales, 216 replays, 116 validaciones mixtas y una validación final de la candidata Lino rechazada. Ocho comandos Cargo de build/test y cinco de formato, seriales. Sin servicio, cambio de credenciales/dependencias, commit, push ni política predeterminada nueva.

Artefactos en `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`: `lexical-attachment-corrections-v1/`, `office-location-correction-v2/`, `relative-bound-contract-v1/` y `known-office-correction-v1/`. La instantánea 113 conserva originales, revisiones, código y verificación. El objetivo continúa activo y no está logrado.
