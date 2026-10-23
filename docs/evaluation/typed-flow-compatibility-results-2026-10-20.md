# Compatibilidad de flujo: diagnóstico local y discrepancia retrospectiva

Fecha UTC: 2026-10-20. Objetivo activo, sin candidata promovida. Este diagnóstico no mide recuperación automática ni generalización.

Al separar hechos de relaciones apareció una ambigüedad en `pcv1_08_01/claim_00`. La afirmación hipotética «Olmo recibe agua exclusivamente del depósito de lluvia K» puede restringir el proveedor inmediato sin excluir que agua de red atraviese K. El banco congelado asigna B a su utilidad condicional; no se modifica esa etiqueta ni se recalculan métricas. `grounded=false` sigue correcto: la fuente no afirma exclusividad.

La revisión ciega de un subagente, sin etiquetas ni resultados, dio A/B/A para exclusividad inmediata, negación explícita de red y alimentación no exclusiva. Otra revisión crítica, retrospectiva y con historia visible, admite el contraejemplo pero señala la lectura pragmática alternativa de exclusión de todo origen de red. Son revisiones de la misma familia de modelos, sin revisión humana. Por eso registramos alcance no resuelto, no una corrección indiscutible de B a A. [Revisión ciega](../../outputs/iromem/evaluation/typed-resolver-sufficiency-v1-2026-10-20/blind-flow-review.json), [crítica](../../outputs/iromem/evaluation/typed-resolver-sufficiency-v1-2026-10-20/adversarial-flow-review.json).

Se implementó un comprobador puro de grafos finitos, aislado en evaluación. No recibe roles ni IDs de benchmark en sus reglas. Recibe nodos, transferencias efectivas, fuentes inmediatas permitidas y restricciones de procedencia; devuelve testigo compatible, ausencia de modelo en el universo finito o límite de búsqueda no cubierto. Referencias inválidas se rechazan. La entrada declara manualmente que los arcos transportan procedencia durante el mismo período: una conexión física por sí sola no basta. No hay traducción automática del español.

| Teoría formalizada junto con premisa de procedencia de red | Resultado local |
| --- | --- |
| K único proveedor inmediato de Olmo | Compatible: R → K → Olmo |
| Negación explícita de toda procedencia de red | Sin modelo en este universo |
| K alimenta Olmo, sin exclusividad | Compatible: R → K → Olmo |
| K único proveedor y axioma adicional que excluye red de K | Sin modelo en este universo |

El último control añade una premisa que no está en la primera afirmación. La ausencia de modelo no demuestra imposibilidad en cualquier universo ni adjudica automáticamente B. Un mundo compatible refuta la incompatibilidad estricta bajo la lectura formalizada; no demuestra respaldo de la afirmación. [Entrada y axiomas](../../outputs/iromem/evaluation/typed-resolver-sufficiency-v1-2026-10-20/input.json), [resultado ejecutado](../../outputs/iromem/evaluation/typed-resolver-sufficiency-v1-2026-10-20/report.json).

Verificación: 299 pruebas pasan con configuración predeterminada y con HTTP/2 optativo; formato y compilación pasan. Diez pruebas nuevas comprueban testigo, exclusividad directa, exclusión explícita, premisa adicional, renombrado/orden, ciclos, restricciones contradictorias, entradas inválidas, límite de búsqueda y documentación de interpretación obligatoria. La crítica estática detectó que faltaba exigir la interpretación en el adaptador; se corrigió con una regresión. También se limitó a tests un import de `json` que generaba aviso de compilación. No hay cambio de producto.

Coste nuevo de proveedor: USD 0; llamadas nuevas: 0. Acumulado conservado: 5.881 operaciones, 23 campañas completas, USD 0,582095598 conocidos más cinco operaciones de coste desconocido. Excluye programación y revisión de Codex; no es coste total del proyecto. No se midió una mejora de latencia ni del selector en este diagnóstico.

Siguiente obligación: completar la gramática y la prueba de suficiencia de 100 pares, con cuantificación, modalidad, polaridad, condiciones, parciales, negaciones y contraste. Separar proveedor inmediato de origen; representar ambigüedad como falta de cobertura y revisar un futuro banco antes de llamadas. No introducir un rol oculto en los hechos. Después siguen pendientes extracción automática, recorrido completo, recursos, estabilidad y cohorte nueva revisada a ciegas. V6 conserva sus hashes y cero llamadas; baseline sigue vigente. Los cinco pasos no se han completado.
