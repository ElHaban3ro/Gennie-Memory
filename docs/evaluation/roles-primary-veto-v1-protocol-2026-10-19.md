# Protocolo previo: agregación asimétrica de dos selectores

Fecha: 2026-10-19. Política experimental explícita `roles_primary_veto_v1`; perfil `jev_roles_primary_veto_v1`. Nace después del fallo de `roles_consensus_mirrored_v1` en v2: perdió una refutación porque el segundo selector se abstuvo. La proyección sobre votos observados sugiere que una regla asimétrica conservaría esa evidencia, pero es ajuste de desarrollo, no confirmación ni validación nueva.

## Regla y riesgo

Se envían exactamente las mismas dos preguntas, instrucciones y memorias que mirrored: roles_v1 propone en el orden original de top16; roles_v2 elige entre los mismos top16 en orden inverso; una petición Jev con recibo global único. Solo cambia la agregación de IDs normalizados:

| Primer voto | Segundo voto | Selección |
| --- | --- | --- |
| A | A | A |
| A | none | A |
| A | B distinto | none |
| none | A | none |
| none | none | none |

El segundo es otro selector, no una tarea específica de crítica. Su abstención no confirma aplicabilidad: aceptarla relaja el veto y puede aceptar una propuesta incorrecta. No se presenta como verificación independiente, seguridad o consenso exacto. El campo heredado `consensus` es traza de ambos votos, con count16 e IDs; `resolve_votes` posee la regla por política y el servicio la revalida. Las versiones anteriores conservan acuerdo exacto. Ausencia del voto, error o elección fuera de rango **degrada**, nunca se convierte en none. El recibo global se conserva una vez incluso al fallar.

El evaluador registra clases `agreement`, `primary_without_second`, `second_without_primary`, `conflict` y `both_abstain`. Auditar especialmente aceptación sin segundo voto, con votos permitidos y selecciones finales por rúbrica. Ninguna ganancia compensa una pérdida.

## Verificación y medición congeladas

Pruebas de parser/remapeo, servicio y evaluador cubren las cinco combinaciones, preservación de semántica anterior, rechazo fuera de top16, voto inválido/ausente, recibo único, deserialización y propagación. Primero repetir completas v4 conocida y después v2 conocida solo si pasa, sin reutilizar una proyección como campaña medida. 128 operaciones/peticiones por cohorte, 64 Decisions/base y 64 Jev/candidata, orden alternado por posición. Estimación 1–2 min cada una. Candidatos recuperados20 idénticos; candidata clasifica16. Mantener top16 y su cobertura útil/mejor nivel comprobados en ambos bancos sin excluir casos.

Puertas previas intactas: cero ID ajeno, cero pérdida por caso de evidencia útil o completitud frente a base, cero degradación nueva y al menos una ganancia; recibos completos y coste/p50/p95 ≤1,5× base. Preservar resultados negativos, etiquetas, preguntas, adjudicaciones y revisión ciega. Guardar hashes antes de cada fase. SQLite temporal propia, sin puertos, nuevas dependencias o reintentos inciertos. Detener ante fallo sin abrir otra cohorte.

Solo si pasan ambas, verificar cobertura de top16 en v5 reservada ya revisada a ciegas y conciliada; no ajustar política, preguntas, gold ni límite según ese resultado. Si pierde cobertura útil/mejor nivel, registrar y detener sin llamadas. Si pasa, ejecutar otras128operaciones/peticiones con la misma regla y puertas. Agentes de misma familia y revisión separada, no humana ni independencia de familia. La política sigue congelada ante el resultado nuevo; un fallo vuelve a desarrollo y exige otra cohorte para validar después.

Métricas miden selección de IDs y fidelidad literal, no fuentes externas o respuesta final de Codex. Coste comunicado no incluye programación/revisores. Un pase conocido no prueba estabilidad ni población; no promover antes de validar la cohorte nueva y completar las cinco puertas del goal.
