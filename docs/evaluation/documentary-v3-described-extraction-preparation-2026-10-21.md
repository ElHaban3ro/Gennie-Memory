# Nueva extracción completa V3: preparación congelada

La revisión anterior corrigió pérdidas de parciales y coherencia de alias con 656 pruebas offline, 18 controles CLI y 226 informes históricos idénticos. Esta etapa prepara la comprobación siguiente sobre textos conocidos completos. No se ha despachado ningún autor ni crítico, ni obtenido una extracción nueva. Los subagentes permanecen cerrados.

## Inventario y cambios prospectivos

Diez paquetes conservan exactamente los 116 pares tipo/hash de texto: 83 fuentes y 33 preguntas, separados en siete lotes de fuentes y tres de preguntas. Los IDs opacos y el orden cambian de forma determinista; el roster privado conserva correspondencia fuera de los paquetes. No hay raws anteriores, frames manuales, etiquetas, pares, métricas ni resultados en las entradas de autor.

El vocabulario anterior se conserva con un nuevo identificador y el perfil local_reference_descriptions_v1. La guía V4 añade restricciones por ocurrencia, diferencia Atom/Proposition/Request, contexto histórico, atribución de cláusulas, modalidad negativa, fuentes y dominios completos. La notación incorpora associated_with, described_ref y filtros reference_pattern. Se adjunta código estructural de validación, atributos y descripciones sin tests. No se adjunta el motor de selección ni se añade una tabla completa de attribute_profiles; la fidelidad dimensional debe revisarse semánticamente.

El autor entregará solo semantic-items, con el digest congelado suministrado copiado literalmente. El coordinador ensambla los bytes originales mediante la CLI y exige --assembled-only al auditar. No se pide al autor calcular el hash del paquete ni construir su wrapper; tampoco se reparan decisiones o scripts fallidos después de feedback. La vinculación mecánica no demuestra por sí sola autoría, frescura o conducta source-only.

## Puerta previa a utilidad y casos nuevos

Exigir los 116 documentos estructuralmente completos y 464 dimensiones faithful por cada uno de dos críticos en contextos nuevos separados del autor y entre sí. Cada lote tiene una sola entrega; las ausencias, unsupported y desacuerdos conservan el denominador. Los críticos reciben texto, raw, validación y contrato, sin juicios anteriores ni referencias esperadas. Es desarrollo conocido y revisión entre agentes de la misma familia; no validación humana, independencia de familia ni cohorte ciega nueva.

Solo después de esa puerta procede medir utilidad automática y comparar con la configuración vigente, además de recursos, estabilidad y finalmente V6 ciega. No se promueve nada por completar esta preparación.

## Comprobación realizada y coste

Los diez paquetes pasan el lector Rust y el ensamblado con placeholders null explícitos. Los diez derivados pasan el receptor --assembled-only; son pruebas de transporte, no extracciones ni aciertos semánticos. El inventario de tipo/hash coincide exactamente con el anterior y tiene 116 textos únicos. El validador real no encuentra errores de vocabulario en una sonda documental vacía; esa sonda tampoco acredita completitud ni representación. No hay cambios de código ni se repitió Cargo: la evidencia de 656 pruebas pertenece al snapshot 093 y sus hashes se conservan.

Cero inferencias, agentes nuevos y llamadas de proveedor en esta preparación; USD 0 de API. La ejecución completa prevista requiere diez contextos de autor y veinte de crítico, aproximadamente 30–60 minutos. Coste/tokens Codex desconocidos y no gratuitos. No se ha consumido ni reservado presupuesto de proveedor para esa ejecución.

Calidad automática vigente sin cambios: 101/116 documentos completos y 98/116 aceptados en las cuatro dimensiones por ambos críticos; puerta fallida. Tampoco cambian las métricas manuales históricas ni la configuración del producto. Acumulado de proveedor: 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos, excluyendo Codex.

Artefactos en outputs/iromem/evaluation/documentary-v3-described-extraction-2026-10-21/: protocol.json, paquetes, guía, vocabulario, contrato de revisión, roster privado, verificadores y pre-inference-manifest.json. Ese manifiesto congela solo la preparación original; las verificaciones posteriores se ligan separadamente en manifest.json. Próxima acción: ejecutar los diez lotes completos y su doble revisión según el protocolo, conservando todo fallo; ninguna inferencia está en curso. Objetivo activo en paso 3.
