# Composición de medición con «hasta»: resultados locales

El paso 3 avanza de **113 a 115/116 representaciones completas y fieles** en el banco conocido corregido manualmente: **97,4 % → 99,1 %**. Se aceptan las correcciones de Lino y Cobre después de doble crítica fuente y validación real. Queda la credencial. El objetivo de cinco pasos sigue activo; esta tanda no mide extracción autónoma, rendimiento de recuperación ni una política instalada.

## Cambio y alcance

Se implementó el contrato [congelado antes del código](documentary-v3-until-measurement-design-2026-10-22.md): feature optativo undécimo `linguistic_composition_until_measurement_v1`, Body `linguistic_composition`, constructor `negative_se_measurement_with_until_v1`. Conserva como una unidad la medición negada, su descripción de duración y el límite «hasta apagarse», sin inventar adjunción, sujeto del infinitivo, apagado efectivo, medición posterior o duración. El posesivo queda abierto; no se atribuye a un nombre cercano.

Los nodos internos del AST no son hechos globales ni predicados nuevos del vocabulario. El validador comprueba forma cerrada, IDs internos, incidencia, orden y evidencia UTF-8. Grafo, canonicalización, presupuesto compartido y corte de consumidores efectivos/aliases conservan el compuesto antes de las rutas ordinarias y la proyección existencial. La evidencia escalar cuenta también cuando el feature léxico está ausente.

El clasificador contextual existente selecciona contexto parcial íntegro para una solicitud compatible de duración con dueño explícito acreditado. Retiene documentos, condiciones, cierre y dudas; sus arrays de respuestas y bindings quedan vacíos. Deudas externas de cobertura en el cierre de fuente o pregunta bloquean la selección; una deuda independiente no la elimina. Una respuesta completa realmente independiente mantiene completitud y proyección; un consumidor del compuesto abierto sigue bloqueado.

Se corrigió además un fallo pequeño del selector léxico anterior: `Term.quantity.unit` es string o null y se comparaba como objeto `Unit`. La comparación ahora respeta el contrato de Term; el tipo de `Request.unit` conserva su contrato. Una prueba del selector completo cubre unidad igual, distinta, ambas ausentes y ausencia de un solo lado.

Solo cambia evaluación. No se añadieron dependencias, comandos CLI, servidores, credenciales ni política predeterminada del producto.

## Evidencia y controles

| Comprobación actual | Resultado | Qué acredita |
| --- | ---: | --- |
| Pruebas de composición | 15/15 | Validación, selección parcial, contexto, consumidores, independencia y proyección |
| Prueba del selector de unidades | 1/1, cuatro variantes | Comportamiento observable con Term válido |
| Suite offline del workspace | 842 aprobadas, 0 fallos, 24 ignoradas | Código actual; 16 pruebas nuevas respecto a 826 |
| Controles CLI propios del coordinador | 18/18 | Casos distintos de los fixtures del implementador |
| Replays históricos sin feature nuevo | 403/403 idénticos byte a byte | Compatibilidad de informes, incluidos los 116 actuales y el selector léxico |
| Validaciones fuente Lino/Cobre | 2/2 completas | Estructura admitida por el binario final |
| Doble crítica fuente | 16/16 dimensiones fieles | Dos contextos nuevos, misma familia Codex; no revisión humana |

Los 18 controles se desglosan en nueve pares con DTO completos, cinco entradas inválidas deliberadas, un agotamiento del presupuesto global y tres casos de deuda externa parcial. Los negativos semánticos no se cuentan como aprobados por tener DTO inválidos. Incluyen selección positiva íntegra, otro dueño, atributo sin relevancia inventada, posesivo no resuelto, configuración requerida, límite no convertido en hecho, respuesta independiente y consumidor bloqueado.

El autor recibió fuentes y contrato, sin preguntas ni gold. Los críticos recibieron solo el paquete congelado con textos, raws, vocabulario y semántica declarada, sin pruebas, resultados o dictamen ajeno. La auditoría verificó 16 citas por offsets y 79 punteros. Es corrección manual conocida con crítica separada, no extracción autónoma ni validación independiente de familia.

Paquete de revisión: SHA-256 `aa850312c896942577f10c77b338fc82eadb0f7b411e9064885eae37f51af471`. Binario final: `9187d04964769470f9d6f52a7edce4ee51cb7e7a9eafdca5af544d39c207f80f`. Resultados e integridad: `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/until-composition-contract-v1/` y `until-composition-source-corrections-v1/`.

## Fallos conservados y reparación

La primera prueba focal tuvo tres fallos por fixtures: dos cantidades usaban objeto Unit donde corresponde string; una pregunta cambiaba a truth sin completar su forma obligatoria. Se corrigieron los fixtures manteniendo validación completa y expectativas de selección. La segunda ronda pasó 14/14.

La primera ejecución de los controles del coordinador pasó 16/18. Un control de configuración tenía una arista con consumidor qualifier no admitido; se corrigió en una versión nueva. El otro encontró un fallo funcional: una respuesta directa válida de siete minutos quedaba incompleta por el candidato abierto de la composición. La reparación calcula primero la prueba ordinaria con todas sus deudas; después adjunta el contexto parcial. Las dudas del compuesto siguen en sus testigos y procedencia global, sin contaminar una prueba independiente ni liberar a sus consumidores.

La prueba adicional de proyección tuvo un error de compilación por ubicación de módulo y luego detectó un fixture inválido: el helper de rehash insertaba `readings:null` en una pregunta plana por acceso mutable. Se corrigieron ubicación y helper, se exigió validación completa de ambos lados y finalmente pasaron 15/15. Se conservan las cinco rondas focales, los 18 controles iniciales y sus informes; no se reemplazan por los éxitos finales. La [crítica técnica](documentary-v3-until-composition-code-review-2026-10-22.md) distingue sus reparaciones estáticas de las pruebas ejecutadas por el coordinador.

## Coste y límites vigentes

Esta tanda ejecutó 443 operaciones CLI de evaluación, nueve procesos Cargo y tres de formato, cinco contextos Codex nuevos y cuatro reanudaciones de tareas. **Cero llamadas a proveedores de modelos; USD 0 de coste API nuevo.** El coste de Codex no está cuantificado. El histórico externo conserva 6.129 operaciones, USD 4,565416508 conocidos y 20 operaciones sin coste conocido; no es un coste completo ni incluye Codex.

Los tiempos por control son instrumentación de desarrollo con entradas de categorías distintas. No prueban latencia o recursos de producción, ni hay comparación nueva contra la configuración vigente. La extracción autónoma original **99/116** y la utilidad histórica **31/39** siguen intactas. No se abrieron V6, ni se actualizaron gráficas del README con métricas manuales.

El banco mixto vigente queda en 115 completas, 115 con ambos críticos fieles y 115 con ambas condiciones. La credencial conserva su representación anterior parcial: su candidata V2 estructuralmente completa no recibió aceptación integral y no se sustituye para aparentar cierre. La [nota sobre transporte contextual](documentary-v3-credential-context-transport-analysis-2026-10-22.md) concreta la evidencia pendiente: entrada, salida, mapa de origen y controles de no colisión y no reinterpretación. Todavía no se ejecutó ese expediente.

Después de cerrar esa causa quedan la extracción coherente de los 116 con guía/vocabulario compartidos y doble revisión, utilidad y contraste emparejado con la base, recursos, estabilidad y casos nuevos ciegos. Se preservan las puertas originales; el objetivo no está terminado ni el mecanismo promovido.
