# V3: banco manual completo, revisión y nueve casos pendientes

Se construyeron y revalidaron representaciones para los 116 textos conocidos, conservando todas las iteraciones. **105 son estructuralmente completas y 11 parciales; ninguna inválida.** La revisión manual final considera 107 fieles y nueve incompletas; la puerta completa falla. Estas cifras no son una nueva medición automática ni se comparan con los 93/116 del extractor anterior. El goal sigue activo en el paso 3; baseline y V6 permanecen intactas.

[Informe estructurado](../../outputs/iromem/evaluation/documentary-v3-known-lowering-2026-10-21/report.json), [selección de los 116 frames](../../outputs/iromem/evaluation/documentary-v3-known-lowering-2026-10-21/selected-frames.json), [revisión final](../../outputs/iromem/evaluation/documentary-v3-known-lowering-2026-10-21/fidelity-review-final.json) y [procedencia](../../outputs/iromem/evaluation/documentary-v3-known-lowering-2026-10-21/execution-provenance.json).

## Construcción y revisión

Tres autores trabajaron inicialmente por separado: 42 fuentes, 41 fuentes y 33 preguntas. Vieron texto, matriz lexical, vocabulario y contrato; no gold, mapa de pares, frames anteriores ni V6. Pudieron corregir sus propios JSON mediante el validador. Después recibieron crítica y exposición cruzada declarada para corregir convenciones. Es construcción asistida de desarrollo, no extracción congelada ni revisión ciega.

Un crítico revisó las 116 entradas. Se preservan V1 (94 fieles, 16 defectuosas, seis inciertas), R2 y final, junto con los frames originales. Las revisiones corrigieron posesión inventada, clases de archivos crudos, propósito, tiempo histórico, contexto colectivo y membresía inferida de «de Salvia». En las revisiones posteriores se reinspeccionaron cambios e interfaces y se reutilizaron juicios solo tras comprobar hashes iguales; no fueron nuevos revisores independientes. Dos correcciones del propio crítico sobre el genitivo ambiguo se registran explícitamente: el contexto neutral conserva el texto sin exigir geografía/modelo/membresía ausentes. No representan ganancias de modelo ni cambios de gold.

Los dos frames de esas fuentes conservan estado estructural parcial por su señal de cobertura conservadora; por eso las cuentas estructurales y semánticas difieren. No se elimina esa señal para obtener un pase.

Quedan nueve entradas: `source_005` y `question_001`, `004`, `005`, `006`, `017`, `018`, `019`, `030`. Corresponden a suficiencia normativa elíptica/tutor, filtros relacionales sobre la respuesta de origen en una red, plazo común sobre todos los archivos y cardinalidad del grupo propietario sin contar sus faros. Están preservadas en el denominador.

## Cambios del mecanismo

El vocabulario admite una séptima clave opcional `attribute_profiles`. Declara, mediante filas finitas, tipos y unidades permitidos para `quantity.amount`, `property.value` y `event.result`. Su contenido forma parte del hash operativo del vocabulario. Un perfil mal formado invalida la operación; un valor o unidad incompatibles deja pendiente esa fila y sus dependientes. También se inspeccionan los miembros de restricciones cerradas, que inicialmente escapaban a esta comprobación y fueron corregidos tras crítica.

La comparación usa una vista privada que excluye claims incompatibles antes de buscar hechos, guards, reglas, eventos e invariantes. El raw y el informe validado originales permanecen íntegros. Una solicitud con perfil pendiente termina sin selección; los hechos independientes válidos se conservan. No se convierten unidades ni se prueba que una cita realmente respalde el valor.

El perfil es optativo. Los vocabularios históricos de seis claves y los 116 frames de este banco siguen sin esa tabla; su compatibilidad semántica no se declara aprobada por esta extensión. Faltan poblar/revisar la tabla del dominio y cubrir otros portadores de cantidades que no estén entre los tres declarados. No hay cambio de producto ni de política predeterminada.

## Verificación

| Comprobación | Resultado y alcance |
| --- | --- |
| `cargo test --offline --workspace` | 577 pasan, 24 omitidas, cero fallos; 57 pruebas V3, 12 nuevas |
| Pruebas nuevas | Ocho puras del perfil y cuatro integradas: aislamiento de datos incompatibles, hecho independiente, unidad de pregunta, no conversión, miembros de restricciones y premisa transitiva |
| CLI del banco manual | Los 116 informes se reproducen byte a byte con el binario actual |
| Control de política colectiva | La pareja manual sobre Iris devuelve política completa con ámbito citado, sin inferir consecuencias por miembro |
| CLI del perfil opcional | Dato permitido aceptado; dato deliberadamente incompatible pendiente y no seleccionado |
| Historia | Cuatro bancos manuales/puntuaciones, diez informes de extracción, seis semánticos, tres auditores y cuatro informes Codex reproducidos idénticos |

Compilación y formato pasan. La crítica del código es estática y acotada; su ejecución corresponde a los logs del padre. No se repitió HTTP/2 ni se midieron recursos, estabilidad de modelo, latencia de producción o recall.

## Calidad, coste y continuación

La última calidad automática sigue en **102/116 DTO válidos y 93/116 aceptados en cuatro dimensiones por ambos críticos originales**. Sus 928 juicios y 29 desacuerdos permanecen intactos. Los 107 juicios favorables sobre frames corregidos manualmente no mejoran esa métrica ni abren la cohorte ciega.

Esta tanda añade **cero llamadas a proveedores y USD 0 de API**; coste/tokens de Codex desconocidos. Acumulado: 6129 operaciones lógicas, 23 campañas emparejadas completas, USD 4,565416508 conocidos y 20 operaciones con coste desconocido, excluyendo Codex.

Para evitar seguir ampliando comprobaciones previas, el siguiente trabajo se limita a los nueve casos concretos: primero comprobar los cuantificadores existentes en plazo común y cardinalidad de propietarios; después añadir solo el comportamiento necesario para filtros de respuesta y contexto normativo. Tras cerrar y revisar esos casos se congela una extracción automática nueva, se compara utilidad/known126/baseline y se miden recursos/estabilidad antes de V6. No se sustituye esa comparación por más porcentajes sobre el banco manual.
