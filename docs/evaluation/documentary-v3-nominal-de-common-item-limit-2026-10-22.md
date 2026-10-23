# Límite de respuesta común para un ítem nominal anónimo

Fecha: 2026-10-22. Revisión de informe conservado y código; sin cambios Rust, Cargo, nuevas llamadas o adopción de producto.

## Resultado observado

El control manual `known-source-candidate/manual-common-item-input.json`, bajo `outputs/iromem/evaluation/documentary-v3-readings-campaign-2026-10-21/nominal-de-contract-v1/`, pregunta el slot `item` de `has_item` para Senda. Usa como texto de control «Senda tiene una bandeja de papel grande.»; es un control nuevo, no una contraparte del banco ni medición de extracción.

En `manual-common-item-final-report.json`, las dos hojas (`adjective_on_tray` y `adjective_on_paper`) tienen `complete_answer=true`, cobertura completa y selección exclusiva de `possession`. Ambas devuelven el término fuente local `ref:tray`; ninguna tiene candidatos pendientes. El envelope conserva `coverage=conditional_reading`, `complete_answer=false` y `requires_review=true`. La metadata del papel ya no contamina esta consulta. El fallo de agregación permanece y no se cambia la expectativa para aprobarlo.

Hashes SHA-256 de los archivos conservados:

| Archivo | SHA-256 |
| --- | --- |
| `manual-common-item-input.json` | `ba282748568e6d1d09ce9ae49c7ad6313c9182a560ff9ad85e490dd553849c74` |
| `manual-common-item-initial-report.json` | `fb577a4941e8038c06720e1e9a85f6f308004b1f2bea20d05333543cfe9d3597` |
| `manual-common-item-final-report.json` | `8e4199c4db8c2fa937ae232bfb74684717715906ae8fbc4b86fb4f7b8e37043e` |

## Causa y capacidad existente

`tools/evaluation/src/documentary_v3_io/readings.rs` exige en `request_signature` una firma canónica completa de los términos de respuesta y del contenido de los claims. `canonical_inner` envía tanto `ref` como `described_ref` a `reference_key`. Con el perfil contextual activo, `contextual_reference_key` exige nombre explícito; `tray` es anónimo y produce `None`. La ausencia de firma impide declarar una conclusión común incluso cuando ambas hojas resuelven la solicitud localmente.

Es una restricción conservadora del contrato de agregación, no evidencia de que las dos lecturas sean semánticamente incompatibles. Compartir ID, clase, mención o complemento no demuestra identidad global de una bandeja. No corresponde crear una clave global para superar esta puerta.

La representación actual puede conservar el testigo existencial local y su procedencia dentro de cada hoja. También conserva las dos respuestas locales en `evidence_by_reading`. No dispone de una proyección ejecutable que publique ese contenido como respuesta común existencial sin exigir antes una identidad canónica del referente. Cambiar a `described_ref` no evita la comprobación; convertir el término a texto o cambiar la pregunta sustituiría el control y no acreditaría la capacidad faltante.

## Proyección semántica posible, todavía no implementada

Ambas lecturas respaldan una conclusión local equivalente a «Senda tiene al menos una bandeja de papel», manteniendo indeterminado el subtipo físico de «de papel». La proyección puede cuantificar una variable local por testigo y comparar su clase y construcción nominal bajo renombrado acotado; no necesita afirmar que dos entidades anónimas de documentos distintos son Same. La adjunción de «grande» permanece abierta y no pertenece a esa conclusión común.

Para publicarla haría falta un contrato explícito de proyección de conclusión existencial, separado de `reference_key`, que conserve texto, claims y referencias originales como procedencia y verifique actor nombrado, rol, polaridad, fundamento, scopes, qualifiers, restricciones y ausencia de conflictos en todos los cruces. Debe impedir fusión de episodios, transferencia de atributos y reutilización del uso nominal entre owners, además de respetar el presupuesto compartido. Este documento no autoriza ni implementa esa ampliación.

El estado vigente conserva el control fallido de respuesta común. La aceptación semántica del candidato y las pruebas del mecanismo no se presentan como utilidad demostrada, extracción autónoma ni adopción de producto.
