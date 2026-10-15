# Respuestas y requisitos de inicialización

Estado: recopilación inicial. Última actualización: 2026-10-15 (America/Bogota).

## Datos expresados por el usuario

- El usuario proporcionó el prompt íntegro de CreationTool para definir e iniciar una aplicación en Rust.
- El nombre de la aplicación es `iROMEM`.
- `/Users/fer/Documents/Projects/Fer/Dev/iROMEM` es la carpeta de destino definitiva.
- El usuario indicó que proporcionará después la descripción de la aplicación y la necesidad que resuelve.
- El usuario aportó `/Users/fer/Documents/Projects/Fer/My Space/iROMEM.md` como especificación técnica extensa del producto y pidió usarla como base del README y de un roadmap guardado en el repositorio.
- El MVP debe implementar la recuperación de recuerdos semilla con clasificación por IA, índices estructurados, textuales y semánticos; debe admitir OpenRouter y comparar Laya, Jev y OpenAI Decisions.
- Corrección posterior del usuario: «Jeff» era **Jev** por un error de traducción y la otra IA mencionada era **Laya**, no LLaMA.
- El usuario quiere un conjunto amplio de evaluación para medir eficiencia, tiempo ahorrado, ventajas y desventajas de cada enfoque.
- El usuario autorizó inicializar un repositorio Git local en la carpeta definitiva.
- El nombre técnico definitivo es `iromem`.
- El MVP correrá en esta Mac. Laya reutilizará el modelo ya descargado por SOC en `/Users/fer/Library/Application Support/AIModels`; no se debe descargar otro por defecto ni modificar el modelo de SOC.
- Se permite enviar datos reales a OpenRouter, Jev y OpenAI, pero **todas las pruebas de evaluación usarán datos sintéticos generados para iROMEM**.
- El MVP inicial **no tendrá hook**. Codex será el host de una integración posterior; la meta inmediata es validar la resolución de memorias semilla.
- El repositorio indicado por el usuario es `git@github.com:ElHaban3ro/iROMEM.git`.
- El usuario pidió configurar ese repositorio como remoto `origin`.
- El usuario indicó que ya proporcionó una clave de OpenRouter y advirtió que quizá sea necesario ajustar el nombre de la variable. No se recibió ni se guardó su valor en estos archivos.
- La clave de OpenRouter se proporcionó inicialmente bajo `openrouter_apikey`; por indicación posterior del usuario se adoptó `IROMEM_OPENROUTER_API_KEY`. Su valor es secreto y no se transcribe aquí.

## Contexto observado (no es una respuesta del usuario)

- Directorio de trabajo disponible y vacío al iniciar: `/Users/fer/Documents/Projects/Fer/Dev/iROMEM`.
- En ese directorio no existían `.creation`, `AGENTS.md`, un proyecto Cargo ni un repositorio Git.
- El nombre técnico `iromem` quedó confirmado después de la propuesta inicial.
- La especificación se autodenomina diseño propuesto v0.1; sus parámetros iniciales no constituyen resultados medidos.
- Se observó una caché local `models--convaiinnovations--laya` con snapshot multilingüe bajo la ruta de AIModels indicada. Esto verifica su presencia en disco, no que el runtime de iROMEM ya pueda cargarlo ni que esté completo para todos los usos.
- En el entorno del proceso actual no apareció un nombre de variable que contenga `OPENROUTER`, `IROMEM` o `LAYA`; esto no demuestra que la clave no exista en otro almacén o sesión.

## Requisitos confirmados

- Conservar el prompt completo en `.creation/initialization-prompt.md` y registrar respuestas, decisiones y plan antes de construir la base.
- Definir el producto mediante preguntas breves antes de crear el workspace, apps, migraciones o frontend.
- Usar el nombre `iROMEM` y este directorio como destino definitivo de la aplicación.
- Conservar la especificación como README y sus archivos locales vinculados; preparar un roadmap separado centrado primero en recuperación y evaluación.
- Git local inicializado y `origin` configurado con la URL SSH proporcionada. No se han realizado commit ni push.
- Validar primero la resolución de memorias semilla en esta Mac, con pruebas sintéticas y sin hook ni integración Codex inicial.
- Usar el modelo Laya ya descargado por SOC, sin alterar los archivos compartidos.
- El 2026-10-15 el usuario autorizó continuar con todo el trabajo pendiente sin volver a pedir permiso; se ejecutaron las campañas remotas dentro de los planes de coste, duración y corpus sintético documentados.

## Preguntas pendientes

1. Los indicadores, corpus, presupuesto y duración de la campaña remota de desarrollo se fijaron y el usuario autorizó expresamente continuar sin nuevas solicitudes de permiso el 2026-10-15. La campaña de 2.800 llamadas y el sondeo adversarial de 800 llamadas terminaron; los umbrales de producto aún requieren calibración.
2. Elegir modelos concretos de OpenRouter y el perfil de embeddings tras comprobar disponibilidad, compatibilidad y seguridad de dependencias al incorporarlos.
3. Para una integración posterior: definir el contrato con Codex, permisos y comportamiento del hook. No bloquea la validación local del MVP.
4. No se han realizado commit ni push; son independientes de la validación local y remota.
5. La disponibilidad de `IROMEM_OPENROUTER_API_KEY` quedó comprobada sin mostrar su valor. Las tarifas/modelos se verificaron antes de la campaña; el adaptador interno de producto sigue pendiente.

## Datos de repositorio

- URL: `git@github.com:ElHaban3ro/iROMEM.git` (sin credenciales).
- Ruta local del checkout: `/Users/fer/Documents/Projects/Fer/Dev/iROMEM`.
- Rama: `main`, valor efectivo tras `git init`; sin commits.
- Remoto: `origin` configurado para fetch y push con `git@github.com:ElHaban3ro/iROMEM.git` (comprobado con `git remote -v`).
- Proyecto nuevo o existente: repositorio Git local nuevo sobre la carpeta de trabajo existente; `.creation/` se conservó.
