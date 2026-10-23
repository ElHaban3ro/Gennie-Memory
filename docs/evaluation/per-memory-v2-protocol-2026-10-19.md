# Clasificación por recuerdo v2: desarrollo conocido

Fecha: 2026-10-19. Objetivo activo, sin promoción. `per_memory_v2` conserva el contrato y los cuatro juicios de v1, con prefijo máximo de ocho candidatos y todos sus positivos compuestos. Una petición, ocho preguntas como máximo y un recibo global; el proveedor experimental es Jev. V1 mantiene dieciséis y sus resultados originales.

Motivo: el piloto v1 costó 1,7866 veces el recibo histórico de base de una pregunta conocida. Ese diagnóstico no estima el coste del banco. Antes de llamadas v2 se comprobó que el prefijo ocho contiene cada ID seleccionado por base en los diarios conocidos v2, v4 y v5. Esto no prueba cobertura de todas las partes ni calidad del clasificador.

Primero: una operación técnica sobre `cev5q_066_06`, DB temporal propia, CLI sin puerto, prevista hasta30s; guardar formato, positivos, coste y tokens sin extrapolar rendimiento. Después, si el formato y coste justifican continuar, comparación pareada sobre v5 conocida:64 preguntas,128 operaciones máximo, sin reintentos inciertos, duración prevista2–4min. Si falla, conservar el fallo y analizar antes de otros bancos. Ningún banco observado vuelve a ser nuevo.

Puertas predefinidas: todos los IDs de cada semilla deben ser permitidos; no perder ningún ID permitido que conserve base; ninguna degradación nueva; mejora útil o evidencia adicional. Recibos y operaciones completos; coste, p50 y p95 como máximo1,5 veces base. Los bytes y cantidad de memorias de semilla se informan separadamente; no son tokens ni coste de Codex. La relación resumida por pregunta corresponde al primer ID y no significa cobertura completa. No se relajan puertas ante resultados malos.

Antes de validación nueva: superar desarrollo, congelar política y fuentes, completar revisión separada sin etiquetas ni resultados y cobertura explícita de partes. No se modifica Decisions/baseline predeterminado ni se utilizan datos reales.
