# Revisión 33 — smoke sintético del contrato de spans

Nueva identidad, consulta, valor y ámbito; no reintenta r31 ni r32. Prueba una instrucción de sujeto con límites de identificador explícitos. Gemma 3 4B es experimental; Decisions predeterminado no cambia.

Puerta previa: top-1 local, anotación ciega del par consulta/recuerdo y hashes congelados de solicitud/corpus/fuente/binario. Si se ejecuta el smoke, son tres operaciones con misma solicitud/corpus y recibos obligatorios. Parar en el primer fallo, sin reintentos. Esta tanda solo evalúa estabilidad del adaptador; no calidad poblacional ni adopción.
