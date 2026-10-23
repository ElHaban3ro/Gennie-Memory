# Expansión de entrenamiento: estados documentales y pools mixtos

Fecha: 2026-10-22. Preparación prospectiva de datos, sin entrenamiento o puntuación. V2 preserva24/24útiles train pero14/17conocidos y11/12desarrollo; reduce ajenos pero no supera calidad. No se barre el peso/margen de la penalización. Hipótesis: el banco pequeño no enseña con suficiente variedad la frontera entre estado documental útil, dato ajeno y asociación sin respuesta; ampliar cobertura ofrece un contraste posterior distinto. No se afirma que la falta de datos sea causa única.

## Alcance congelado antes de redactar

128pares nuevos exclusivamente para entrenamiento:32preguntas×4documentos, cuatro familias nuevas con8preguntas cada una. Cada familia usa cantidades de útiles0,1,2,3,0,1,2,3por pool, posiciones mezcladas; no enviar al modelo IDs/familia/pool/etiquetas/razones. Total48útiles/80irrelevantes. Reutilizar pregunta exactamente dentro de su pool, pero ningún documento se repite. El modelo recibirá cada pregunta/documento aislado; agrupar permite comprobar cardinalidad y revisar ausencia/múltiples útiles. No elegir top1como sustitución de seleccionar todos los útiles.

| Familia | Construcción de entrenamiento nueva |
| --- | --- |
| f13 | Estado explícito del campo solicitado: dato no localizado, ilegible o no registrado, frente a falta de otro campo o dato de otro objeto |
| f14 | Hecho/estado del registro solicitado frente a que su plantilla o formulario permita omitirlo; opcionalidad de campo no determina el valor ni su ausencia efectiva |
| f15 | Atributo escalar de un referente simple frente a otro referente cercano con mismo atributo/valor; incluir parciales concretos y negación explícita de una premisa del mismo caso |
| f16 | Configuración escalar contrastada explícitamente entre dos perfiles del mismo objeto frente a perfil ajeno aislado, con datos pedidos parciales o estado limitado sin exclusividad inventada |

Autores reciben esta rúbrica y producen textos nuevos sin leer bancos, archivos de resultados o errores individuales históricos, closed o V6. No nombres/factos/paráfrasis de conocidos en entrenamiento. No construir casos por periodos temporales comparados, requisitos de actor/acción o intercambio de roles sujeto/objeto entre participantes: esas construcciones están reservadas en el banco cerrado anterior. El revisor de separación que ya vio closed puede comprobar solapamientos de construcción sin divulgar textos/etiquetas cerradas a raíz/autores. Si hay solapamiento, registrar antes de entrenar; no afirmar separación de familias sin prueba. No abrir V6ni para deduplicar.

## Rúbrica y esquema

Fila: id,family_id,pool_id,split='train_extension',query,document,label(useful|irrelevant),role(A|B|C|none),rationale. Fuente sintética completa en document, sin datos reales o conocimiento externo supuesto. A aporta valor/estado explícito del atributo solicitado; que el valor no conste o esté ilegible puede ser un estado útil del mismo campo. B refuta una premisa necesaria mediante incompatibilidad explícita del mismo caso. C aporta parte, límite o contraste pertinente sin afirmar completitud/exclusividad. none: otro referente/atributo, silencio que no niega, mera asociación, opcionalidad de plantilla que no determina estado del registro, perfil ajeno aislado. Useful=A/B/C. El contenido debe sostener la etiqueta sin una regla inventada tras puntuar.

En pools cero, todos los documentos siguen siendo plausibles y temáticamente próximos, pero ninguno aporta un valor/estado/refutación/parcial de la pregunta. En pools multi, cada útil es pertinente por sí mismo, sin exigir que otro documento lo legitime o resolver coreferencias entre documentos. Incluir A/B/C en cada familia cuando su construcción permita el rol, sin forzar refutación en estados desconocidos o exclusividad en perfiles compatibles. Query/document sin números de caso, pistas de conteo, etiquetas o rationale. Negación, longitud, posición y nombres no deben predecir utilidad.

## Autoría, revisión y congelación

AutorA:64paresf13–f14; autorB:64paresf15–f16. IDs `x_f13_q01_m01`..`x_f16_q08_m04`; pools `x_f13_q01`,etc. Cada autor escribe su `*-author-v1.json` y proyección `*-blind-v1.json` únicamente id/family_id/pool_id/split/query/document. No leer al otro autor. Crítico separado recibe únicamente los blind y la rúbrica ciega separada sin cuotas/cardinalidades esperadas; no lee este protocolo de autoría ni etiquetas/razones y emite id,label,role,rationale,ambiguity. Preservar ambos originales y conciliar TODOSlos desacuerdos antes de entrenar o puntuar. Misma familia de agentes, no revisión humana o independencia de familia de modelos.

Validar esquema,conteos,IDs,cardinalidades,proyección ciega,unicidad,ausencia de duplicados exactos normalizados con128nuevos/48train/24desarrollo/64conocidos. Raíz no deduplica abriendo closed; revisor separado comprueba separación sin publicar contenido. Un desacuerdo no se elimina para conseguir pase; cambios prospectivos conservan original/motivo y tamaño. Si se pierde separación del cerrado, registrar la limitación y preparar reserva nueva antes de afirmar validación familiar; no disimularla.

Un pase de datos no autoriza una corrida sin protocolo: preparar después entrenamiento con datos finales por SHA, regla única de mezcla con48trainoriginales, hiperparámetros/épocas/pasos/control y coste. Desarrollo24y64conocidos siguen observados/regresión, nunca train. Closed24yV6siguen cerrados bajo las puertas originales. La expansión cambia tamaño/distribución: una corrida posterior no se atribuye exclusivamente a un factor sin control correspondiente. Sin scores de estos128en esta fase.

Salida: `outputs/iromem/evaluation/utility-learning-expansion-v1-2026-10-22/`. Sin APIs,inferencia de modelo,instalaciones,entrenamiento,Rust,Cargo,DB,secreto,servicios,rama,commitopush. Coste APIUSD0; autoría/revisión Codex no cuantificados. Entrenamiento/adopción y todos los criterios originales siguen pendientes.
