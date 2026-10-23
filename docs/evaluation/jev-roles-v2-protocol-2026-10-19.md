# Protocolo previo: Jev con roles v2

Fecha: 2026-10-19. Jev/roles_v1 eliminó los dos errores de ausencia de la base y ganó cinco evidencias útiles en v4 conocida, pero seleccionó una recomendación de iluminación como respuesta a tratamiento de moho en vez de la evidencia que descarta moho. Un ID ajeno bloquea promoción; no se corrige gold ni se ejecuta su segunda cohorte.

Probar la instrucción `roles_v2` ya definida con Jev, sin retocarla con resultados Jev. La prueba previa Decisions/roles_v2 falló; cambiar modelo es otra hipótesis, no garantía. Perfil de evaluación `jev_roles_v2`: `provider: jev`, `selection_policy: roles_v2`, una llamada. Comparar con Decisions/base; cambian modelo e instrucción. Los modelos se mantienen ya disponibles, sin nuevas variables o endpoints.

Primero desarrollo v4 y, solo si pasa, v2: 128 operaciones/peticiones por cohorte, orden alternado adyacente, 2–4 minutos cada una. Si falla detener antes de abrir v5. Si pasan ambas, v5 nueva 066–073/224 recuerdos/64 preguntas, revisión ciega separada y conciliación por ID/cita antes de otras 128 operaciones. Conservar originales y desacuerdos. Misma familia de agentes; no validación humana.

Mismos límites: ninguna selección ajena, pérdida de evidencia útil/completitud o degradación nueva; al menos una ganancia. Recibos completos, coste y p50/p95 ≤1,5× base. Auditar candidatos idénticos, fuente declarada, revisión/texto/ámbito/vigencia, modelos solicitados, hashes y eventos. Ningún reintento incierto ni cambio de etiquetas tras resultados. Corpus sintético y DB temporal, sin puertos. Coste conocido de cada intento y acumulado; no incluye uso de Codex programando/revisando. Si falla, objetivo activo y variante no promovida.
