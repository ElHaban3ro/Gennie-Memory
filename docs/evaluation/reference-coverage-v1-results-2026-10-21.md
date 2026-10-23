# Residuales de referencias V1: diagnóstico local y límites comprobados

UTC 2026-10-21. **El diagnóstico localiza palabras citadas que no llegaron a las hojas de identidad, pero no permite certificar ni reparar semántica.** Se conserva esta herramienta de evaluación; no se amplía el extractor ni se cambia el producto. Goal activo en paso 3. Coste nuevo de proveedor: **USD 0**, cero llamadas. [Protocolo](reference-coverage-v1-protocol-2026-10-21.md), [verificación](../../outputs/iromem/evaluation/reference-coverage-v1-2026-10-21/verification.json).

## Implementación y evidencia

`audit-reference-coverage` recorre todos los objetos de referencia emitidos, incluidos propietarios, ciertas/lecturas, pregunta, premisas y valores anidados. Cada hoja name/type_quote se liga a una cadena de menciones contenedoras con ubicación única; los intervalos candidatos y elegibles quedan separados. Citas contenedoras no consumen por sí solas campos faltantes. Las repeticiones ambiguas no se asignan arbitrariamente. La partición [inicio,fin) en bytes UTF-8 reconstruye la mención original sin normalizar Unicode, case, artículos o plurales. Las referencias externas se auditan por separado; nunca cubren un homónimo local desde otro contenedor. Ausente/malformado/no literal/unsupported/no disponible no significa residual cero.

Se ejecutó sobre **los ocho borradores originales completos**, sin seleccionar por DTO ni leer etiquetas/revisiones desde el instrumento. Se conservaron 18 nodos, incluyendo repeticiones en lecturas y propietarios; no son 18 casos independientes. Todas sus particiones reconstruyen exactamente los bytes originales. [Informe íntegro](../../outputs/iromem/evaluation/reference-coverage-v1-2026-10-21/coverage.json).

La comparación descriptiva posterior con los errores ya conocidos muestra:

| Operación | Ejemplo de fragmento no consumido por hojas | Interpretación limitada |
| --- | --- | --- |
| op_000 | «del pedido O7 de» | Expone palabras de la restricción omitida, junto con conectores. |
| op_001 | «el pedido O7 de» | El pedido y su nombre no aparecen en las hojas del sujeto. |
| op_054 | «del huerto Verbena» | El propietario no se conserva estructuralmente. |
| op_021 | «el» y espacios | Hay residual aunque ambos críticos anteriores consideran fiel la extracción. |
| op_088 | «Cima» | El nombre quedó únicamente en la mención. |
| op_043 | «los faros de» en el nodo del propietario | Permite inspeccionar la cita excesiva, sin adjudicar su portador automáticamente. |
| op_115 | «Del expediente» | La clase explícita falta en la hoja, junto con un conector. |
| op_065 | «expediente Atlas» | Nombre y clase no aparecen en las hojas. |

Los fragmentos se abrevian aquí para lectura; el JSON conserva espacios, paths, candidatos y particiones exactos. No se calculó precisión, recall, calidad por porcentaje de caracteres ni umbral sobre esos residuales. La inspección es retrospectiva conocida, no revisión ciega nueva ni evidencia causal de mejora. Las métricas de extracción **no cambian** respecto del [piloto anterior](grounded-revision-v1-results-2026-10-21.md): 4/8 DTO, dos acuerdos semánticos y uno incluyendo evidencia.

## Qué quedó refutado

Los controles fabricados preservan los ocho riesgos LA01–08. Una premisa corregible o un cuantificador puede dejar residual legítimo; no debe trasladarse automáticamente a identidad. Un nombre homónimo con residual cero no prueba correferencia. Scope, cardinalidad, orden de propietarios, pertenencia/composición aditiva y tipo ontológico pueden estar equivocados con idéntica cobertura literal. La identidad distribuida entre cláusulas o una referencia nunca emitida puede no dejar ninguna pista dentro de la mención corta observada.

Por tanto, este instrumento no justifica transformar automáticamente menciones en referencias completas, eliminar palabras restantes ni reemplazar la identidad estructurada por un ancla opaca. Tampoco justifica otra revisión del mismo modelo con feedback por sí solo. La investigación avanza al descartar ese supuesto antes de gastar en red, manteniendo íntegra la expresividad exigida.

## Verificación, fallos conservados y estado

Pasaron **462 pruebas** por suite, con y sin HTTP/2; quince pruebas nuevas del instrumento. Los controles cubren Unicode y reconstrucción, solapamientos, repeticiones, contenedores externos, referencias vacías o sin mención, operaciones no disponibles, roster incompleto y salida existente inmutable. Cuatro bancos manuales y diez informes históricos se reproducen byte por byte, igual que el informe nuevo. Formato y diff-check pasan; fuentes/binario con hashes guardados.

La primera compilación falló al intentar formatear el digest SHA-256 como LowerHex; se corrigió con codificación hexadecimal por byte, sin dependencia nueva. El log se conserva. El crítico estático identificó cuatro contraejemplos mecánicos durante el desarrollo; todos tienen corrección, regresión y revisión final separada. Esa crítica tiene exposición histórica y no se presenta como revisión semántica ciega. El aviso histórico de método compartido no utilizado en el crate de pruebas bootstrap permanece; la suite y compilación terminan correctamente. [Críticas conservadas](../../outputs/iromem/evaluation/reference-coverage-v1-2026-10-21/code-critic-final2.json).

Sin llamadas ni cargos nuevos: acumulado **6001 operaciones, 23 campañas emparejadas completas, USD 0,845298508 conocidos más 16 operaciones de coste desconocido**, sin Codex. No cambia la latencia medida del extractor; este trabajo no es una medición de recall/end-to-end o un pase de recursos. Snapshots001–057 intactos; nuevo cierre058. Sin cambios de default, gold, README/gráficas, dependencias, SQL, dashboard o servicios. V6 conserva su manifest sin lectura semántica ni llamadas.

Continúan pendientes extracción/alineación completas, calidad en conocidos y difíciles, recursos/estabilidad y cohorte nueva ciega. Antes de otra llamada debe elegirse un mecanismo que afronte la extracción semántica, con controles de expresividad, sin convertir el diagnóstico literal en una certificación. La herramienta aporta observabilidad; no satisface por sí sola el objetivo de mejorar calidad.

La crítica del siguiente diseño considera distinta la hipótesis de extraer primero referencias y luego cláusulas: reduciría interferencia entre tareas, pero aún no demuestra mejor descomposición semántica. Se exige primero una matriz local completa de expresividad y un comprobador de conservación de nombres, clases, propietarios, cardinalidad, padres, supuestos y enlaces. La segunda etapa tendría que conservar también todos los hechos y calificadores; dos llamadas por ítem pueden exceder el límite de recursos. No se implementa ni ejecuta esa tubería ahora. Si falla la matriz local, se descarta antes de red; si pasa, solo habilita probar la etapa de referencias completa, sin escoger las salidas buenas. [Crítica y controles previos](../../outputs/iromem/evaluation/reference-coverage-v1-2026-10-21/next-design-critic.json).
