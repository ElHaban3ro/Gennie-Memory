# 0024 — Referencias con mención y clase declarada en evaluación

Fecha UTC: 2026-10-21. Estado: diagnóstico, sin adopción de producto.

Los pilotos de cláusulas confundieron tipo de objeto con metadatos, omitieron portadores y asignaron clases no declaradas. Se separan mención literal, nombre, clase citada o ausencia explícita, propietarios y cuantificador. Un módulo puro nuevo adapta esta versión al contrato por cláusulas y después al parser V5, sin modificar las versiones anteriores.

La clave de clase es codificación UTF-8 reversible conservadora, no alias manual ni tipo semántico del resolver. Los términos de entidad y filtros de solicitud pueden declarar tipos semánticos con evidencia propia; estos se conservan aparte para no romper rutas intermedias. Los tipos individuales en conjuntos que V5 no puede conservar degradan toda la operación, no se eliminan para mejorar cobertura. Igualdad léxica y literalidad no prueban correferencia ni atribución; ausencia de tipo no crea una clase anónima global ni comodín.

Propietario: `tools/evaluation/src/documentary_extraction_contract/grounded_references.rs`. Bootstrap lo incluye solo a través del contrato de runner de pruebas; no cambian dependencias, apps, SQLite, configuración o política runtime. Alternativa descartada: borrar kind de la identidad o llenarlo con aliases del gold para hacer coincidir fuente/pregunta. El nuevo modelo conserva clases distintas y declara posibles pérdidas por sinónimos, plural o mayúscula.

La prueba nueva de integración confundió C con dato parcial: el contrato V1/V5 ya permite A calificada para parte/intermedio y reserva C para contraste de configuración. Se conserva ese fallo y su fuente, se corrige la expectativa de la prueba con evidencia del contrato, y se exige un único hecho de depósito/proveedor inmediato, nunca un origen de red inventado. No se cambian resolver, gold ni resultados anteriores para aprobarla.

Revisión: código/contraejemplos y replay antes de red, mismo piloto completo de ocho y dos críticas de todos los contenidos antes de ampliar. Falla si hay ausencia, defecto o incertidumbre, aunque el DTO pase. Después siguen banco completo conocido/difícil, calidad, recursos/estabilidad y cohorte nueva ciega. [Protocolo](../evaluation/grounded-reference-v1-protocol-2026-10-21.md).
