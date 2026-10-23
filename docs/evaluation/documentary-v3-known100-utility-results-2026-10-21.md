# Diagnóstico de utilidad V3 en los 100 pares conocidos

Se ejecutaron por primera vez juntos los 100 pares del banco documental conocido con los 116 frames manuales revisados. Se congelaron mapa por texto, entradas, etiquetas históricas, binario y criterio de selección antes de ejecutar. No es extracción automática ni validación ciega: la referencia corresponde al banco V5 anterior y no fue readjudicada para V3. La selección significa `selected_claim_ids` no vacío, separada de respuesta completa y de candidatos pendientes.

La primera corrida detectó 27 selecciones: 21 entre las referencias útiles y seis entre las 60 referencias `none`. Solo tres pares producían respuesta completa. Cinco discrepancias seleccionaban datos parciales de nombres no acreditados: pedido O8 ante O7, Cedro ante Arce, Rocío ante Brisa y los cruces Atlas/Boreal. Todos advertían identidad pendiente, pero esa advertencia no impedía incluir el dato en `selected_claim_ids`.

Se añadió un control de anclaje nominal para respuestas parciales y las rutas de contexto compartidas. Exige correspondencia documental de los nombres competidores a través de alias locales y conserva sus vínculos. La falta de prueba queda pendiente, sin demostrar diferencia entre entidades. Una clase no acreditada todavía permite evidencia parcial bajo un nombre acreditado; no completa la respuesta. Las referencias anónimas y variables universales mantienen el tratamiento parcial previo: este control no prueba su aplicabilidad ni cierra el problema de identidad anónima.

Se conservaron dos iteraciones fallidas. La primera aplicación del control interfería con una exclusión válida de conjunto cerrado; se restituyó la comparación de verdad por diferencia explícita. La siguiente repetición completa eliminó las cinco selecciones sin anclaje, pero perdió una regla universal útil para un archivo individual. Se conservó la pérdida y se corrigió la exigencia de nombre sobre variables anónimas antes de repetir los mismos 100 pares.

| Diagnóstico con referencia histórica | Antes | Iteración fallida | Revisión final |
| --- | ---: | ---: | ---: |
| Referencias útiles seleccionadas | 21 | 20 | 21 |
| Referencias `none` seleccionadas | 6 | 1 | 1 |
| Respuestas completas | 3 | 3 | 3 |
| Concordancia de selección / 100 | 75 | 79 | 80 |
| Pérdidas útiles respecto a antes | — | 1 | 0 |

La mejora medida es acotada: cinco selecciones nominales sin anclaje eliminadas sin perder las 21 selecciones útiles anteriores ni las tres respuestas completas. Los textos, frames, referencias y denominadores permanecen iguales. No es una comparación con baseline ni una mejora automática de recall. El dato 80/100 mezcla clases y no se presenta como exactitud global del producto.

Sigue fallando la conservación completa: de 39 referencias útiles no ambiguas, solo 21 se seleccionan (18/29 A, 3/7 B y 0/3 C); la referencia ambigua tampoco se selecciona y permanece separada. Hay además una selección parcial de un archivo individual frente a una pregunta universal (`pcv1_01_02/claim_01`), conservada como discrepancia pendiente de interpretación/alcance, sin convertirla automáticamente en falsedad factual. Los anclajes anónimos, información explícitamente desconocida, restricciones y relevancia parcial requieren más trabajo. El intento automático previo sigue en 65/116 fieles para ambos críticos; no se repitió ni reparó.

El primer runner de repetición falló antes de producir informe por no crear su directorio de salida; se creó el directorio y se volvió a ejecutar sin sobrescribir resultados. No se confunde ese error de orquestación con un fallo semántico.

**595 pruebas pasan, 24 omitidas; 75 V3**, dos controles nuevos. Compilación y formato pasan. El binario final reproduce idénticos los 100 informes finales y 248 de los249informes anteriores. El par024/006 cambia su diagnóstico pendiente por el control nuevo, conservando selección vacía y ausencia de respuesta completa; ambos informes permanecen. La primera verificación que esperaba igualdad total falló y se conserva como hallazgo, sin atribuir igualdad a ese par.

Cero API y USD 0 nuevos; coste/tokens Codex desconocidos. Sin reactivar agentes, modificar producto o abrir V6. El goal permanece activo en paso 3: resolver las pérdidas conocidas y el alcance pendiente antes de una nueva extracción congelada y las puertas completas de baseline, recursos, estabilidad y casos nuevos.

[Protocolo](../../outputs/iromem/evaluation/documentary-v3-known100-utility-2026-10-21/protocol.json), [antes](../../outputs/iromem/evaluation/documentary-v3-known100-utility-2026-10-21/report.json), [iteración con pérdida](../../outputs/iromem/evaluation/documentary-v3-known100-utility-2026-10-21/after-report.json) y [revisión final](../../outputs/iromem/evaluation/documentary-v3-known100-utility-2026-10-21/after-r2-report.json). La verificación del hash final y las pruebas se registran en `execution-provenance.json` del mismo directorio.
