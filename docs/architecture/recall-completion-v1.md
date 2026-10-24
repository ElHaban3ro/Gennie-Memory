# Factores de recall y adquisición HTTPS explícita

Fecha: 2026-10-24. La implementación añade mecanismos optativos al contrato v1. La validación semántica se registra por separado en el [informe de cierre](../evaluation/recall-completion-v1/README.md); implementar tipos y mocks no acredita calidad general.

## Aplicabilidad y relevancia separadas

`recall-v1` admite ahora la política existente `per_memory_factors_v1`: hasta ocho candidatos, tres preguntas tipadas independientes por candidato, una petición HTTP y un recibo global. Conserva preguntas/reglas, prefijo, proveedor y regla de selección. El perfil de producto presenta resumen, condiciones, estado y fuente juntos como datos; la ruta heredada conserva su presentación histórica del resumen. Retrieval vuelve a validar IDs, revisiones, cardinalidad, relaciones y composición de los factores. Los demás candidatos quedan sin evaluar y la cobertura parcial permanece explícita.

Cada assessment añade `applicability` (mismo referente/contexto), `requested_fact` (dato/parte/estado solicitado), `premise_correction` (corrección explícita) y `relevance` (OR de los dos últimos). Selección requiere `applicability && relevance`. Son juicios fallables del proveedor. Una memoria puede ser temáticamente útil pero pertenecer a otro referente: sus factores se conservan separados y no se selecciona. Baseline y políticas categóricas exponen null, sin inventar razones o juicios de aplicabilidad.

`factor_scores` conserva `same_context`, `requested_fact`, `premise_correction` como puntuaciones individuales. Se validan finitud, rango [0,1] y normalización binaria con tolerancia 0,02. Un grupo incompleto/malformado falla atómicamente conservando el recibo conocido. El protocolo Decisions/Jev exige probabilidades binarias válidas en cada respuesta; su ausencia falla sin convertirla en cero. El tipo opcional permite distinguir otros adaptadores/trazas históricos sin scores y candidatos no evaluados. No se calcula una probabilidad conjunta ni se interpreta como confianza factual. El adaptador conserva el modelo servido, tokens y coste global una vez.

## Calibrador optativo

`ProductRequest.calibration` acepta un artefacto inline del perfil `factor_empirical_bins_v1`:

```json
{
  "schema_version": 1,
  "profile": "factor_empirical_bins_v1",
  "profile_sha256": "SHA256 de bootstrap/src/openrouter/per_memory/factors.rs",
  "provider": "decisions",
  "selection_policy": "per_memory_factors_v1",
  "requested_model": "openai/gpt-6-luna-decisions",
  "served_model": "openai/gpt-6-luna-decisions-20261006",
  "training_sha256": "SHA256 del JSONL de calibracion congelado",
  "factors": {
    "same_context": [{"upper": 1.0, "positives": 6, "n": 8}],
    "requested_fact": [{"upper": 1.0, "positives": 4, "n": 8}],
    "premise_correction": [{"upper": 1.0, "positives": 2, "n": 8}]
  }
}
```

Los conteos del ejemplo son ilustrativos, no resultados; served_model muestra la identidad observada en el primer recibo real, sin convertirla en alias. Cada factor admite 1–20 intervalos ordenados, límite superior >0 y <=1, último límite 1, `n>0`, `positives<=n`. Se elige el primer intervalo con score <= upper y se aplica suavizado Laplace `(positives+1)/(n+2)`. Son frecuencias empíricas de ese perfil; un solo intervalo estima únicamente su tasa base y carece de discriminación. La partición de calibración determina conteos; desarrollo no ajusta final. El artefacto/informe externo liga además hashes de etiquetas, corpus y particiones. El operador puede aportar datos arbitrarios; el producto valida estructura/compatibilidad, no firma ni autoría de entrenamiento.

Antes de búsqueda/red se comprueban perfil, proveedor, política, requested_model exacto contra el alias solicitado, compatibilidad del served_model (alias exacto o alias seguido de guion y ocho dígitos) y huella exacta del archivo fuente del adaptador (incluye parser/pruebas; un cambio de bytes invalida artefactos anteriores). Si no coinciden, `calibration_profile_mismatch`, sin llamada. El calibrador se entrena con un único served_model observado, independiente del alias requested_model. El recibo de cada aplicación debe coincidir exactamente con ese served_model o se devuelve `calibration_model_mismatch`, conservando consumo y sin publicar semilla. No se infiere una identidad de modelo cuando falta recibo.

La respuesta separa `calibrated_factor_scores` de `factor_scores`, con estado `empirical_profile_bound` cuando se aplicó o `not_calibrated` si no hay calibrador. Si se solicita calibrador y faltan scores, `calibration_scores_unavailable` conserva el recibo y omite semilla. Calibrar no altera elecciones, umbrales ni baseline y no es un certificado de verdad. Un artefacto sin evaluación reservada documentada no acredita calibración validada. Coste local: validación de hasta sesenta intervalos y tres búsquedas por candidato, sin tablas, descargas o dependencias.

La [prueba reservada ejecutada](../evaluation/recall-completion-v1/README.md) empeoró Brier en los tres factores para ambos proveedores. Los artefactos de esa corrida quedan experimentales, rechazados para adopción como mejora. `empirical_profile_bound` acredita aplicación del mapeo, no haber superado la prueba de fiabilidad. Ningún umbral/default fue promovido.

## Conector HTTPS explícito

`source-import-https <request.json>` y session `source_import_https` adquieren una fuente nombrada expresamente. La solicitud tiene `registration` con el contrato source-register (texto vacío), `url` y `expected_sha256` de los bytes UTF-8 esperados. `registration.source.ref` debe ser idéntico a URL; locator ausente. Se validan esquema, actor/ámbito/versión/tipo antes de red. Solo HTTPS, puerto 443, sin usuario/contraseña, query o fragmento; máximo512 caracteres. El operador personal puede nombrar hosts internos HTTPS deliberadamente: no es un proxy multiusuario ni una URL extraída de recuerdos/modelos.

Cliente nuevo sin proxy, sin credencial OpenRouter ni redirecciones, verificación TLS normal, plazo 10 s. Solo HTTP2xx y Content-Type `text/plain`/`text/markdown`; límite 256 KiB antes/durante lectura, UTF-8 estricto y SHA256 obligatorio. Se rechazan otros formatos sin extracción automática. Se adquiere completamente fuera de la transacción SQLite. Solo tras verificar contenido se llama al registro existente sobre la única conexión; versiones inmutables y replay conservan su semántica.

El recibo devuelve snapshot confirmado y `acquisition` con perfil `explicit_https_import_sha256_v1`, URL, hash, TLS validado y ausencia de redirecciones. El snapshot conserva su base `operator_registered_snapshot`: la adquisición no persiste una certificación externa ni eleva `verified_literal` a verdad/autoría/vigencia. HTTPS vincula transporte al host atendido; el hash esperado vincula bytes a la versión elegida por el operador. Para auditoría conserve recibo del conector. Sin red automática durante recall, credenciales nuevas, daemon, HTTP local, scraping, conectores de servicios autenticados o reintentos.

E5 y el híbrido siguen como investigación por decisión vigente. La condición «completar su integración si vuelve a ser una ruta activa» no se activa por este trabajo. La ruta heredada de recall mantiene presentación histórica de candidatos, contratos y defaults; su traza de factores añade los scores individuales recibidos.
