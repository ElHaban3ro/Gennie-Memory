# 0033 — Ficha semántica automática solo en evaluación

Fecha: 2026-10-22. Estado: investigación experimental; V1/V2 detenidas; V3 falla el piloto; V4 parcial con daños nuevos; sin adopción.

La ablación del reranker encontró parciales útiles que el padre omite, pero las dos puntuaciones agregadas no permiten rescatarlos mediante una función global monótona sin nuevos ajenos en ese TRAIN. Una revisión nativa ciega de52fichas textuales obtuvo51decisiones concordantes y una identidad sin resolver, fiel a la fuente. Su puerta manual original no se modifica ni se declara superada.

Se prueba una sola ficha automática que explicita referente, foco solicitado, aporte y citas. Gemini2.5FlashLite es una opción económica con transporte generativo ya existente; la capacidad de este contrato sigue por demostrar. Jev/Decisions existentes devuelven choices, no acreditan extracción de citas. No se usan modelos frontera por API, fichas manuales como respuestas de runtime ni routers por gold.

La interfaz distingue include, exclude e indeterminate: solo el primero selecciona; los otros conservan razón/deuda. Un útil indeterminado es una omisión. Fallos técnicos o de recibo no se convierten en abstención semántica. La validación de citas comprueba procedencia, no verdad o pertinencia.

Preparación en tools/evaluation, ejecución manual cfg(test) en bootstrap reutilizando su transporte. Sin política/default/DB/dependencias nuevas.24paresTRAIN fijos antes de408restantes; techoUSD0,50/plazo900s, sin retries o fallback. Toda regresión del piloto detiene expansión. [Protocolo y condiciones](../evaluation/semantic-link-flash-lite-v1-protocol-2026-10-22.md).

Revisión de esta decisión: solo una candidata que preserve útil y exclusión de ajenos, supere las puertas originales, casos nuevos ciegos, recursos/recibos≤1,5× y estabilidad puede proponerse para adopción. Un fallo se conserva completo y exige diagnóstico de causa antes de una versión nueva. No se alteran gold, reservas o resultados previos.

ResultadoV1:detención técnica tras1HTTP/cita no literal;0juicios válidos,USD0,0001028,sinadopción o cambio degold. Se investiga referencia a spans canónicos, sin reparar retrospectivamente V1. [Informe](../evaluation/semantic-link-flash-lite-v1-results-2026-10-22.md).

V2separa referencias de texto libre: el modelo selecciona IDs de tokens existentes y el adaptador compone citas literales; el foco se vincula aquery. Esto corrige la ruta de recopia y copia de valor exclusivo de documento sin acreditar foco o evidencia fieles. Mismo modelo, datos, piloto y puertas; nueva interfaz de ocho campos y coste recalculado, intento separado, sin repararV1. [Protocolo](../evaluation/semantic-link-span-v2-protocol-2026-10-22.md).

V2 detenida por contradicción en respuesta doce; 11 selecciones correctas parciales no acreditan fidelidad (3/12 fieles,6no sustentadas,3ambiguas). USD0,0013256; sinadopción o cambio degold. [Informe](../evaluation/semantic-link-span-v2-results-2026-10-22.md).

V3 propone siete campos remotos y política local explícita, manteniendo fuentes y referenciasV2. Añade distinciones generales de identidad/atributo, ausencia explícita y evidencia completa; no valida facetas falsas por construir una decisión coherente. Conserva datos/puertas y los fallos anteriores. [Protocolo](../evaluation/semantic-link-span-v3-protocol-2026-10-22.md).

V3 termina el piloto con 24 fichas válidas, 12/12 útiles preservados y 8 ajenos, incluidos 2 nuevos: falla la puerta de calidad. Derivar la decisión localmente elimina redundancia, pero no verifica categorías semánticas. USD 0,0031114; sin adopción o cambio de gold. Se prepara una candidata directa de aportación textual, con referencias documentales y sin taxonomía intermedia; una sola prueba de esta hipótesis, con todas las puertas intactas. [Informe](../evaluation/semantic-link-span-v3-results-2026-10-22.md).

V4 ensaya clasificación directa de aportación textual: cuatro campos, evidencia documental y decisión del modelo. Se eliminan categorías y mapa de pregunta; salida máxima512. Cambios conjuntos, sin atribución causal aislada. Un piloto adverso detiene expansión; ninguna excepción a las puertas de fidelidad/calidad/recursos. [Protocolo](../evaluation/semantic-link-direct-v4-protocol-2026-10-22.md).

V4 pasa preservación del piloto22/24, pero el prefijo ampliado101/126 añade6 pérdidas y9 ajenos nuevos. HTTP429 del pool compartido en petición127, sin retry;USD0,0118199 conocidos+1 operación sin importe. El fallo técnico no explica los daños semánticos. Sin adopción o cambio de gold. Se abandona esta rama de cambios de esquema para probar lectura discriminativa de representaciones congeladas con OOF agrupado y puertas intactas. [Informe](../evaluation/semantic-link-direct-v4-results-2026-10-22.md).
