# Consenso con cuatro candidatos: fallo de calidad

Fecha:2026-10-19. Desarrollo conocido v4; no promovido. Se detuvo antes de v2/v5 conforme al [protocolo previo](roles-consensus-compact-v1-protocol-2026-10-19.md). Las etiquetas y preguntas se conservaron.

## Medidas

| Categoría | Base | Consenso compacto |
| --- | ---: | ---: |
| Apoyo útil |16/16|16/16|
| Refutación útil |13/17|12/17|
| Contexto parcial útil |14/16|15/16|
| Ausencias correctas |13/15|15/15|
| Selecciones no permitidas |2|1|
| Nuevas degradaciones técnicas |0|0|
| Coste conocido USD |0,004501500|0,004063584|
| p50 operación ms |382,737|369,845|
| p95 operación ms |605,326|568,885|

Coste/p50/p95 relativos:0,903/0,966/0,940; recursos pasan el límite1,5. Hay cuatro ganancias útiles y cuatro regresiones bloqueantes, sin compensarlas por agregado. Todas las128operaciones terminaron y tienen recibo; costo del par completo USD0,008565084. La candidata hace dos peticiones por operación:192peticiones en el conjunto con base, sin confundir con128operaciones. No incluye Codex ni respuestas finales.

## Por qué no pasa

El cribado local conservó evidencia útil y mejor nivel de relación en48/48 útiles v2 y49/49 v4. Sin embargo la clasificación falló aun teniendo la evidencia correcta entre los cuatro primeros:

- `cev4q_064_02`: solo un voto reconoce que no se instaló batería nueva; consenso vacío pierde la corrección de la base.
- `cev4q_065_08`: ambos votos se abstienen ante la retirada del envío por texto; pierde otra corrección.
- `cev4q_063_07`: ambos eligen permiso de Laurel ante una pregunta de Azafrán, aunque la revocación de Azafrán estaba primera. Un acuerdo incorrecto sustituye evidencia útil.
- `cev4q_060_05`: ambos se abstienen ante la retirada de sol directo; pierde una corrección.

Estos mismos votos con16candidatos habían elegido evidencias permitidas en el intento anterior. No se atribuye causalidad aislada al contexto eliminado o al número de etiquetas: son llamadas distintas y no un estudio de invariancia exhaustivo. Sí refuta que cobertura local suficiente garantice calidad del recorte. No convertir una abstención nueva que elimina refutación en mejora de seguridad.

[Informe](../../outputs/iromem/evaluation/roles-consensus-compact-v1-development-2026-10-19/cohort-v4/report.json), [análisis de fallos y candidatos](../../outputs/iromem/evaluation/roles-consensus-compact-v1-development-2026-10-19/failure-analysis.json), [preflight](../../outputs/iromem/evaluation/roles-consensus-compact-v1-development-2026-10-19/preflight.json). El límite4 y la política quedan explícitos para reproducir, sin sustituir las variantes anteriores o baseline.

## Arreglo incidental de recibos

Un crítico autorizado separado encontró que una respuesta de elección inválida perdía su recibo válido y las ramas de error podían sobrescribir/descartar el otro parcial. Dos regresiones fallaron antes del arreglo y pasan después: extracción defensiva de campos válidos antes de validar la elección y agregación de ambos subtotales conocidos. Las ausencias siguen desconocidas. El consenso por lote conserva su recibo global una vez. La revisión posterior no encontró nuevas duplicaciones; sigue siendo revisión estática.

La campaña terminó antes del arreglo y no tuvo errores técnicos: sus128recibos y costes no se cambiaron. El hash previo se preserva; la versión posterior solo corrige tratamiento de errores y añade pruebas, sin cambiar decisiones válidas.61pruebas workspace pasan. No se reejecutó la campaña para buscar un pase. El banco nuevo permanece sin usar.

## Decisión

Descartar la promoción del recorte y conservar el arreglo de contabilidad. El mejor resultado de calidad sigue siendo el consenso16, bloqueado por p95. La próxima revisión debe atacar una causa semántica o de transporte verificable, evitando encoger una lista solo por cobertura o encadenar prompts sin hipótesis y contraste. El objetivo de cinco puertas sigue activo.
