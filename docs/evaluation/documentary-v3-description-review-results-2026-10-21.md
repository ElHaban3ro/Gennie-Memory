# Revisión crítica de descripciones V3: fallos reproducidos y corregidos

Se revisó por separado la extensión de descripciones locales y se corrigieron causas concretas antes de otra extracción. La implementación inicial no estaba lista: tenía un ciclo indirecto capaz de abortar la CLI, admitía una descripción internamente contradictoria como respuesta completa y perdía evidencia por tratar algunos operandos descritos como carriers distintos. La revisión también encontró un parcial sin ancla acreditada. Se preservan las críticas, reproducciones negativas e iteraciones; no se modifican raws ni etiquetas históricos.

## Causas y correcciones comprobadas

- Un guard interno por ID podía volver a la misma descripción aunque el JSON y dependencies fueran finitos. La CLI hija original terminó con returncode -6 y stack overflow, sin informe. Ahora readiness corta guards antes de comparar contextos y el validador marca unsupported_description. Los carriers con esa deuda quedan fuera de la vista ejecutable y se conservan como candidatos con host íntegro. La consulta pendiente retorna antes de comparar. Ciclos de uno y dos guards terminan sin selección ni respuesta completa.
- La conjunción P y no-P del mismo átomo/contexto se emparejaba consigo misma como Same. El control original produjo complete_answer true. Ahora esa contradicción y cantidades exactas incompatibles impiden resolver la descripción; se preservan contenido, deuda y claims independientes. La equivalencia de aliases explícitos no oculta la contradicción. Algunos hosts contradictorios pueden conservarse como contexto parcial calificado, sin soporte completo ni publicación de propiedades; no se presenta eso como una respuesta resuelta.
- El filtro reference_pattern descartaba described_ref antes de examinar su raíz. Ahora comprueba clase/enlaces de la raíz preparada y conserva el término completo. Se verifican respuestas descritas y mezclas de respuestas independientes; un miembro no acreditado no elimina los acreditados.
- Los parciales ref/described perdían asociaciones válidas y los objetivos descritos podían eludirlas. Ahora ambos carriers pasan los mismos requisitos de asociación. Una fuente anónima sin ancla ya no obtiene un parcial por el atajo histórico. Un anclaje descriptivo parcial exige correspondencia nominal positiva, contexto y ausencia de clases/tipos conocidos incompatibles en la pareja de aliases utilizada. description_anchor transporta nombres citados, IDs y ambos términos; same_description indica correspondencia local completa, sin afirmar identidad global nueva.
- Una cantidad común universal perdía el operando descrito tras comprobar alpha. Ahora el descargador reutiliza mapped_term y exige la variable real del dominio. El control conserva 6 kg bajo un dominio universal verificado; un objeto constante o procedencia no coincidente no completa esa respuesta. No se elimina ningún descriptor ni calificador para lograrlo.

## Verificación

| Alcance | Resultado |
| --- | --- |
| Suite offline completa | 650 pasan; 24 omitidas; 0 fallan |
| Pruebas de descripción / nuevas respecto del estado anterior | 23 / 11 pasan |
| Total de pruebas del motor V3 | 114 pasan |
| Controles CLI manuales anteriores | 12/12 |
| Regresiones CLI nuevas con documentos no inválidos | 6/6 |
| Informes históricos idénticos en bytes | 226/226 |
| Compilación y formato | pasan |

Los 226 son los mismos 100 pares manuales, 116 validaciones y diez recibos congelados. Se comprobaron hashes y el inventario íntegro de las tandas anteriores, sin archivos añadidos o reemplazados. La prueba de contradicción y los dos controles de asociaciones/anónimo fallaron antes de las correcciones y pasan después. Las otras rutas tienen crítica estática y controles posteriores; no se inventa una comparación ejecutada anterior para ellas.

Una corrección intermedia evitó el aborto pero aún seleccionaba el guard como parcial. Se conservó ese informe y se corrigió contra el criterio CLI previamente fijado: ahora permanece candidato pendiente, sin selección útil. La última crítica encontró una pérdida de payload cuando coincidían attribute_profile_mismatch y unsupported_description: se conservan ambos motivos y request_constraint íntegro, verificados con una prueba nueva por el coordinador.

El primer driver de compatibilidad buscó el protocolo en la carpeta nueva; abortó antes de ejecutar CLI. Un primer protocolo de regresión además omitía arrays de evidencia y dependencias de enlaces; el caso mixto quedó invalid y las ausencias no habrían sido prueba no vacua. Se preservaron protocolo/salidas iniciales y se corrigió solo la preparación, con expectativas intactas. La versión final exige documentos no inválidos en cada caso; las seis regresiones pasan.

## Revisión separada y límites

Un contexto crítico Codex nuevo produjo cuatro turnos de revisión estática; los anteriores permanecen cerrados. No ejecutó Cargo, pruebas ni APIs y no leyó fixtures o resultados. Hubo exposición accidental inicial a fragmentos de tests por búsquedas en archivos completos, registrada inmediatamente; las lecturas posteriores se limitaron a producción. También recibió mensajes sobre reproducciones del coordinador. Esto es revisión separada de código de la misma familia, no medición ciega de extracción ni independencia de familia.

Los cuatro dictámenes conservados identifican fallos y luego acotan las correcciones. No quedan bloqueos circulares concretos en los caminos revisados; la corrección menor de dos deudas simultáneas fue probada por el coordinador después del último dictamen, sin atribuir esa comprobación al crítico. No hay certificación general de calidad.

Persisten límites que deben contrastarse antes de una mejora global: parciales con raíces anónimas bajo alpha verificado pueden perderse por la nueva exigencia nominal; la coherencia global de clases/tipos de todos los aliases no está probada, solo la pareja nominal utilizada. El matching general de calificadores alpha-renombrados sigue pendiente. No se vuelve al atajo anónimo ni se relajan las puertas para evitar esos límites.

## Calidad, coste y siguiente trabajo

No hubo nueva extracción ni revisión de contenidos extraídos. Permanecen 101/116 representaciones completas estructurales y 98/116 aceptadas por ambos críticos, puerta fallida. Las 18 comprobaciones CLI son manuales y no reemplazan esas métricas. La utilidad manual histórica sigue en 36/39 no ambiguas, cero selecciones en 60 none y tres respuestas completas. No se midieron utilidad automática, baseline, recursos/estabilidad del producto ni V6. El objetivo continúa en el paso 3, sin adopción ni cambios del predeterminado.

Cero llamadas nuevas de proveedor y USD 0 de API. Un contexto crítico, cuatro turnos, coste/tokens Codex desconocidos y fuera del ledger de proveedor; no se declara gratuito. Acumulado intacto: 6.129 operaciones lógicas, 23 campañas emparejadas, USD 4,565416508 conocidos y 20 importes desconocidos. Las ejecuciones CLI tuvieron límite de observación por proceso; no son mediciones de latencia de producción ni ahorro de tokens.

Siguiente trabajo: verificar los límites de parciales cuantificados/aliases con controles genéricos, conservando la prohibición de anclas anónimas individuales. Después integrar la guía prospectiva de Atom/Proposition, atributos locales, citas y condiciones históricas en una nueva preparación congelada de los 116 textos. Exigir revisión completa separada antes de utilidad automática y comparación con la base; V6 sigue reservada. No se escoge un subconjunto favorable ni se repara la tanda anterior.

Artefactos: `outputs/iromem/evaluation/documentary-v3-description-review-2026-10-21/`, incluidos cuatro dictámenes originales, fallos, protocolos, controles, reproducciones y verification.json.
