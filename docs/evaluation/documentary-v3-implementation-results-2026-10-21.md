# Representación documental V3: implementación local y límites

La representación V3 ya tiene validador y comparador puros en Rust, aislados en el evaluador. Pasan 562 pruebas del workspace, con 24 omitidas y cero fallos; 42 pruebas nuevas corresponden al mecanismo V3. **El preflight del dominio completo falla y no hay una nueva medición de calidad automática ni de recall.** El goal sigue activo en el paso 3 y baseline permanece vigente.

El [informe estructurado](../../outputs/iromem/evaluation/documentary-v3-implementation-2026-10-21/report.json), la [procedencia](../../outputs/iromem/evaluation/documentary-v3-implementation-2026-10-21/execution-provenance.json) y el [contrato local](documentary-v3-local-contract-2026-10-21.md) separan estos resultados. Los diseños, raws, críticas y fallos previos se conservan.

## Qué se implementó

`validate` comprueba objetos cerrados, hashes, citas por bytes UTF-8, roles tipados, referencias, ámbitos y dependencias. Su resultado validado no puede reconstruirse deserializando un JSON externo. El vocabulario desconocido de una fila deja pendientes esa fila y sus dependientes sin borrar hechos independientes válidos; un vocabulario global inválido invalida el documento.

`classify_pair` conserva apoyo, refutación, parcial, conflictos y candidatos pendientes. Distingue identidad conocida de ausencia de prueba, mantiene calificadores y premisas, hereda restricciones de alternativas y limita la relevancia a enlaces declarados. Los requisitos y dispensas devuelven condiciones tipadas y estado efectivo; una dispensa no demuestra permiso. No hay extracción, SQL, red ni cambios de producto en estas funciones.

La CLI añade `validate-documentary-v3` y `classify-documentary-v3`: archivos de entrada cerrados, límite de 16 MiB y salida nueva sincronizada sin sobreescribir resultados.

## Verificación real

| Comprobación | Resultado y alcance |
| --- | --- |
| `cargo test --offline --workspace` | 562 pasan, 24 omitidas, cero fallos; incluye los 42 controles V3 |
| Compilación del ejecutable y formato | `cargo build --offline -p gennie-memory-eval` y `cargo fmt --all --check` pasan |
| Control CLI con frames manuales | Fuente/pregunta completas; selecciona claim `c` y referencia `b` (Rango), con cita fuente |
| Validador CLI del mismo control | Completo |
| Vocabularios completos domain1/domain2 | Ambos inválidos por filas `class_quotes` vacías; entradas y reportes preservados |
| Historia | Cuatro bancos manuales y sus puntuaciones, diez informes de extracción, seis informes semánticos, tres auditores y cuatro informes Codex se reproducen byte a byte |

No se repitió la variante HTTP/2 esta tanda. No se midieron extracción automática, recursos de producción, estabilidad de modelo, recall ni comparación contra baseline. Los controles CLI usan representaciones escritas manualmente y un vocabulario mínimo separado: no son una reparación del banco completo ni una prueba de calidad textual.

Se preservan el fallo inicial de compilación y la tanda de 35 pruebas correctas/una fallida. La causa de esta última era exigir que cada rama duplicara calificadores heredados del padre; se corrigió sin relajar la expectativa. Las críticas sucesivas detectaron además deuda de guards/premisas, conflictos en resultados del mismo evento y metadatos normativos. El último hallazgo corregido era una dispensa que heredaba `allowed`; ahora publica conclusión nula y estados de requisito. La revisión estática final es acotada; no certifica ausencia de otros errores.

## Por qué todavía no se amplía

El inventario abarca los 116 textos, sin usar gold para definir vocabulario. Su revisión tuvo 13 observaciones; en el recheck una quedó resuelta solo en el perfil, nueve parcialmente y tres sin resolver, además de cinco hallazgos nuevos. El preflight ya declaraba fallida la puerta completa. La ejecución CLI añade evidencia de una incompatibilidad concreta: domain1 declara tres tipos sin citas de clase admisibles; domain2, cuatro. No se desactiva la validación para aceptarlos.

También faltan contratos ejecutables de estado documental sin documento inventado, acciones/participantes, relevancia neutral, compatibilidad de atributos y conteo. Las operaciones propuestas en `resolution-report.json` son notas: el código no las consume. No deben presentarse como implementadas por estar hash-vinculadas.

El mecanismo usa cantidades decimales literales sin convertir unidades; no implementa álgebra general de intervalos, calendario, sumas arbitrarias, conteo genérico hacia cualquier slot, sinonimia ni cadenas recursivas de reglas/relevancia. La cardinalidad implementada cubre patrones cuantificados y una consecuencia cerrada acotada, no todos los conteos del dominio.

La siguiente puerta es corregir forma y cobertura del vocabulario completo, demostrar cada operación necesaria con controles no vacuos y revisión independiente, y solo después repetir extracción completa y utilidad. Luego quedan conocidos126, baseline emparejada, recursos/estabilidad y cohorte nueva ciega. V6 solo se comprobó por hash de su manifest; no se abrió.

## Métricas y coste

La última medición automática sigue siendo **102/116 DTO válidos (87,93%) y 93/116 fieles en las cuatro dimensiones para ambos críticos originales (80,17%)**. Sus 928 juicios y 29 desacuerdos diagnósticos permanecen intactos. Es una puerta fallida; las pruebas unitarias nuevas no aumentan esos porcentajes.

Esta tanda: **cero llamadas a proveedores y USD 0 nuevos de API**. Tokens y coste de Codex desconocidos, no gratuitos. Acumulado: 6129 operaciones lógicas, 23 campañas emparejadas completas, USD 4,565416508 conocidos y 20 operaciones con coste desconocido; Codex excluido. No se declara ahorro, mejora de calidad o adopción.
