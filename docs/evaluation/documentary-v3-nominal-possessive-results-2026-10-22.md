# Posesivo nominal: implementación y revisión documental

Estado: mecanismo optativo implementado y verificado; continúa el paso 3 del objetivo. No hay adopción de producto ni nueva medición de extracción autónoma. El conjunto corregido conocido conserva 110/116 entradas completas y aceptadas por ambos revisores; quedan seis. La base autónoma histórica conserva 99/116.

Se introduce un carrier optativo `nominal_possessive` con uso léxico del determinante y argumento contextual abierto. El enlace conserva la construcción nominal sin convertir «su» en una persona, propiedad física o identidad demostrada. El contexto parcial transporta ambos documentos tipados íntegros, la proposición y petición originales; mantiene vacíos los términos, proposiciones de respuesta y bindings. Los consumidores dependientes se apartan de las rutas que podrían producir una respuesta completa. La proyección existencial no inventa un referente para ese argumento.

El diseño y sus contraargumentos están en `documentary-v3-possessive-design-2026-10-22.md` y `documentary-v3-possessive-countercritique-2026-10-22.md`. La revisión estática se registra en `documentary-v3-possessive-static-review-2026-10-22.md`. Son evidencia de diseño/código, distinta de ejecución y fidelidad documental.

La primera propuesta fuente de la credencial fue rechazada: el revisor A encontró pérdida de la correlación entre la obligación eliminada, tutor e Ina; el revisor B dejó incertidumbre sobre el antecedente del posesivo. Ambos dictámenes se conservan. La propuesta V2 describe el contenido de la obligación paciente mediante `described_ref` y un predicado dinámico explícito; esa corrección no resuelve por sí sola la discrepancia sobre el posesivo. No se suma esta candidata al porcentaje aceptado.

La primera compilación focal falló por seis inicializadores de prueba sin los campos nuevos del motor. Se detectó además un fixture truth con slot/extent incorrectos. Estos fallos de preparación se conservan separados de los controles semánticos. La revisión estática encontró tres defectos: reserva ignorada antes de clonar, atribución del lado fuente mediante identidad de puntero y mezcla de bindings de diferentes filas de relevancia; añadió una revisión de cobro del lookup de cierres. Los cuatro defectos están corregidos y revisados por lectura. La ejecución detectó además un import ausente y dependencias qualifier→scope no admitidas en un test; se corrigieron los fixtures sin cambiar sus expectativas. La frontera léxica impide aceptar «suficha» como dos palabras. Los logs iniciales siguen conservados.

Artefactos de esta tanda: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/nominal-possessive-contract-v1/`. No hubo llamadas externas: coste API incremental USD 0; el consumo de subagentes Codex no está cuantificado. No se abrieron los casos reservados V6 ni se cambiaron etiquetas, dictámenes históricos o la configuración predeterminada.

## Verificación ejecutada

- `cargo test --offline --workspace`: **800 aprobadas, cero fallos, 24 ignoradas**; diez pruebas focales nuevas y formato correcto.
- **11/11 controles CLI**: nueve con documentos completos y dos rechazos técnicos de inputs inválidos. Conservan contexto parcial íntegro, rechazan beneficiario/régimen/complemento/fundamento incompatibles y no inventan un poseedor bajo proyección existencial. La mezcla de filas de relevancia falla en ambos órdenes; una única correlación completa correcta conserva el parcial.
- **358/358 informes idénticos por bytes**: 216 históricos, 116 del conjunto vigente, 18 controles tipados anteriores, el control original sin feature y siete controles de proyección existencial.
- Ambas propuestas de credencial validan estructuralmente completas; la primera conserva dos dictámenes adversos y la segunda no recibe aceptación total por corregir solo la obligación. **Ninguna se incorpora al conjunto aceptado**. No se vuelve a sortear revisores para intentar cambiar el dictamen de un componente intacto.

El primer runner CLI obtuvo 7/8 por consultar campos V1 en un informe V2; el informe del evaluador era válido. Se corrigió el lector para inspeccionar todas las celdas de la matriz y la ausencia de conclusiones existenciales; el segundo runner pasó 8/8 con inputs idénticos y salidas idénticas. Esa primera incidencia también se conserva. Las tres pruebas de correlación son un addendum separado.

Los ocho controles principales tardaron aproximadamente 10–18 ms por proceso caliente; el primer arranque observado fue de 306 ms. Incluyen arranque CLI y no miden latencia de producto ni una comparación de rendimiento.

## Siguientes fuentes

La autoría aislada de Lino conserva dos lecturas con inventario abierto. El motor valida completas ambas hojas y mantiene parcial el sobre; no se llama fallo del extractor a la ambigüedad ni se cierra un inventario sin prueba. Sus incidentes de exposición accidental a tests genéricos están en el recibo, por lo que no se certifica aislamiento estricto sin tests. La representación no cuenta como entrada completa aceptada.

Los tres textos con «en Nacar» requieren conservar la preposición y su adjunción sin convertir Nacar en lugar físico, modo, régimen o propósito por conveniencia. La autoría/diseño fuente-only se conserva aparte; un carrier nuevo necesita crítica y controles antes de implementarse. Permanecen las seis deudas del conjunto y las puertas originales de los cinco pasos.
