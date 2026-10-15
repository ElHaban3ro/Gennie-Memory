# Campaña remota v5: ejecución y cierre de semillas

Estado: **campaña ejecutada el 2026-10-15** tras autorización del usuario. El [informe de resultados](../../outputs/iromem/evaluation/v5-campaign-remote/README.md) registra 2.800 llamadas intentadas, ocho HTTP 504 de OpenAI, ninguna tarea incierta, US$0,06959 informados o reservados y la composición de 1.600 semillas por modelo. Los casos son sintéticos y proceden de [la muestra congelada](../../tests/benchmark/v5/development-campaign-1600.jsonl). Las instrucciones siguientes sirven para auditar o reproducir el recorrido, no para volver a cobrar llamadas ya realizadas.

Preflight repetido el 2026-10-15: el preparador regeneró un plan **idéntico byte por byte** desde la muestra y la instantánea híbrida; su huella interna sigue siendo `b7c8452a82bcfe6e3418686635aeccf1c1af6a3dc6ed5d675476a5c354c21035`. La variable `IROMEM_OPENROUTER_API_KEY` está presente en el `.env` ignorado, sin registrar su valor. Las fichas oficiales de ambos modelos seguían mostrando las tarifas indicadas abajo. Este preflight no realizó peticiones al proveedor ni constituye aprobación del gasto.

Se completó una [prueba de integración sin red](../../outputs/iromem/evaluation/v5-campaign-development/remote-dry-run-summary.json) del plan entero: 2.800 decisiones simuladas, 3.200 tareas terminales, 1.600 semillas compuestas por modelo y cero IDs prohibidos. Utilizó una clave ficticia y un proveedor simulado; sus aciertos, tiempos y costes no describen Jev ni OpenAI.

## Preflight

1. Verificar que el plan y la [instantánea híbrida](../../outputs/iromem/evaluation/v5-campaign-development/v5-campaign-hybrid-results.jsonl) mantienen las huellas de entrada fijadas, que `.env` contiene `IROMEM_OPENROUTER_API_KEY` sin mostrarla y que hay espacio local suficiente para el diario. El plan calcula 1.400 llamadas a `typesafe/jev-1.13` y 1.400 a `openai/gpt-6-luna-decisions`; 200 consultas sin candidatos por modelo se resuelven sin petición.
2. Confirmar disponibilidad y precio vigentes en las páginas de [Jev 1.13](https://openrouter.ai/typesafe/jev-1.13/) y [GPT-6 Luna Decisions](https://openrouter.ai/openai/gpt-6-luna-decisions). Las páginas consultadas el 2026-10-15 mostraban US$0,042/M y US$0,10/M tokens de entrada, respectivamente; la facturación efectiva se registrará por llamada.
3. Conservar la base y el binario de la corrida local antes de componer: las huellas SHA-256 de `.local/pilots/v5-dev-60-hybrid.db` y `target/debug/iromem` deben coincidir con `database_sha256` y `binary_sha256` del primer registro en [execution-manifests.json](../../outputs/iromem/evaluation/v5-campaign-development/execution-manifests.json). Si no coinciden, no usar la base cambiada como si reprodujera el benchmark.

## Clasificación remota

Desde la raíz del repositorio, un solo proceso a la vez:

```sh
python3 scripts/run_openrouter_campaign.py run \
  --plan outputs/iromem/evaluation/v5-campaign-development/v5-campaign-openrouter-plan.json \
  --journal .local/pilots/v5-campaign-openrouter-journal.jsonl
```

El diario se sincroniza antes y después de cada petición. Si el proceso termina con tareas incompletas, inspeccionar el estado antes de reanudar con **el mismo comando**; una petición `started` sin `finished` exige resolver su resultado incierto como fallo, sin repetirla automáticamente. El primer fallo explícito del proveedor detiene el lote para diagnóstico; las llamadas ya terminadas no se repiten. El límite de US$1 y 3.600 segundos activos se revisa antes de iniciar cada petición y una sola petición puede rebasarlo.

Cuando todas las tareas tengan estado terminal:

```sh
python3 scripts/run_openrouter_campaign.py export \
  --plan outputs/iromem/evaluation/v5-campaign-development/v5-campaign-openrouter-plan.json \
  --journal .local/pilots/v5-campaign-openrouter-journal.jsonl \
  --output .local/pilots/v5-campaign-openrouter-decisions.json
```

El informe completo contiene 1.600 resultados por modelo, incluidos vacíos sin llamada y fallos explícitos. `export --partial` sirve solo para diagnóstico, no para concluir la comparación.

## Composición y evaluación

Para cada modelo, construir solicitudes desde el informe remoto, la muestra etiquetada y la **misma petición híbrida con vector congelado**. Ejemplo Jev:

```sh
python3 scripts/build_seed_requests.py \
  --queries tests/benchmark/v5/development-campaign-1600.jsonl \
  --retrieval-results outputs/iromem/evaluation/v5-campaign-development/v5-campaign-hybrid-results.jsonl \
  --decision-report .local/pilots/v5-campaign-openrouter-decisions.json \
  --model typesafe/jev-1.13 \
  --output .local/pilots/v5-campaign-jev-seed-requests.jsonl

IROMEM_DATABASE_PATH=.local/pilots/v5-dev-60-hybrid.db \
  target/debug/iromem compose-seed-batch .local/pilots/v5-campaign-jev-seed-requests.jsonl \
  > .local/pilots/v5-campaign-jev-seed-results.jsonl

python3 scripts/evaluate_seeds.py .local/pilots/v5-campaign-jev-seed-results.jsonl \
  --requests .local/pilots/v5-campaign-jev-seed-requests.jsonl \
  --output .local/pilots/v5-campaign-jev-seed-evaluation.json
```

Repetir con `--model openai/gpt-6-luna-decisions` y rutas `v5-campaign-openai-*`. El constructor exige IDs y revisiones presentados, etiquetas idénticas y vector híbrido válido. El compositor revalida la selección contra la base. Los fallos de proveedor generan semilla degradada y no cuentan como acierto. Guardar informes por caso, coste informado, llamadas fallidas, latencias y fidelidad por segmento en un paquete versionado sin credenciales; contrastar siempre ambos modelos con los mismos casos y candidatos.
