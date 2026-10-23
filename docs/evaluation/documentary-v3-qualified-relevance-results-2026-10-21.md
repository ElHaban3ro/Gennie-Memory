# Contexto de relevancia con identidad pendiente en V3

Una relación declarada `supplied_by→upstream_origin` no conservaba el depósito que abastece Olmo porque la fuente no declaraba su clase de parque y el agua era una referencia anónima. El comparador ahora permite transportar **la proposición original completa como contexto parcial**, sin afirmar identidad documental ni publicar la tubería solicitada.

La nueva vía requiere un enlace de relevancia ya declarado, todos sus bindings compatibles y al menos un anclaje nominal acreditado entre esos bindings. Una referencia anónima debe satisfacer un patrón descriptivo local con clase/tipo, denotación y enlaces exigidos; la semejanza no demuestra identidad global. Un término no referencial desconocido, clase explícita distinta, nombre distinto, anáfora o conflicto no habilitan esta vía. No se añaden nombres ni enlaces al vocabulario del banco.

Se limita a proposiciones positivas fact/state y preguntas answer/fact. Se conservan los controles de contexto, ámbito normativo, asociaciones requeridas, dependencia, rama, fundamento y premisas. La publicación lleva `answer_terms:[]`, `answer_propositions:[original]`, `context_identity_proven:false`, identidad y slot pendientes, además del candidato sin resolver. La vía estricta anterior conserva sus resultados y términos.

| Medida sobre los mismos 100 pares manuales | Anterior | Nuevo |
| --- | ---: | ---: |
| Útiles seleccionadas /39 no ambiguas | 28 | 29 |
| Referencias none seleccionadas /60 | 0 | 0 |
| Respuestas completas | 3 | 3 |
| Concordancia con selección heredada /100 | 88 | 89 |

La única ganancia es `pcv1_08_01/claim_01`: «Olmo recibe agua del depósito de lluvia K» para la pregunta por la tubería de la red. Se conserva como contexto, sin convertir K en tubería ni afirmar que no exista alimentación desde la red. Cero pérdidas útiles y de respuestas completas; no se selecciona la referencia ambigua. Quedan **10 omisiones útiles** (7A,2B,1C), registradas individualmente antes/después. Otras omisiones necesitan relaciones, estado de medición, contexto o representación adicionales; no se fuerza su aceptación para cumplir etiquetas.

La comparación ejecuta ambos binarios sobre entradas, vocabulario y referencias idénticos. Los 100 informes del binario anterior coinciden con la etapa previa. Nueve controles comprueban anclaje con clase pendiente, otro destino, destino anónimo sin anclaje, clase anónima ausente en cada lado, rama, fundamento, enlace ausente y clase del destino resuelta. Son desarrollo manual y mutaciones mecánicas del grafo, no extracción fiel ni revisión ciega. Ambos documentos se exigen completos en cada control; la versión anterior omite ambos positivos.

Pasan **604 pruebas,24omitidas**. El conjunto V3 comprende82del motor/validador y2del adaptador CLI,84en total. Compilación y formato pasan. El hash final reproduce233informes idénticos:100pares nuevos,9controles propios,8controles de alcance anteriores y116validaciones de extracción congelada. Una aserción del script de verificación contó inicialmente solo el módulo del motor contra el total que incluía adaptador; se conserva ese fallo de contabilidad y se corrige el recuento, sin cambios de código ni expectativas de comportamiento. La auditoría del witness confirma proposición íntegra, ausencia de origen publicado e identidad pendiente.

**La puerta global sigue fallando.** No se midió extracción automática nueva: la última sigue en65/116fieles para ambos críticos. No hay revisión semántica independiente nueva, comparación de producto/baseline, latencia, estabilidad ni V6. No se reactivaron subagentes. El siguiente trabajo requiere resolver las omisiones restantes con controles, seguido de extracción real y validación completa antes de generalización o adopción.

Coste nuevo: **0peticiones API y USD0 de proveedor**; coste/tokens Codex desconocidos. Acumulado intacto:6129operaciones,23campañas,USD4,565416508conocidos más20importes desconocidos,excluyendo Codex. El producto conserva baseline; objetivo activo en paso3.

[Propuesta](../../outputs/iromem/evaluation/documentary-v3-qualified-relevance-2026-10-21/proposal.json), [protocolo](../../outputs/iromem/evaluation/documentary-v3-qualified-relevance-2026-10-21/protocol.json), [comparación](../../outputs/iromem/evaluation/documentary-v3-qualified-relevance-2026-10-21/report.json), [ejecución](../../outputs/iromem/evaluation/documentary-v3-qualified-relevance-2026-10-21/execution-provenance.json) y [controles](../../tests/fixtures/documentary-v3-qualified-relevance-controls-v1.json).
