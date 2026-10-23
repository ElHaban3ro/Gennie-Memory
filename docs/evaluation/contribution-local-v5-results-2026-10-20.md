# Fuentes por pregunta: recursos aprobados, calidad pendiente

Fecha UTC: 2026-10-20. La representación local **pasa coste y latencia en los 18 pares**, pero falla calidad. El diagnóstico posterior no elimina sus errores al enviar un solo juicio por petición. `baseline` sigue vigente; no se incorpora una política nueva ni se abre V6. El objetivo permanece activo en el paso 3.

## Mecanismo y comprobación mínima

Cada pregunta contiene ahora su fuente literal entre comillas; el estado común deja de incluir el mapa de recuerdos. Se conservan las definiciones A/B/C/none y las ocho analogías, codificadas como tripletas sin explicaciones redundantes. Las reglas se compactan y precisan asociación de sujeto/atributo, chequeo genérico frente a medida específica y lectura temporal ambigua. Es una hipótesis conjunta de formato e instrucción, no una ablación que pruebe una causa interna del proveedor.

El banco mínimo conserva **64/64 roles correctos, 17/17 contribuciones y cero selecciones ajenas**. Su coste baja de USD 0,001451520 a **USD 0,000964992**, un **33,52 %** menos, con 22.976 frente a 34.560 tokens de entrada. Son tandas conocidas distintas, no validación nueva ni contraste contemporáneo de latencia. No se cambiaron preguntas, textos, IDs, orden o etiquetas.

Se sustituyeron combinaciones de booleanos internas por perfiles explícitos, manteniendo exactamente los payloads y reportes anteriores. Las pruebas comprueban claves/tipos coherentes y que cada instrucción solo incluya su fuente propia. Esto verifica la petición, no demuestra independencia dentro del servidor.

## Comparación difícil emparejada

Mismos 18 casos y 126 roles canónicos conciliados antes de red; orden alternado, cliente reutilizado y 36/36 respuestas HTTP/2. Base real Decisions con hasta20 candidatos y candidata Jev1.13 con7. No hubo selección fuera de los pares revisados. Se conservan la concordancia histórica y sus discrepancias por separado, sin recalcular resultados previos.

| Métrica en desarrollo difícil | Base | Local V5 |
| --- | ---: | ---: |
| Contribuciones útiles conservadas | 11/17 | 16/17 |
| Selecciones ajenas revisadas | 1 | 5 |
| Errores de rol canónico | No clasifica roles | 6/126 |
| Mediana de clasificación | 533,22 ms | 298,25 ms |
| p95 de clasificación | 1.058,08 ms | 353,69 ms |
| Coste informado | USD 0,001119900 | USD 0,001374660 |

Coste/mediana/p95 relativos: **1,227× / 0,559× / 0,334×**, los tres dentro del máximo1,5×. Es el primer pase de recursos de esta línea de contribución, no una mejora completa. Los tiempos abarcan petición, lectura, JSON y validación, no `recall` ni respuesta final. El p95 con18 por rango más próximo es el máximo; no se retiraron peticiones lentas. No se deduce estabilidad de una corrida.

La candidata conserva las once contribuciones útiles de la base y gana cinco, pero acepta cinco recuerdos ajenos y omite uno permitido. La concordancia canónica es120/126 y14/18 escenarios exactos. Los errores persisten en archivo de prueba frente a política M4, Neblina frente a Luciérnaga, comprobación genérica/cita futura frente a autonomía y carné frente a avalista. Recupera el depósito parcial de Laurel, pero pierde la hora de funcionamiento que la variante compartida sí seleccionaba. Esas ganancias no compensan pérdidas ni aceptaciones indebidas.

Incluso frente al gold histórico original conserva cuatro selecciones no permitidas; una corresponde al depósito ahora anotado como parcial. Los desacuerdos de etiquetas no vuelven elegible esta variante y no se atribuyen a falsedad de los textos fuente: se evalúa su selección como evidencia del atributo pedido.

## Diagnóstico controlado de preguntas hermanas

Se fijaron antes de la nueva red **los seis desacuerdos** más tres controles útiles correctamente clasificados: nueve juicios en cuatro casos. Se enviaron cuatro lotes completos contemporáneos y nueve peticiones de una sola pregunta. Cada petición individual mantiene estado, modelo, instrucción, criterios y clave original; solo retira las preguntas hermanas. El auditor normaliza únicamente una clave realmente preguntada/respondida y rechaza una sustitución de namespace de igual tamaño. Los cuatro recibos completos se cobran una vez aunque aporten varios juicios a la comparación.

Resultado de13HTTP, sin errores técnicos:

- Lote completo nuevo: **5 errores canónicos entre los nueve juicios seleccionados**.
- Pregunta individual: **6 errores entre esos mismos nueve juicios**; los seis desacuerdos del piloto original reaparecen.
- Los tres controles útiles continúan correctos.
- Un juicio del lote completo varió respecto del piloto: recuperó la hora de funcionamiento. Esa variación no se usó para reemplazar el fallo anterior. Entre lote nuevo e individual solo cambió ese juicio, desfavorablemente para el individual.

Retirar preguntas hermanas no es una solución suficiente. Los positivos indebidos persisten aun cuando no se envían las otras fuentes del caso; esto debilita la explicación de mezcla entre recuerdos como causa única. No prueba qué componente interno falla ni que ninguna instrucción pueda ayudar. Tampoco califica siete llamadas individuales por consulta, ni su coste. No se combinaron mejores respuestas entre tandas.

La siguiente prueba debe contrastar capacidad semántica entre familias con contrato y bancos congelados, antes de seguir afinando referencias o añadir verificaciones. Cada familia tendrá que conservar el mínimo y superar el contraste difícil con sus recursos medidos. No se abre V6 ni se baja la puerta de calidad a cambio del ahorro conseguido.

## Contabilidad y verificación

Esta tanda: **65 operaciones / 65 HTTP, USD 0,004184094**, sin importes desconocidos ni llamadas inciertas. Desglose: mínimo16/USD0,000964992; contraste36/USD0,002494560 incluyendo controles; diagnóstico13/USD0,000724542. No se vuelven a cobrar recibos del piloto al reutilizarlos como origen del diagnóstico.

Acumulado del objetivo: **5.506 operaciones, 23 campañas emparejadas completas, USD 0,564875606 conocidos +4 operaciones históricas de importe desconocido**. La programación/revisión de Codex queda fuera de esa cuenta; los tokens registrados pertenecen al proveedor remoto.

**243 pruebas locales pasan con y sin HTTP/2**, formato y compilación pasan. Se reprodujeron exactamente los tres informes nuevos, los cuatro informes mínimos anteriores y el contraste compartido anterior. Se conservaron los bancos originales, las ambigüedades de la conciliación, los fallos y todos los recibos. V6 mantiene todos sus hashes y cero llamadas. No se iniciaron servicios, crearon conexiones SQLite ni modificaron procesos del usuario.

Artefactos: `outputs/iromem/evaluation/contribution-local-v5-minimal-2026-10-20`, `contribution-local-v5-known18-paired-2026-10-20`, `local-isolation-diagnostic-v1-2026-10-20` y `contribution-local-phase-summary-2026-10-20.json`. Cierre de huellas/contabilidad: `quality-goal-2026-10-19/snapshots/037`. El README conserva métricas y gráficas de su ruta vigente, sin presentar este desarrollo fallido como validación general.
