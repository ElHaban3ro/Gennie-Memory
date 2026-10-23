# Revisión prospectiva de la representación documental V3

El fallo del banco completo motivó una revisión del contrato antes de otra extracción. Se conserva el objetivo de calidad y el paso 3 sigue abierto. **No se implementó código Rust ni se midió una mejora de calidad en esta etapa.**

La propuesta inicial recibió diez hallazgos conceptuales. La siguiente crítica dio siete por resueltos, tres por parciales y añadió seis observaciones. Una segunda corrección recibió dos bloqueadores de tipos y una receta obsoleta. La última corrección recibió revalidación textual sin nuevos hallazgos concretos ni bloqueadores conocidos en ese alcance. Todos los borradores, críticas y resoluciones permanecen archivados; la revisión es del mismo sistema de modelos, no una prueba humana ni garantía de completitud.

La interfaz propuesta se reduce a dos funciones puras: `validate` conserva el documento y sus limitaciones; `classify_pair` devuelve apoyo, refutación, evidencia parcial, conflictos y candidatos pendientes por solicitud. Se retira la proyección V5 porque no tiene un consumidor necesario. Los históricos mantienen sus parsers; la futura comparación con baseline debe recorrer selección y composición reales.

Las correcciones tienen consecuencias operativas explícitas:

- Propietarios anónimos, cardinalidad y alcance de testigos no se aplanan; dos IDs distintos no prueban personas u objetos distintos.
- Cerrar una relación no cierra otra; limitar valores posibles no concede permiso individual. La exclusión exige desigualdad demostrada frente a cada miembro.
- Permiso, obligación, prohibición y ejecución se distinguen. Reglas, dispensas, restricciones y conclusiones conservan su ámbito normativo; los juicios incompatibles impiden respuesta completa.
- Cada evento conserva método y roles temporales. Diferencia literal de fechas no demuestra disyunción; no se traslada una fecha o método entre eventos.
- Clase, unidad y fundamento solicitado afectan la cobertura de la respuesta. Una cantidad útil puede conservarse como parcial sin afirmar que satisface una unidad o fundamento diferentes.
- Las ramas conservan calificadores completos. El resultado agregado contempla solicitudes completas mezcladas con otras pendientes, además del caso sin evidencia establecida.

La [rúbrica de revisión V2](semantic-review-v2-contract-2026-10-21.md) separa fidelidad/completitud frente al texto, preservación raw→normalizado y atribución de evidencia. Dos hallazgos textuales se corrigieron y revalidaron: incluir prohibiciones/solicitudes independientes en cobertura y exigir una omisión identificable antes de fallar completitud por unsupported. Ningún juicio histórico se reetiquetó. La rúbrica todavía necesita comprobación de comportamiento con controles nuevos.

Los catorce contrastes iniciales y quince revisados son especificaciones del autor, **no pruebas ejecutadas, etiquetas independientes ni una nueva exactitud**. Las nuevas objeciones también son contraejemplos conceptuales, no errores observados del producto.

## Métricas y coste conservados

La última medición sigue siendo la del banco completo: **102/116 DTO válidos y 93/116 entradas con cuatro juicios faithful de ambos críticos originales**. La puerta completa sigue fallida. No se volvió a ejecutar esa medición ni las 520 pruebas de la tanda anterior; no hubo cambios de Rust y se verificaron los diez hashes del evaluador registrados entonces. No hay nueva cifra de recall, latencia, estabilidad o recursos.

Cero operaciones y cargos de proveedor nuevos observados. Coste/tokens Codex desconocidos, no gratuitos. Acumulado: 6129 operaciones lógicas, 23 campañas completas, USD 4,565416508 conocidos más 20 operaciones con importe desconocido, sin Codex. V6 permanece cerrada; solo se verifica el hash de su manifiesto.

## Siguiente trabajo del paso 3

El vocabulario seed1 de la propuesta es limitado y no puede aprobar cobertura global. Antes de gastar otra campaña completa se debe inventariar el vocabulario del dominio desde fuentes y contratos de desarrollo, sin gold, respuestas, entidades ni aliases de casos; revisar sus filas genéricas y verificar representabilidad de todas las familias previstas. Después se implementan las dos funciones y se comprueban las consecuencias de cada restricción en controles prospectivos. No basta que un campo se serialice o quede en una cita lateral.

Solo tras esa comprobación corresponde repetir extracción y crítica completas, medir conservación de utilidad en los 126 pares conocidos y comparar el recorrido real con baseline, recursos y estabilidad. La cohorte nueva ciega y la adopción siguen condicionadas a esas puertas. La revisión del diseño no sustituye ninguna.

[Reporte estructurado](../../outputs/iromem/evaluation/representation-v3-design-2026-10-21/report.json), [propuesta revisada](../../outputs/iromem/evaluation/representation-v3-design-2026-10-21/proposal-revised3.json), [última crítica](../../outputs/iromem/evaluation/representation-v3-design-2026-10-21/proposal-recheck3.json), [verificación mecánica](../../outputs/iromem/evaluation/representation-v3-design-2026-10-21/mechanical-verification.json).
