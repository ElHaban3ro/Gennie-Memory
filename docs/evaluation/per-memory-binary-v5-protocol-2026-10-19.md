# Utilidad binaria v5: OR explícito entre dato y refutación

2026-10-19. V4 se detuvo tras una sola operación completada (USD0,000056406), sin ampliar a las33restantes: revisión crítica encontró que «si no hay dato solicitado, none» podía vetar una corrección válida. Fue un riesgo de redacción detectado estáticamente, no una regresión semántica medida. Lanzar antes de concluir esa revisión fue un error de coordinación; registro/payload inicial se preservan.

Variante optativa `per_memory_binary_v5`. Deriva V3 (tarea explícita) y añade únicamente state.rules: corrección requiere cláusula explícita incompatible con premisa exacta en el mismo caso; si ambas pueden ser verdaderas falla ese test. Comprobar de forma independiente dato/estado/parte solicitados explícitos. A si cualquiera de los dos tests pasa; none solo si ambos fallan. No exigir incompatibilidad a apoyo o partes solicitadas. V4 anterior permanece sin cambiar. No usa ejemplos ni gold.

Los demás campos de payload, modelos, top8, tarea binaria, composición de todos los positivos y recibo global único quedan iguales. No cambia defaults, FTS, recursos o dependencias. Un HTTP por consulta, con coste/tokens reales registrados.

Piloto34conocidas: las18del diagnósticoV3 más16unrelated, consultas/gold originales con huellas obligatorias. Primero una operación para formato, luego33. CLI sin puerto, DB temporal propia ya creada, duración prevista30s, máximo34HTTP sin reintentos inciertos. Exigir cero ID ajeno, cero pérdida útil de base/degradación y coste histórico diagnóstico<=1,5×. Un fallo detiene ampliación; ningún piloto sustituye comparación pareada.

Tras piloto favorable: comparación completa pareada v5(64preguntas/128operaciones), después v4/v2 conocidas. Puertas iguales: cero ID ajeno, pérdida de evidencia/partes disponibles o degradación nueva; alguna ganancia útil/adicional; recibos completos y coste,p50,p95<=1,5×. Después política congelada, V6 nueva revisada a ciegas, sin usarla para ajustes; incorporar solo si pasa todo. Conservar los fallos y costes desconocidos previos.
