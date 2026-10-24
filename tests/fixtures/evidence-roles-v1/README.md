# Roles anotados, desarrollo local

Veintiséis casos sintéticos para el contrato `evidence-roles-v1`: nueve aceptaciones (respuestas, correcciones y partes útiles), nueve incompatibilidades deliberadas de identidad, cuatro correspondencias no demostradas y cuatro entradas inválidas. No son muestras nuevas de selección automática ni prueban extracción completa. Anotaciones del mismo autor, sin validación humana.

Ejecutar `gennie-memory-eval evaluate-role-contract tests/fixtures/evidence-roles-v1/development.json <informe-nuevo.json>`. Los casos fallidos se conservan y el comando termina con error. El informe liga entradas por SHA-256. Las pruebas Rust también mutan evidencia válida para comprobar que citas inventadas y falta de sujeto no pasan.
