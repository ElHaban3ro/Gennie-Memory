# Entrega semántica V3 separada del ensamblado

Se implementó en Rust un adaptador de evaluación que separa decisiones semánticas de empaquetado, hash y publicación. Responde a la causa mecánica del lote ausente de la extracción anterior; no rescata ese lote ni modifica sus resultados. Los 12 textos ausentes siguen ausentes y el paso 3 del objetivo permanece activo.

El modelo entrega un archivo `documentary_v3_semantic_items_v1` con `input_packet_sha256` copiado del paquete congelado y todas las filas `id/raw` en orden. El coordinador comprueba el paquete y su inventario, conserva íntegros los bytes UTF8 originales y sus valores JSON, calcula hashes y genera una entrega derivada. No rellena metadatos, convierte Proposition en Atom ni elimina unsupported. El recibo vuelve a comprobar todo; la opción `--assembled-only` exige el formato nuevo y evita aceptar una degradación al formato histórico.

Se rechazan claves duplicadas en cualquier profundidad, números JSON fraccionarios o fuera de rango, cambios de paquete/orden/inventario y derivados alterados. Las cantidades decimales documentales siguen siendo strings. Cada entrada y salida está limitada a 16 MiB. Los errores mantienen el denominador en el registro de campaña: los controles CLI incluyen recibos con todas las filas indisponibles cuando falta el derivado.

## Verificación ejecutada

| Comprobación | Resultado |
| --- | --- |
| Suite offline del workspace | 627 pasan; 24 omitidas; 0 fallan |
| Adaptador V3 / motor V3 dentro de la suite | 16 / 91 pasan |
| Controles CLI de ensamblado, fallos y manipulación | 17/17 |
| Informes históricos reproducidos idénticos en bytes | 226/226 |
| Compilación y formato | pasan |

Los 226 son 100 pares manuales, 116 validaciones de la extracción anterior y diez recibos de la extracción estricta. Comparador y validador semánticos tienen hashes idénticos a los verificados antes del adaptador. Las pruebas de publicación cubren fallo parcial de escritura, destino existente y symlink; nunca reemplazan archivos ajenos. Una primera compilación de prueba fallida quedó registrada. El primer script CLI también falló: dos filas compartían el mismo objeto Python y la mutación intencional afectaba a ambas. Se conservó el intento y se corrigió el aislamiento del fixture en un segundo directorio; no hubo cambio de producto para aprobarlo.

La publicación escribe y sincroniza un temporal exclusivo y publica por hard link sin reemplazo. Un fallo al limpiar el temporal después de publicar puede devolver error con el destino completo ya presente; debe registrarse e inspeccionarse sin reintento ciego. Un cierre abrupto puede dejar un temporal y no hay garantía de durabilidad del directorio frente a caída. Estos límites no se presentan como publicación transaccional de campaña.

## Calidad y coste

No hubo nueva extracción automática ni revisión de sus contenidos, utilidad automática, comparación con baseline, recursos de producción o V6. La última extracción sigue con 101/116 completas estructurales y 98/116 aceptadas en cuatro dimensiones por ambos críticos: puerta global fallida. La utilidad manual histórica sigue en 36/39 no ambiguas, cero selecciones en 60 none y tres respuestas completas. Los 17 controles no sustituyen esas métricas.

El fixture positivo ocupa 25.024 bytes semánticos y 52.964 bytes derivados, porque se conservan originales y valores. Son bytes de archivo local, no tokens ni ahorro remoto. No se midió latencia del producto ni mejora de velocidad de extracción. Cero llamadas de proveedor y USD 0 nuevos de API. En el trabajo de diseño previo al cierre de agentes participaron dos contextos Codex y cuatro turnos de revisión; coste y tokens desconocidos. No se lanzaron más agentes tras su cierre. El acumulado de proveedor permanece en 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos; excluye Codex.

La revisión de transporte fue estática, sin ejecución independiente de pruebas ni aprobación semántica. El hash repetido por el modelo y la igualdad de originales no acreditan autoría, frescura ni conducta fuente-only: esos tres indicadores permanecen false. Una campaña real necesita su registro independiente de despacho/recepción; el ensamblador por sí solo no detecta repetición del mismo paquete.

## Siguiente causa a resolver

La revisión estática de capacidad encuentra una carencia general en miembros de conjuntos cerrados con propiedades específicas. Las condiciones históricas ya pueden conservarse con calificadores temporales/contextuales; algunos descriptores bajo negación caben en guards existentes. No procede añadir propiedades globales a Reference que filtren condiciones entre menciones o tiempos.

Se propone estudiar un término local descrito con restricciones no promovidas a hechos y resolución explícita. Es diseño pendiente, sin extensión ni matcher implementados. Antes de otra extracción hay que especificar y verificar consumidores, negación, bindings, restricciones de valor, testigos y degradación conservadora con controles genéricos. Después corresponde una versión congelada sobre los 116 textos completos y revisión separada antes de utilidad automática. No se cambia gold ni se entrena el comparador para aceptar raws defectuosos.

Artefactos y hashes: `outputs/iromem/evaluation/documentary-v3-semantic-handoff-2026-10-21/verification.json`. Se conservan originales, críticas, intento fallido, controles, replays y manifiesto. Sin adopción ni cambio de política predeterminada.

## Compatibilidad de paquetes de guía — 2026-10-22

La recepción admite dos formas cerradas del paquete congelado: la histórica con `validator_contract_source_without_tests` y la actual con `author_guide`. No admite mezclar ambos campos ni agregar etiquetas o campos ajenos. El hash sigue ligando todos los bytes del paquete, incluida la guía; no se modifica el raw. La nueva prueba verifica recepción íntegra, rechazo de campos adicionales y cambio de guía contra hash previo. Verificación actual: 730 pruebas de workspace aprobadas, 24 ignoradas, 35 focales de IO y formato correcto. Esta compatibilidad no acredita fidelidad semántica de la tanda en curso. Evidencia: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/consolidated-source-extraction-v1/packet-receiver-fix-verification.json`.
