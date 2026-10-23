# Retención: normalización y contexto parcial — 2026-10-22

El piloto de retención pasa de **0/3 a 3/3 evidencias útiles**, sin seleccionar el caso ajeno (**0/1**) y sin producir respuestas completas. La comparación del banco completo con sus extracciones anteriores conserva **24/39 útiles y cero ajenos/60**, sin pérdidas. No se suman ambos resultados ni se declara 27/39: la nueva extracción solo se ejecutó en un piloto conocido, y las 100 entradas congeladas siguen usando sus frames anteriores. La puerta completa continúa fallando; M5 y el objetivo siguen activos.

| Comprobación | Resultado |
| --- | --- |
| Piloto de normalización: textos completos y fieles para ambos críticos | 8/8 |
| Utilidad con nueva normalización y comparador anterior | 0/3; fallo conservado |
| Mismos frames normalizados y comparador nuevo | 3/3 útiles; cero ajenos/1 |
| Respuestas completas en el piloto | 0 |
| Banco completo, mismos frames anteriores | 24/39→24/39; cero ajenos/60; cero pérdidas |
| Controles críticos del mecanismo nuevo | 49/49: tres parciales y 46 negativos |
| Pruebas workspace | 725 aprobadas, cero fallos, 24 ignoradas |
| Compatibilidad histórica | 226/226 informes idénticos |

## Causa y corrección

Había dos causas distintas. La normalización anterior representaba de forma incompatible el pasado «fijaba», el archivo crudo y el alcance universal. La guía prospectiva añade el modificador literal `terminal_operation`, la clase registrada `raw_file` y conserva tiempo, cotas y dominio. No afirma que el borrado se haya ejecutado ni que los miembros de una colección existan. Una crítica corrigió antes del piloto la diferencia entre plazo exacto, mínimo y máximo.

Autores separados extrajeron seis fuentes y dos preguntas sin contrapartes ni etiquetas. Cuatro críticos nuevos, separados por fuente/pregunta, revisaron fidelidad: ocho textos aceptados en las cuatro dimensiones por ambos críticos correspondientes. Las seis fuentes necesitaron una reparación técnica de dependencias de binding; las salidas originales quedan intactas y el diff acredita que no cambió el contenido semántico. Las dos preguntas pasaron al primer intento. Es desarrollo conocido y revisión de la misma familia Codex, no una cohorte reservada ni revisión humana.

Aun con frames fieles, el comparador anterior recuperó cero de tres útiles. La nueva ruta experimental conserva la proposición normativa como contexto cuando las referencias anónimas a la política comparten clase registrada y anclajes restrictivos compatibles. Comprueba también identidad del resto de referentes, condiciones, alias, dominio y alcance. Una afirmación individual no se convierte en plazo común universal. No equipara las políticas: conserva `identity`, `normative_target` y `answer_slot` pendientes; añade `domain_membership` cuando corresponde. Los términos de respuesta, proposiciones de respuesta y bindings permanecen vacíos; las proposiciones, qualifiers y scopes fuente se preservan exactamente.

La crítica detectó estáticamente dos riesgos de restricciones aportadas por alias de variable/dominio. Se corrigieron antes del binario final y los contraejemplos válidos quedaron congelados. No se afirma haber reproducido una selección errónea binaria previa para esos dos riesgos. Los 49 controles pasan; los positivos siguen siendo contextos parciales y los 46 negativos no seleccionan recuerdos. Algunos controles de fundamento son estructurales y no acreditan fidelidad de extracción.

Además se extendió a `description` en contexto V2 la comprobación bidireccional de vínculos de admisión ya usada por `capability`. El fallo de propietario distinto se reprodujo antes de corregirlo. La revisión separada comprobó 14 casos válidos en contexto y compatibilidad del adaptador histórico; el fixture cíclico inválido se conserva separado. Esta corrección de seguridad no aumentó el recall del banco completo.

## Coste, tiempos y límites

**Cero llamadas externas nuevas; USD 0 externos nuevos.** Coste y tokens facturables Codex desconocidos. El acumulado externo sigue en 6.129 operaciones, 23 campañas, USD 4,565416508 conocidos y 20 operaciones con importe desconocido, excluyendo Codex.

La comparación de los mismos 100 inputs observó p50 CLI 26,717→26,667 ms y p95 52,023→53,394 ms. Hubo verificaciones locales concurrentes; son tiempos descriptivos, no una prueba de mejora de recursos ni latencia extremo a extremo. Excluyen extracción, revisión, red y producto.

La ruta nueva pertenece únicamente a evaluación. No permite referencias normativas nombradas, anclajes sin nombre propio aunque exista un alias nombrado, referencias pendientes ni restricciones no demostradas. La reserva V6 no se abrió. No se cambió el comportamiento predeterminado del producto, el README, dashboard, credenciales o datos del usuario.

El siguiente trabajo necesario es completar la normalización prospectiva de las familias todavía omitidas y ejecutar el recorrido completo con una guía congelada; después, y solo tras pasar la puerta local, comparar con producto, medir recursos y usar casos nuevos con revisión ciega. No se declara terminado el objetivo mientras falten esas pruebas.

## Evidencia

En `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/`:

- `retention-normalization-v1/`: diseño, crítica, guía corregida, autores, reparación técnica y cuatro críticas ciegas.
- `retention-normalization-v1/pilot-v1/pair-report-v1.json`: resultado negativo 0/3 intacto.
- `anonymous-normative-context-v1/normalized-pilot-comparison/report.json`: mismos frames, mejora 0→3/3.
- `anonymous-normative-context-v1/critical-review/review.md`: 49 controles y límites.
- `anonymous-normative-context-v1/known100-comparison/report.json`: banco completo sin cambios de inputs.
- `anonymous-normative-context-v1/root-verification/`: fuentes y logs finales.
- `admission-description-context-v1/`: fallo previo, corrección y controles de admisión.

Binario final SHA-256: `0b57ca055512defecf5d8aa8d891ef17ae3cad565404026cba3b1e1f2344989d`.
