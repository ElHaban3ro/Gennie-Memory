# Piloto técnico previo: clasificación por recuerdo

Fecha: 2026-10-19. Una operación conocida `cev5q_066_06` con Jev/per_memory_v1, una petición y hasta16 preguntas. DB temporal propia, CLI sin puerto, duración prevista hasta30s. Pregunta/corpus ya observados: no es validación nueva ni pase de calidad.

Objetivo: comprobar aceptación del formato, completitud de16juicios, composición de todos los positivos y recibo global único. Guardar coste, tokens, duración, número/bytes de semilla y errores; no asumir que una petición con16preguntas cueste como una con dos. Comparar consumo con el recibo baseline de ese mismo caso solo como diagnóstico, sin extrapolar el ratio al banco.

Si falla formato o recursos resultan manifiestamente superiores al límite, detener antes de campaña completa y diagnosticar conservando el fallo. No repetir una llamada incierta ni cambiar gold. El contrato de utilidad por recuerdo define riesgos y puertas posteriores; el predeterminado sigue baseline y no se usan datos reales.
