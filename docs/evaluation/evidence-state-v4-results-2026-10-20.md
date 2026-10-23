# Estado de evidencia V4: contrato explícito y prueba local

UTC 2026-10-20. **32/32 resultados de contrato correctos; calidad autónoma pendiente.** El banco y sus hechos son manuales. No hay extracción automática, integración, mejora de latencia ni rendimiento poblacional demostrados. El goal sigue activo y baseline continúa predeterminada.

## Cambio de contrato, conservando historia

[V4](evidence-contribution-v4-contract.md) amplía A con estado explícito de ejecución de un procedimiento que mide la magnitud solicitada. Realizado, no realizado y pendiente informan de evidencia, con sujeto, magnitud, método y condiciones; no acreditan valor ni desconocimiento global. Previsto no es pendiente de ejecución; no publicar un resultado de ensayo no identificado como máximo no acredita no ejecución ni ausencia documental del máximo. Se distinguen especificación, norma, observación y capacidad.

La incertidumbre se conserva como conjunto exacto de roles. Si todas las lecturas son útiles se conserva el texto con relación incierta y calificación obligatoria; {A,B} no se convierte en B. Si incluye none se deriva a revisión, se cuenta como posible evidencia no seleccionada y falla la puerta autónoma. La regla no recupera información ausente de un texto ilegible.

Esto cambia expresamente el alcance de A y el resultado estructurado; no repara retrospectivamente V3. El diagnóstico previo sigue en **98/100**, una discrepancia y un ambiguo. Sus informes bruto y puntuado se reprodujeron byte por byte con el código actual. No se comparan 32/32 V4 con 98/100 V1 como si fuese una mejora del mismo sistema.

## Revisión y mecanismo implementado

Banco nuevo de desarrollo controlado: 32 pares, nueve preguntas y 30 textos únicos. Incluye ejecución/planning, pendiente, magnitud o sujeto ajenos, método genérico sin magnitud, componente, condiciones, publicación, cancelación, límite observado, máximo especificado, contraste, refutación y ambigüedad de identidad. No es V6 ni la validación nueva del paso 4.

El crítico recibió contrato y textos/hechos sin etiquetas ni resultados. Se preservaron borrador, primera revisión, correcciones y revisión final: pendiente sustituía incorrectamente a planned; capacidad a asignación; faltaban máximo, especificación, observación, resultado limitado al texto, cancelación y carácter normativo de política. Se corrigieron antes de ejecutar. Las etiquetas revisadas concuerdan en todos los campos con la revisión ciega final. Es IA de la misma familia, sin revisión humana ni independencia entre familias.

`tools/evaluation/src/typed_facts/v2.rs` calcula relación desde hechos, con ontología de propiedades, magnitudes y procedimientos. La relación magnitud/método se declara independientemente de la pregunta. Los IDs y etiquetas no entran en el cálculo. Se preserva V1, ampliando su parser compartido con las bases de fuente specification/normative; los resultados V1 permanecen idénticos. No cambia `recall`, SQLite, dependencias, configuración, dashboard ni servicios.

La crítica estática detectó pérdida de identidad del método en premisas pendientes, comodines por campos omitidos y comprobación insuficiente de conciliación. Se corrigieron. Otra revisión encontró distinta polaridad según se expresase el método en value o measurement_method y aceptación de aliases contradictorios: se normalizó la representación y se rechaza contradicción, con regresiones. El puntuador exige hashes, revisión final, cierre declarado de objeciones y concordancia real de roles/límites, además de reconstruir el resultado desde el código.

## Resultados ejecutados

| Medida local | Resultado |
| --- | ---: |
| Resultados completos coincidentes con revisión | 32/32 |
| Seleccionados automáticamente bajo hechos manuales | 20 |
| Rechazados con none en todas las lecturas | 11 |
| Utilidad sin resolver | 1 |
| Seleccionados con relación A/B y calificación | 1 de los 20 |
| Resultados que usan estado de medición para cantidad | 9 |
| Útiles en al menos una lectura | 21 |
| Posible útil sin selección automática | 1 |
| Puerta de comportamiento límite | Pasa |
| Puerta de calidad autónoma | **Falla** |

`esv4_16` contiene un teléfono y una etiqueta cortada que puede ser Q2 o Q3. Seleccionar una lectura inventaría identidad; descartarlo como irrelevante ocultaría evidencia posiblemente útil. Se informa A/none y revisión pendiente. No hay un error de coincidencia del comportamiento esperado, pero tampoco recuperación autónoma completa. Debe resolverse la representación/transmisión de incertidumbre y sus límites antes de calificar el mecanismo completo.

Las 96 comparaciones adicionales —orden, renombrado y ambos, sobre los 32 pares— conservaron todos los roles, selección y flags, incluido el caso sin resolver. Pasan 329 pruebas con configuración predeterminada y con HTTP/2 optativo; compilación y formato pasan sin avisos. Catorce pruebas nuevas cubren estados, condiciones, componentes, identidad, aliases, negación y revisión. Se reprodujeron informes V4 y los 100 pares V1 byte por byte. Una copia con hashes válidos pero revisión incompatible se rechazó; también sobrescribir el informe.

[Fuente revisada](../../tests/fixtures/evidence-state-v4/source-only-reviewed.json), [revisión final](../../tests/fixtures/evidence-state-v4/blind-review-final.json), [conciliación](../../tests/fixtures/evidence-state-v4/adjudication.json), [resultados brutos](../../outputs/iromem/evaluation/evidence-state-v4-2026-10-20/report.json), [métricas](../../outputs/iromem/evaluation/evidence-state-v4-2026-10-20/score.json), [verificación](../../outputs/iromem/evaluation/evidence-state-v4-2026-10-20/verification.json).

## Coste y puerta siguiente

Cero llamadas de proveedor nuevas y **USD 0 de coste remoto nuevo**. Acumulado conservado: 5881 operaciones de proveedor, 23 campañas completas, USD 0,582095598 conocidos más cinco operaciones de importe desconocido. Excluye programación/anotación/revisión de Codex. La ejecución local y las transformaciones no se suman al contador remoto. No se midió coste, p50 o p95 de un recorrido automático ni se atribuye ahorro a este prototipo.

Los cinco objetivos se mantienen: contrato V4 explícito; banco local revisado; resolver local implementado, con posible útil aún sin selección; extracción y mecanismo completo pendientes; casos nuevos, estabilidad, recursos y adopción pendientes. V6 conserva hashes y cero llamadas. No se modifica README ni sus gráficas con un porcentaje de hechos manuales.

La siguiente investigación debe representar la incertidumbre documental como tal, sin completar un nombre cortado ni atribuir el teléfono a Q2. Separar hechos de mundos posibles y conocimiento disponible en una fuente; comprobar con revisión ciega si puede conservarse una evidencia limitada sin afirmar su vínculo. También falta demostrar suficiencia general de V4 sobre todo el desarrollo conocido antes de preparar extracción. No escoger otra etiqueta ni retirar el caso difícil para pasar.
