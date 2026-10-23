# Requisito de fidelidad pendiente antes de medir transferencia automática

Se cruzaron los100pares manuales conocidos con las116extracciones y sus críticas ya congeladas, mediante tipo de texto y SHA256 exacto. Se recomputaron los cuatro juicios de ambos críticos por entrada y se verificaron hashes e integridad. No se alteraron raws, etiquetas, dictámenes ni perfiles, y no se llamó al clasificador.

El [protocolo congelado](documentary-v3-frozen-extraction-protocol-2026-10-21.md) exige superar fidelidad antes de evaluar utilidad automática. Esta auditoría observa la dependencia entre ambos pasos, sin ejecutar esa puerta antes de tiempo. Reutiliza críticas anteriores de contextos separados de la misma familia de modelos; no es revisión nueva, humana o de una familia independiente.

## Conteos derivados

| Pares conocidos | Ambos textos aceptados por los dos críticos anteriores | Algún lado sin aceptación conjunta | Total |
| --- | ---: | ---: | ---: |
| A | 9 | 20 | 29 |
| B | 4 | 3 | 7 |
| C | 1 | 2 | 3 |
| None | 24 | 36 | 60 |
| Ambigua | 0 | 1 | 1 |
| Total | 38 | 62 | 100 |

Entre los39pares útiles, **14** tienen fuente y pregunta aceptadas en las cuatro dimensiones por ambos críticos;25no cumplen ese requisito. Los36pares útiles seleccionados sobre frames manuales incluyen22cuyo equivalente automático congelado tiene algún lado sin aceptación conjunta. Esto impide atribuir esas36selecciones al extractor congelado.

Estos conteos **no son recuperación, precisión, predicción de rendimiento ni límite superior de recall**. Una crítica desfavorable no demuestra que toda la entrada sea falsa o inútil; una entrada aceptada tampoco garantiza selección correcta. Los pares comparten fuentes/preguntas y no son observaciones independientes. No se comparan97/100de concordancia manual con38/100de aceptación previa: son magnitudes distintas.

Los100pares abarcan83fuentes y33preguntas.93pares tienen ambos lados estructuralmente completos, pero solo38cumplen la aceptación conjunta de fidelidad. Se mantienen los resultados originales por entrada:112/116completas y65/116aceptadas por ambos críticos (45fuentes,20preguntas). Al menos un crítico rechaza fidelidad textual en43entradas, completitud en32y atribución en44; la preservación raw→validado recibió faithful de ambos críticos en las116entradas. Los defectos se solapan y no se suman como entradas distintas.

## Las tres omisiones restantes

Los dictámenes completos se conservan con sus discrepancias. Se describen problemas de las extracciones congeladas; los frames manuales actuales pueden diferir y no se les atribuyen automáticamente todos esos defectos.

1. **P2/Ópalo.** Ambos críticos de la fuente rechazaron convertir «usa» en una propiedad de perfil no declarada. Un crítico de la pregunta rechazó owned_by como interpretación fuerte de «de Ópalo». La guía semántica V2 ya pide uso neutral y asociación débil; todavía no hay una extracción nueva que pruebe esa corrección. El source manual actual ya conserva uses y no equivale al raw congelado.
2. **Malva.** Se registraron confusión entre sustancia/método y pérdida del alcance exclusivo del origen. Ambos críticos de la pregunta detectaron «de riego» conservado solo en la mención, sin restricción operativa. Hace falta representar propósito y procedencia completa antes de confiar en matching.
3. **Duna.** Un crítico cuestionó que el cierre de admisión alcanzara todos los objetos en lugar de solo modos; la discrepancia se mantiene. Ambos señalaron citas locales tomadas de la cláusula del otro dispositivo. La pregunta pasó ambas revisiones. Falta demostrar una representación del dominio cerrado y atribución local, sin borrar calificadores ni inferir uso efectivo desde admisión.

La siguiente acción es probar representabilidad y corregir la extracción con esos requisitos, después repetir la revisión completa conocida. **No se validará solo el subconjunto de14pares** ni se relajará el comparador para ocultar pérdidas de significado. Continúan pendientes las tres omisiones manuales, la puerta completa de fidelidad/utilidad, baseline, recursos/estabilidad y V6 ciega.

## Verificación y coste

Se recomputaron por separado los100joins y conteos desde los dictámenes originales. Cuatro controles mecánicos rechazan duplicado de texto, cambio del score conjunto almacenado, deriva de hash y texto sin correspondencia exacta. Los116inputs/reviews originales siguen vinculados y el manifest congelado permanece intacto. La guía V2 se enlaza por hash y pointers existentes en el registro de prioridades; no se afirma una mejora por haberla escrito.

El código y binario coinciden con los ya verificados en la tanda anterior de611pruebas y387replays. No se repitió Cargo para esta auditoría de artefactos y documentación. No hubo extracción, revisión semántica, clasificación o medición de latencia nuevas; tampoco se reactivaron subagentes. La última fidelidad automática sigue en65/116, sin cambiar.

Coste nuevo: **0peticiones API,USD0de proveedor**. Coste/tokens Codex desconocidos. Acumulado intacto:6129operaciones,23campañas,USD4,565416508conocidos más20importes desconocidos,excluyendo Codex. El objetivo sigue activo en el paso3;sin adopción ni apertura de V6.

[Protocolo de auditoría](../../outputs/iromem/evaluation/documentary-v3-fidelity-readiness-2026-10-21/protocol.json), [cruce completo](../../outputs/iromem/evaluation/documentary-v3-fidelity-readiness-2026-10-21/report.json), [dictámenes de las tres omisiones](../../outputs/iromem/evaluation/documentary-v3-fidelity-readiness-2026-10-21/pending-pair-frozen-reviews.json), [prioridades y guía](../../outputs/iromem/evaluation/documentary-v3-fidelity-readiness-2026-10-21/triage.json) y [verificación](../../outputs/iromem/evaluation/documentary-v3-fidelity-readiness-2026-10-21/verification.json).
