# Diagnóstico del smoke r12

- Resultado: `degraded`, fallo cerrado `slot_evidence_query_target_quote_invalid`; ningún recuerdo seleccionado.
- Búsqueda: candidate recall 1/1; el único candidato sintético correcto llegó al clasificador.
- Modelo servido: `google/gemma-3-4b-it`.
- Recibo: 757 tokens de entrada, 281 de salida, coste informado US$0,00004433.
- Latencia: clasificador 2254,559 ms; búsqueda 0,420 ms; total servicio 2256,293 ms; total campaña 2257,443 ms.
- El validador rechazó una cita de slot/premisa vacía o que no coincide byte por byte con un substring de la consulta. La salida cruda no se guardó, así que no se sabe cuál de las dos condiciones ni qué campo falló.
- Fue una sola petición. No se reintentó; smokes 2/3 y TRAIN no se ejecutaron.
- Próximo paso: crítica de una posible restricción de enum dinámico en JSON Schema. No ejecutar otra llamada hasta confirmar compatibilidad strict, tamaño y neutralidad semántica.
