# Preparación local de LoCoMo

LoCoMo es un benchmark de conversaciones largas entre agentes LLM con anotaciones para preguntas y respuestas y resumen de eventos. No es un conjunto de recuerdos personales reales ni una validación humana de la función `mem-write`. La licencia del repositorio es CC BY-NC 4.0; el corpus se mantiene como submódulo Git y los datos preparados se guardan localmente, fuera del control de versiones.

## Fuente y alcance

- Repositorio: [snap-research/LoCoMo](https://github.com/snap-research/LoCoMo), submódulo `LoCoMo/` fijado en `3eb6f2c585f5e1699204e3c3bdf7adc5c28cb376`.
- Archivo usado: `LoCoMo/data/locomo10.json`; no se importan personas ni otros archivos del repositorio.
- Cobertura publicada: diez conversaciones, 272 sesiones y 1.986 preguntas anotadas.
- El diálogo es generado con agentes LLM; los resúmenes de sesión incluidos por los autores también son generados. Las QA/evidencias son anotaciones del benchmark. No se presentan como recuerdos humanos espontáneos.

## Preparación e importación

Ejecutar desde la raíz del repositorio:

```sh
python3 tools/datasets/prepare_locomo.py
```

`tools/datasets/prepare_locomo.py` transforma cada sesión en una solicitud `mem-write`: conserva los turnos con sus hablantes en `original_text` y usa el `session_summary` publicado como `summary`. Escribe 272 recuerdos tipo `event`, en el ámbito de su conversación, con referencia y localizador a la revisión exacta de la fuente. Los identificadores de operación son UUIDv5 deterministas; se puede repetir la importación contra la misma base sin duplicar operaciones.

La ruta de escritura invoca `mem-write-batch` offline. No llama a modelos, proveedores externos ni APIs pagadas, y no reescribe el diálogo. La base y los artefactos se escriben en `.local/locomo/`:

- `write-validation.db`: base con los recuerdos importados.
- `mem-write-requests.jsonl` y `mem-write-receipts.jsonl`: solicitudes y recibos de la ruta real `mem-write`.
- `memory-corpus.json`: proyección reproducible para los motores comparadores, enlazada por `memory_id` confirmado y con texto original, resumen y ámbito; su hash también se guarda en el manifiesto.
- `all-questions.jsonl`: inventario íntegro de preguntas, respuestas/anotaciones originales y estado de evidencia.
- `search-cases.jsonl`: casos con evidencia completa o sin evidencia anotada, con evidencia de turnos enlazada a los recuerdos de sesión.
- `unresolved-evidence.jsonl`: preguntas cuyas referencias de evidencia no se pudieron resolver.
- `manifest.json`: revisión, hash SHA-256, cobertura y recuentos de la preparación.

## Interpretación y límites de las etiquetas

La evidencia del benchmark apunta a turnos, mientras que el recuerdo importado representa una sesión completa. El preparador resuelve cada turno al recuerdo de su sesión y usa esos recuerdos como `expected_ids`. Por tanto, la métrica que se obtenga mide recuperación de sesiones que contienen turnos anotados, no recuperación exacta de mensajes. Para búsqueda, un resultado es pertinente si trae al menos una sesión anotada; también se informan por separado los casos donde aparecen todas las sesiones anotadas. Para `recall`, LoCoMo declara `selection_semantics: any_evidence_memory`: seleccionar una de esas sesiones cuenta como semilla con evidencia útil, mientras que la cobertura de todas las sesiones anotadas se conserva como métrica separada. La abstención vacía sigue exigiéndose en preguntas sin evidencia.

El preparador conserva las 1.986 preguntas. Las categorías 1–4 tienen respuestas anotadas; las preguntas de categoría 5 son adversariales y pueden tener solo `adversarial_answer`, que se conserva por separado sin convertirlo en respuesta correcta. También preserva respuestas numéricas y campos ausentes sin inventar datos.

En la revisión fijada, 1.978 preguntas tienen evidencia resoluble, cuatro no tienen evidencia y cuatro contienen referencias incompletas o inexistentes. Las cuatro sin evidencia permanecen como casos de abstención; las cuatro no resolubles se mantienen en el inventario, pero no entran en `search-cases.jsonl` para evitar contarlas como negativos. El runner de campañas las identifica como `external_benchmark_annotation`, sin atribuirlas a etiquetas sintéticas propias. Si se modifica la revisión del corpus, el manifiesto y estos recuentos deben volver a comprobarse.

El paquete sirve para preparar y ejecutar una evaluación de recuperación sobre conversaciones sintéticas anotadas. Por sí solo no demuestra calidad de `mem-write` frente a recuerdos reales, fidelidad de las respuestas finales, utilidad para usuarios ni rendimiento end-to-end del asistente.
