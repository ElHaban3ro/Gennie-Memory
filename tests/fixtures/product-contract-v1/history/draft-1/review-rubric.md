# Rúbrica de revisión sintética

Revisar exclusivamente `review-packet.jsonl`. No consultar corpus, particiones, manifiesto, código, etiquetas originales ni otro revisor. El paquete contiene consultas y recuerdos sintéticos, sin gold ni nombre de partición. Cada ID es opaco. La misma autoría redactó el banco y este paquete; la revisión no acredita datos humanos ni independencia entre familias de modelos.

Por caso, evaluar la consulta original con su contexto explícito. Las referencias sin resolver declaradas por quien llama implican `clarification_required`, sin selección. Si no existen recuerdos relevantes por vocabulario ni por significado, `no_candidates` es solo una hipótesis de búsqueda a verificar por el runtime; distinguirlo de abstención con distractores recuperables.

Por recuerdo usar una relación: `supports_answer` (dato explícito que responde), `refutes_premise` (negación explícita de una premisa de esa misma consulta), `partial_context` (dato provisional, condicionado o incompleto que responde a parte del dato solicitado) o `unrelated` (misma temática pero no aporta ese dato). No convertir ausencia de mención en refutación. Que un documento indique que no contiene el dato puede ser contexto parcial útil; no confundirlo con ausencia física de recuerdos.

Seleccionar toda evidencia útil de cualquiera de las tres primeras relaciones, conservando incertidumbre y condiciones. Si no hay evidencia útil, `abstained`, salvo aclaración o hipótesis de búsqueda vacía. Para cada decisión dar justificación breve de evaluación citando texto literal de recuerdos cuando corresponda; esto es anotación offline, no una justificación fabricada por el proveedor runtime.

Salida JSON:

```json
{
  "reviewer": "reviewer_a",
  "packet_sha256": "hash del paquete leído",
  "rubric_sha256": "hash de esta rúbrica",
  "cases": [
    {
      "case_id": "id opaco",
      "outcome": "evidence_selected",
      "assessments": [{"memory_id":"id", "revision":1, "relation":"supports_answer"}],
      "selected_ids": ["id"],
      "rationale": "Evaluación y evidencia literal breve",
      "concerns": []
    }
  ]
}
```

Incluir todos los casos y todos los recuerdos por caso. `concerns` recoge cualquier ambigüedad de pregunta, etiqueta, suficiencia de contexto o distinción entre resultado semántico y búsqueda. Para aclaraciones, marcar recuerdos `unrelated` a efectos de selección suspendida y explicar que no constituye un juicio semántico del proveedor. No consultar redes ni llamar APIs de pago.
