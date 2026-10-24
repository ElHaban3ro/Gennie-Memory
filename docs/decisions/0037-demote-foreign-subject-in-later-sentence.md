# 0037 — Demover sujetos ajenos en segmentos posteriores

## Contexto

La evaluación retrospectiva de `smoke-14` mostró un recuerdo con una oración útil de Marta y otra oración independiente de Teo. El guard V3 revisaba relativas y complementos de reporte antes del predicado, pero podía aceptar una cita amplia que enlazara el sujeto citado de la primera oración con el predicado de un sujeto con ID ajeno en la siguiente.

## Alternativas

- Rechazar toda la traza cuando una de sus proposiciones presenta atribución ajena, lo que también perdería evidencia correcta del mismo candidato.
- Aceptar la cita amplia sin revisar la oración del predicado.
- Demover solo la proposición cuyo predicado aparece en una oración posterior con otro ID estable y sin el ID consultado.

## Decisión y estado

Se elige la tercera alternativa. V3 marca esa proposición como `unresolved_attribution` y limpia su target; las demás proposiciones del candidato se validan independientemente. La regla exige un límite explícito (fin de oración, punto y coma, o `, while` / `, mientras`), sujeto con ID estable ajeno antes del predicado e ID consultado ausente de ese segmento. No reescribe texto ni añade dependencias. Predicados relativos, reportados y coordinación siguen en sus guardas previas.

La regresión sintética de Marta/Teo demuestra que Valencia permanece seleccionable mientras Girona queda demovido. También se conserva el control inverso (Carlos antes, Marta después) y la adjudicación local previa de 14 casos: 7/7 respuestas y 0/7 ajenos con trazas escritas para la prueba. Las pruebas de retrieval y `gennie-memory` pasan; ver [resultado y límites](../evaluation/slot-evidence-v3-sentence-scope-results-2026-10-23.md).

La extensión se contrastó contra una fixture prospectiva sintética de 10 casos, congelada antes de revisar etiquetas: `tests/fixtures/slot-evidence-v3-sentence-scope-prospective-v1.json`, SHA-256 `f8c8f359f84eec7eba768be847806a439db3940a385f211ae1c2972ea7f6ee5f`. Dos revisores ciegos coincidieron en los diez juicios semánticos. Una prueba focalizada comprueba positivos directos y sujetos ajenos en oraciones posteriores, además de Teo con `while` y punto y coma; no evalúa extracción del proveedor.

La ruta sigue experimental. No acredita extracción autónoma ni mejora poblacional; los 14 smokes remotos V3 fallaron y no se repiten. No cambia baseline ni política predeterminada. Para revisar esta decisión se necesita una cohorte nueva ciega que pruebe límites de oración, cláusulas en una misma oración, apoyo, refutación y parcialidad; debe conservar evidencia útil y no seleccionar atribuciones ajenas antes de otra comparación contra baseline.
