# Sensibilidad a la pregunta con documento fijo

Paso3: diagnóstico de fuentes conocidas tras fallar diversidadTRAIN152 y normalización153. No es ampliación de entrenamiento, candidata nueva, puerta del paso4 o validación fresca. No se ajustan cabezas ni pesos. Objetivo: comprobar si las representaciones y lectores congelados cambian correctamente al variar lo que pregunta el usuario y conservar íntegro el documento.

## Diseño anterior a inferencias

V1con20filas/10pares se conserva sin ejecutar. Se detectó que dos parejas sustituían identidad para alinearse con otra fuente y que dos correcciones cambiaban atributo y retiraban premisa simultáneamente. V2corrige el diseño antes de cualquier puntuación:18filas/ocho documentos, con ocho originales intactos y diez preguntas contrafactuales. Exclusiones por alcance prospectivo, no por resultados nuevos. No se modifica gold original.

Seis parejas comparan dato/relación solicitado: cuatro none→A y dos A→none. Dos ternas separan premisa y dato: pregunta originalB; nuevo atributo conservando la misma atribución falsaB; misma segunda pregunta retirando solo esa atribución,none. Un cambio de atributo no elimina la corrección si sigue refutando una premisa expresa. En dos parejas el contraste cambia instancia→regla de plantilla o perfil del mismo aparato: se declaran separados, no un único factor común a los ocho grupos.

Revisión separada recibe solo{id,query,document}opacos de V2, sin autoría/roles/condiciones/etiquetas. Conservar dictamen original y toda conciliación por fuente/rúbrica antes de modelo. Misma familiaCodex con historia del proyecto, no revisión humana o independencia de familia. El estado explícito de ausencia del dato solicitado se rige por la rúbrica existente, sin inventar una regla tras puntuar. Si permanece conflicto binario o de rol, no ejecutar. No usar resultados para revisar etiquetas.

## Captura y lectores fijos

Mismo baseQwen3FP16sinLoRA,assets/runtime/instrucción/tokenización del151/152. Ocho features originales se reutilizan por hash/orden e identidad exacta de texto; solo diez consultas nuevas se codifican. No se abren reservas; fingerprints/casts310tensores/proyecciónCPU y tokens completos verificados como152. Una ejecuciónCLI local sin puerto, estimada1–2minutos y plazo máximo180s, diezforwards,ceroencoderbackwards/updates,ceroheadfits,ceroAPI. Tiempoinstrumental no es latencia productiva;Codex/energía/amortización desconocidos.

Se aplican tres lectores ya persistidos sin adaptación:control152,ampliado152 y estadísticasdelcontrol153. Para cada grupo se usa siempre el head que excluyó el componente completo del documento original y todos sus homólogos. Los nuevos textos no entraron en TRAIN de ningún lector. Verificar que las ocho predicciones originales reproducen sus referencias conΔprobabilidad≤1e-12 y mismas decisiones. Añadir yes/no base del mismo hidden para diagnóstico, sin compararlo causalmente con padreLoRAhistórico no ejecutado sobre las preguntas nuevas.

## Medición y decisión

Reportar18decisiones por lector, útiles conservados/ajenos, seisparejas completas, dos ternas completas y los dos contrastes de premisa por terna. La invariancia de corrección requiere seleccionar ambas consultas B, no basta omitir ambas. El cambio de premisa requiere conservarB y rechazar consulta sin atribución falsa. Desglosarordenación útil>ajeno y corte0,5 sin moverlo. Comparar coseno/distancia de features para documentar sensibilidad numérica, sin llamar semántica correcta a cualquier vector diferente.

Cada original y contrafactual conserva probabilidades,features,fuente/componente/lector. Diariodurable y marcador exclusivo,sin reinicio/overwrite; hashesplaninputs/lectores/código/runtime antes de captura,preflight separado previo y auditorposterior sinencoder/refit. Detectarimports/llamadas de ajuste inesperados. Un resultado perfecto aquí no supera la puerta local432ni prueba generalización. Todas las puertas originales24/24desarrollo,64/64conocidos,seis conjuntos,recursos≤1,5×base,recibos,estabilidad yV6ciega siguen intactas. Producto conservado.
