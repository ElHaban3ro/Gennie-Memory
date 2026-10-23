# Admisión y premisa pendiente: contrato y mecanismo — 2026-10-22

La configuración candidata alcanza **28/39 evidencias útiles (71,8 %)** frente a **27/39 (69,2 %)**, con **cero selecciones ajenas de 60** y ninguna pérdida. La ganancia corresponde a Duna. Requiere el nuevo enlace y perfil explícitos del vocabulario; **no se atribuye solo al cambio del comparador**. Con los inputs y el contrato anteriores, el binario nuevo conserva los 100 informes idénticos y 27/39 útiles.

| Medición sobre los mismos 100 pares | Binario anterior, contrato original | Binario nuevo, contrato original | Binario nuevo, contrato candidato |
| --- | --- | --- | --- |
| Evidencias útiles | 27/39 | 27/39 | 28/39 |
| Selecciones ajenas | 0/60 | 0/60 | 0/60 |
| Respuestas completas | 0 | 0 | 0 |
| Selecciones solo condicionales | 1 | 1 | 1 |
| Pérdidas respecto a la base | — | 0 | 0 |

Quedan 11 omisiones bajo el contrato candidato. El ambiguo sigue separado y conserva su observación anterior; no se cambia su referencia. La puerta completa 39/39 continúa fallando y no se adopta en producto. La ganancia de retención 0→3/3 permanece separada y no se suma a este banco.

## Contrato implementado

`admission_request_context` es un perfil opcional `admission_request_context_v1` de evaluación, con filas y referencias al vocabulario verificadas. La primera declaración conecta `admits→property`, solicita `resolution/value` en `mode` y reconoce la premisa explícita `operates_mode` de esa pregunta. No establece equivalencia entre `property` y `quantity` ni convierte admisión en uso.

La ruta acepta una admisión negativa del modo solicitado o una restricción positiva cerrada, con existencia declarada, sobre los modos admitidos. Exige sujeto acreditado y restricciones compatibles, dominio lexical «modo» en el componente solicitado y todos los miembros/alias, ámbitos y dependencias aptos y condiciones restantes compatibles en ambos sentidos. La única premisa admitida como pendiente debe existir en la pregunta, tener la dependencia documental explícita y coincidir con el sujeto y modo de esa solicitud. Permanece sin satisfacer ni refutar.

Un `described_ref` solo permite una descripción positiva `quantity/resolution` de su propio referente, exacta conocida y sin condiciones ni ámbito normativo. Cualquier constraint adicional o no soportada bloquea la ruta conservando la fuente; no se elimina para comparar el referente base. No se transforma la cantidad descriptiva de Borrador en una resolución de Fino.

La salida `partial_admission_request_context` conserva el cuerpo fuente completo, ámbitos, calificadores y premisa original con su evidencia y dependencia. `answer_terms`, `answer_propositions` y `bindings` están vacíos. Permanecen deudas de admisión, dato solicitado, fundamento, premisa y membresía cuando corresponda. Todos los flags de uso, valor, satisfacción/refutación, equivalencia y exclusión inferidos son false.

El camino se reserva antes de la relevancia genérica: rechazar su perfil, descriptor o premisa no permite eludir las guardas mediante otro fallback. La configuración optativa no altera los inputs originales ni el comportamiento predeterminado del producto.

## Verificación y fallos preservados

- Formato, compilación y **729 pruebas aprobadas**, cero fallos y 24 ignoradas.
- Tres pruebas focales: dos ramas positivas y rechazos, 17 mutaciones inválidas del perfil y regresión de igualdad por alias. Dos controles de preparación inválidos se separan expresamente; no prueban rechazo semántico de descriptors o premisas válidos.
- Crítica independiente: **42 pares completos** —cuatro contextos parciales positivos y 38 negativos—; además un par parcial y tres inválidos separados. En total coinciden los 46 oráculos, sin presentarlos como 46 pruebas semánticas completas.
- **226/226 informes históricos idénticos** y 100 informes del contrato original idénticos en ambos binarios.
- Auditoría de Duna: selecciona únicamente la admisión `exclusive`, conserva descriptor/premisa/ámbitos/calificadores exactos, mantiene todas las deudas y no selecciona el claim de Loma. No publica 150 ni 600 como respuesta.

Se conservan un error de compilación de visibilidad, el RED semántico del stub y dos errores de preparación de fixtures. El validador no fue relajado para hacerlos pasar. Los controles críticos tuvieron reparaciones técnicas separadas antes de ejecutar el matcher, con manifiestos originales intactos; representan mutaciones estructurales, no nueva fidelidad de extracción.

La primera implementación pasó las 45 comprobaciones iniciales, pero falló un contraejemplo completo congelado antes de probarla: el modo principal de la pregunta tenía un alias igual a un miembro admitido. Se añadió una comprobación de igualdad acreditada por todo el grafo local antes de emitir la cautela. Unknown no se interpreta como Difference. La entrada y el resultado erróneo quedan intactos; los otros 45 informes son idénticos antes y después de esa corrección.

La revisión es de la misma familia Codex sobre desarrollo conocido. La fidelidad de un extractor y la generalización requieren validaciones posteriores; pasar forma, citas y guardas no las demuestra.

## Comparación, tiempos y coste

Los 100 inputs candidatos se derivaron antes del mecanismo añadiendo únicamente el enlace y el perfil declarados, más los hashes de vocabulario de los documentos. Se comprobó que texto, contenido semántico raw, alcance del par y referencias no cambiaron. No hubo nueva extracción. Los tres brazos alternaron su orden por caso.

| Tiempo CLI observado | Anterior/original | Nuevo/original | Nuevo/candidato |
| --- | --- | --- | --- |
| p50 | 18,185 ms | 17,882 ms | 18,331 ms |
| p95 | 29,634 ms | 37,130 ms | 28,572 ms |

Hubo verificaciones locales concurrentes. Estos tiempos describen el comparador y no acreditan mejora de recursos ni latencia extremo a extremo; excluyen extracción, crítica, red y producto. No se extrapolan a otra campaña.

**Cero llamadas externas nuevas; USD 0 externos nuevos.** Coste y tokens facturables Codex desconocidos. Acumulado externo conservado: 6.129 operaciones, 23 campañas, USD 4,565416508 conocidos y 20 operaciones de importe desconocido, sin Codex.

## Estado y evidencia

M5 y el objetivo siguen activos en paso 3. Falta completar las once omisiones y el recorrido fiel antes de comparar con producto, medir recursos/estabilidad y evaluar casos nuevos con revisión ciega. V6 permanece cerrada. README, dashboard, defaults, datos y credenciales no cambian.

Bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/admission-request-context-v1/`:

- `implementation/`: contrato, fixtures originales, aclaraciones técnicas y fuente anterior al arreglo de alias.
- `critical-review/final-critical-review.json`: controles, denominadores y límites; `before-v1/` conserva el fallo real.
- `known100-config-comparison/protocol.json`: inputs derivados, hashes y puertas congeladas.
- `known100-config-comparison/report.json`: los tres brazos, resultados y tiempos.
- `historical-compatibility/report.json`: 226 reproducciones idénticas.
- `root-verification/`: fuentes, fixtures, logs, verificación y auditoría de Duna.

Binario final SHA-256: `c622474a0f442643d6050ef632bbba02412bb097fddc9c6b2a20c975a4773034`.
