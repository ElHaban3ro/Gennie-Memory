# Protocolo previo: Jev con selección por roles

Fecha: 2026-10-19. La revisión `roles_v2` falla desarrollo v4: cuatro ID ajenos frente a dos de la base; se detiene antes de ejecutar el desarrollo v2 o abrir otra validación nueva. El gold de v4 no cambia. Se preservan resultados y costes de todos los intentos.

Contrastar `Decisions + baseline` con `Jev + roles_v1`, ambos ya disponibles en producto, una llamada por pregunta. El perfil de evaluación `jev_roles_v1` no es una nueva política runtime: establece `provider: jev`, `selection_policy: roles_v1`. La base usa Decisions. Cambian modelo e instrucción; no se atribuye la diferencia a uno aislado. No se cambia predeterminado ni se activa cascada. No hay nuevos secretos, dependencias o endpoints.

Puertas previas: desarrollo v4 y v2 con 128 operaciones por cohorte, adyacentes y alternadas, proveedores y políticas explícitos por caso, hasta 256 peticiones, previsión 2–4 minutos por cohorte. Si falla la primera, detener la segunda y preservar el resultado. Se registra modelo servido, gasto conocido, tokens, recibos ausentes y tiempos del runner. No reintentar peticiones inciertas.

Si supera ambas, una cohorte v5 nueva de ocho conversaciones 066–073/224 recuerdos/64 preguntas requiere crítico ciego separado que vea solo fuente y preguntas, conciliación y hashes antes de 128 operaciones nuevas. Sin forzar cuotas tras revisión; relación por ID para alternativas útiles. Mismo grupo de modelos de revisión, sin validación humana. SQLite temporal, ningún servicio ni dato real.

Se mantienen las puertas: cero ID ajeno, cero pérdida de evidencia útil o completitud frente a la base emparejada, cero degradación nueva y al menos una ganancia. Recibos completos, coste y p50/p95 ≤1,5× base. Auditar hash/eventos, candidatos idénticos, modelo solicitado correcto y fidelidad de semillas. No relabelar campañas observadas, no resumir parciales como respuestas completas, no presentar selección sintética como verdad de respuesta final de Codex. Mantener objetivo activo si la mejora no se demuestra.
