# Lectura discriminativa con encoder base congelado

Fecha: 2026-10-22. Paso3 del objetivo, una sola candidata de investigación. V4 generativa produjo daños nuevos en126 juicios antes de un HTTP429; no se reintenta ni se rescatan sus resultados. Este experimento cambia el mecanismo de selección, sin introducir otra variante de prompt o entrenar de nuevo LoRA.

## Hipótesis y resolución previa de fuga

Una cabeza logística sobre los1024 componentes finales del reranker puede representar utilidad mejor que su lectura léxica de dos tokens. La hipótesis está sin probar. No se aplica al padre ajustado7522394b: sus176 antecedentes son exactamente TRAIN. El mapa final enlaza400 filas con ese ajuste y deja32 sin homología de plantilla demostrada; ninguna de estas cifras convierte el banco observado en datos nuevos. OOF de la cabeza no eliminaría esa exposición del encoder.

Antes de extraer features se decide usar la base original Qwen3-Reranker-0.6B, revisión e61197ed45024b0ed8a2d74b80b4d909f1255473, cargada estrictamente en FP16, sin módulos o pesos LoRA. Se reutilizan assets y runtime MLX existentes; sin descargas, instalaciones o API. Esta resolución elimina el ajuste conocido al banco, sin probar ausencia absoluta de exposición en pretraining. La propuesta inicial del padre queda archivada como no ejecutada.

El contraste dentro del mismo encoder es cabeza OOF frente a yes/no desde el mismo hidden. Frente al padre histórico cambian encoder/adaptación y readout: no es atribución causal a una sola parte ni control contemporáneo de recursos. Las decisiones históricas del padre se ligan a los mismos textos/tokens por hashes.

## Banco y partición congelada

Los432 TRAIN existentes conservan bytes, etiquetas y roles:176 antecedentes y256 pool. El mapa fuente une familias, pools mínimos, plantillas documentadas y textos exactos repetidos; las similitudes de clase semántica sin prueba de plantilla común se documentan como propuestas y no unen grupos. El cierre incorpora cuatro puentes concretos de campo ausente o componente solicitado entre f14/f16 y las plantillas correspondientes. Se conservan los mapas borradores y sus correcciones previas a inferencia. No usa scores o errores para decidir aristas. El autor había visto accidentalmente scores históricos conocidos al inspeccionar un plan; lo declara y no se presenta como agrupación ciega a toda historia.

Se obtienen cinco componentes indivisibles de312/72/32/8/8 filas. Leave-one-component-out: cinco ajustes, cada fila predicha exactamente una vez con una cabeza que no la entrenó ni vio su componente. Todos los folds de entrenamiento contienen ambas clases binarias. Los componentes no tienen los mismos roles; un fold sin C no recibe un denominador C inventado. Se reportan conteos por componente y rol, además del agregado. No se cambia la agrupación después de resultados, no hay partición aleatoria por filas ni selección de folds favorables.

La normalización media/desviación se ajusta exclusivamente en TRAIN de cada fold. Columnas con desviación<1e-8 usan escala1. Sin PCA, selección de variables, escalado global ni ajuste sobre88. Los grupos se deciden por fuentes; las etiquetas solo se usan en el ajuste y para comprobar soporte binario antes de ejecutar.

## Receta única

Features: hidden final1024 del texto íntegro con instrucción/control/tokenizador de145, sin IDs, roles o gold en tokens. Sin truncamiento, padding ni cache entre filas. Se guarda FP32 y se calcula la cabeza en FP64 CPU con intercepto. Objetivo: media de entropía cruzada binaria más0,01/2 por norma cuadrada de todos1025 parámetros, incluido intercepto. Sin pesos de clase.

Newton determinista desde cero, Hessiano mediante Woodbury en dimensión de ejemplos; máximo50 pasos aceptados por fold, gradiente infinito≤1e-7, línea Armijo1e-4, paso inicial1 y hasta30 reducciones por mitad. La búsqueda de paso es del optimizador de este objetivo fijo; no selecciona hiperparámetros por calidad. Si no converge o hay valores no finitos, error técnico; no cambiar receta tras OOF. Umbral fijo sigmoid≥0,5. Sin barrido de regularización/umbrales ni elección de checkpoint. No se ajusta una cabeza final sobre todo TRAIN si la puerta OOF falla.

Los controles CPU sintéticos verifican gradiente por diferencias finitas, dirección Newton dual frente a Hessiano primal, descenso, convergencia, simetría de etiquetas, determinismo, columnas constantes, sigmoid estable y aislamiento de normalización. No usan features reales. Los pasos de la cabeza se contabilizan separados de forwards/backwards/updates del transformer.

## Integridad, ejecución y puertas

Antes de inferencia: hashes de runtime, assets, archivos fuente, código, mapa, receta y plan; preflight independiente. El runner comprueba los310 tensores cargados contra bytes BF16 persistidos convertidos a FP16 por CPU y conserva fingerprints inicial/final. Todo el encoder permanece congelado, sin AD ni generador JSON. Una sola pasada432; cada hidden también reconstruye yes/no y verifica proyección CPU del mismo vector con rtol0,002/atol0,05. Es un control numérico, no una tasa de calidad. Guardar todos los vectores y duplicados, sin eliminar filas desfavorables.

Presupuesto:432 forwards máximos,0 backwards/updates del transformer,5 cabezas y250 pasos aceptados máximos, plazo total900s. CPU y GPU existentes, CLI temporal sin puerto, estimación2–5minutos o parada temprana. API USD0; energía/amortización/Codex desconocidos. Finitud, recibos locales/diario y término del proceso se auditan. Un timeout de observación no autoriza reiniciar.

Puerta para continuar: al menos una ganancia útil C y cero nuevas pérdidas útiles/ajenos frente al padre fijo en OOF agrupado. Reportar también comparación con yes/no base. Entrenamiento o OOF sobre este banco observado no prueba generalización fresca. Las puertas originales24/24+64/64, seis conjuntos, fidelidad, recursos≤1,5×, estabilidad y casos nuevos con revisión ciega permanecen intactas. Reservas originales/V6 cerradas. Ninguna adopción o cambio de política/default por este ensayo.

Sin cambios Rust/Cargo/apps/DB/secretos/servicios/dashboard/ramas/commit/push. Las930 pruebas HTTP2 previas no se presentan como una nueva ejecución; esta implementación experimental requiere sus controles y auditoría propios. Artefactos: outputs/iromem/evaluation/frozen-utility-readout-v1-2026-10-22/.
