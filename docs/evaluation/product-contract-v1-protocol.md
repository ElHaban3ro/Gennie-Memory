# Separación prospectiva del producto v1

Estado vigente: ejecutor `run-product-suite` y auditor offline `audit-product-campaign` implementados. Las tres etapas del perfil congelado pasaron; [resultados y límites](product-contract-v1-results-2026-10-23.md). La reserva final está consumida para esa receta.

`gennie-memory-eval validate-product-suite <suite.json>` comprueba integridad del manifiesto local sin ejecutar inferencia. No reemplaza el evaluador histórico ni convierte sus consultas conocidas en una reserva nueva.

El manifiesto exige una política congelada por archivo/hash, tres particiones JSONL no vacías (`development`, `calibration`, `final`), cantidades y SHA-256 exactos. Rutas relativas contenidas en el directorio del manifiesto, archivos ≤8 MiB. Rechaza ID repetido, consulta runtime idéntica ignorando request_id, y cruces de grupos de fuente, entidad o familia entre particiones.

Ejemplo de estructura (los hashes y cantidades deben calcularse sobre archivos reales; los umbrales de ejemplo no son una decisión de adopción):

```json
{
  "contract_version": 1,
  "policy_path": "policy.json",
  "policy_sha256": "<sha256 del archivo>",
  "protocol": {
    "max_calls": 30,
    "max_seconds": 300,
    "max_cost_usd": 0.5,
    "unlimited_cost_authorized": false,
    "min_useful_recall": 0.95,
    "max_unrelated_selections": 0,
    "max_degradations": 0,
    "max_p95_ms": 1500,
    "label_source": "human_reviewed"
  },
  "partitions": [
    {"split":"development","path":"development.jsonl","sha256":"<sha256>","count":10},
    {"split":"calibration","path":"calibration.jsonl","sha256":"<sha256>","count":10},
    {"split":"final","path":"final.jsonl","sha256":"<sha256>","count":10}
  ]
}
```

Cada línea contiene `id`, listas no vacías `source_groups`, `entity_groups`, `family_groups`, `request` del producto, `expected_outcome` y `expected_evidence` con `{memory_id,revision,relation}`. Las expectativas quedan fuera del request. Relaciones esperadas: `supports_answer`, `refutes_premise`, `partial_context`, `unassessed`; evidencia vacía cuando no se selecciona. Los grupos son anotación explícita: el validador no detecta paráfrasis semánticas ni verifica que las agrupaciones sean correctas.

`label_source` admite `agent_synthetic_unreviewed`, `blind_synthetic_adjudication` o `human_reviewed`. El origen no equivale a una certificación. Coste ilimitado requiere `max_cost_usd:null` y autorización explícita reflejada en `unlimited_cost_authorized:true`; el campo no otorga autorización por sí mismo.

La salida declara `structural_validation:passed`, `semantic_quality:not_evaluated`, `runtime_schema_validation:deferred_to_product`, cero llamadas y `final_executed:false`. Sin `corpus`, comprueba envoltorios y separación de etiquetas; el enlace de revisiones gold queda `not_provided`. Con `corpus:{path,sha256}`, valida además política efectiva, identidad/ámbito/entidad, revisión vigente, actividad y vigencia de evidencia anotada. Los grupos de fuente y entidad deben corresponder a todos los recuerdos del ámbito, incluidos distractores y revisiones anteriores. No comprueba todo el esquema runtime ni relevancia semántica. La validación del producto y el enlace al corpus deben completarse antes de inferencia. Este comando solo valida; `run-product-suite` ejecuta con control de llamadas/tiempo y reserva monetaria local, deteniendo avances ante recibos desconocidos. La reserva no garantiza el precio facturado por el proveedor.

## Uso de las particiones

1. Desarrollo: ajustar segmentación, selección y contrato. Evaluar el camino completo entrada→búsqueda→clasificación→composición, sin deducir calidad global de un adaptador aislado.
2. Calibración: fijar parámetros/umbrales por proveedor y política usando únicamente esta partición, registrar configuración/corpus/modelo y binario. No igualar escalas de puntuaciones distintas.
3. Final: congelar política y criterios antes de abrir resultados. Ejecutar una vez bajo presupuesto autorizado; cualquier ajuste posterior requiere nueva reserva independiente. Reportar evidencia útil recuperada, selecciones ajenas, degradaciones, p95, bytes, tokens y coste conocido/desconocido. Ningún fallo técnico se cuenta como abstención semántica ni ningún recibo ausente como gasto cero.

Existe un [banco nuevo de 18 casos y 36 recuerdos](../../tests/fixtures/product-contract-v1/README.md), con dos revisiones sintéticas separadas y conciliación explícita. La campaña posterior ejecutó desarrollo, calibración y final sin ajustes entre etapas, con resultados enlazados arriba. Las pruebas sintéticas del código demuestran conformidad estructural; la aprobación del producto completo permanece abierta.

## Corpus y revisión ligados

El corpus opcional conserva compatibilidad con manifiestos iniciales. Para nuevos bancos, exigirlo. La política ligada debe contener `provider`, `selection_policy`, `search:{limit,query_mode}` y `budgets:{input_bytes,seed_bytes,deadline_ms}`; el validador compara los valores efectivos de cada request, incluidos defaults. El corpus ligado requiere entidad y fuente declarada por recuerdo; no valida fuentes externas.

`gennie-memory-eval audit-product-reviews <suite> <packet> <rubric> <review-a> <review-b>` verifica las dos revisiones contra los hashes de paquete/rúbrica y contra el corpus/consultas congelados. Exige cobertura completa y compara selecciones, relaciones y resultados con las etiquetas. Conserva preocupaciones y desacuerdos en la salida, incluso cuando `label_agreement:true`. No adjudica automáticamente observaciones ni confirma que revisores de identidad diferente sean humanos o de familias independientes. La revisión sintética acordada autoriza a considerar las etiquetas listas para una campaña, no a promover la política.

## Ejecución y auditoría

`gennie-memory-eval run-product-suite <suite.json> <binario-gennie-memory> <directorio-nuevo>` exige el banco ligado, provider-verification.json con evidencia de precio/acceso/presupuesto, review-packet/rubric y review-a-v2/review-b-v2. Revalida las revisiones. Usa DB y capturas nuevas, una sesión propia, y conserva un diario sincronizado antes/después de cada operación. No reabre directorios anteriores ni reintenta operaciones inciertas. Es un comando con inferencia pagada, separado de los validadores offline.

Una búsqueda local sin gold determina si reservar una llamada. El plazo global limita esperas de sesión; Drop cierra stdin y termina/espera únicamente el proceso hijo propio. La carga local de fixtures es previa a inferencia y no tiene cancelación durante su transacción. Las puertas incluyen resultados, relaciones, evidencia ajena, degradación, fidelidad y p95 por partición. La calibración puede validar una receta sin modificarla.

`gennie-memory-eval audit-product-campaign <suite.json> <binario-gennie-memory> <directorio-campaña>` reconstruye resultados desde el diario sin red y revalida hashes, secuencia, casos, revisiones, recibos y texto exacto de semilla. Es apto para reanalizar resultados existentes, no restituye una reserva ya abierta.
