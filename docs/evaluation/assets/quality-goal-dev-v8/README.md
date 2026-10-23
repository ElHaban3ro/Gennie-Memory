# V8 de desarrollo — paquete ciego

Conjunto nuevo de desarrollo sintético: 4 conversaciones, 20 mensajes/recuerdos y 16 consultas, cuatro por cada una de las relaciones apoyo, refutación, parcialidad e irrelevancia. Las consultas comparten vocabulario/ámbito con distractores para que FTS no equivalga a evidencia.

Antes de usarlo con un clasificador, entregar a cada revisor únicamente `source.json` y `blind-questions.json` en una copia sin la clave. Cada dictamen debe citar el mensaje fuente, clasificar la relación y señalar límites. Conciliar desacuerdos y congelar hashes antes de cualquier inferencia. `answer-key.json` es material de autoría y nunca se adjunta a requests.

Este banco no es humano ni confirmatorio. No abre V6, no es conjunto de prueba ciega de adopción y no autoriza cambiar etiquetas tras observar salidas. No ejecutar proveedor hasta que la revisión ciega esté conciliada y exista un protocolo local de candidata fijado.

{
  "source.json": "294b8962ba7506d799d34453678ab1e030eb4172809d3981dd3c58a3919745eb",
  "memory-corpus.json": "2e88eae73c5d5b9e970f40d2c3539c59e18c5c3efac4fa211a40309f7888ab10",
  "blind-questions.json": "3fe257694172581d23fd36d87c64bf35ec8ce2eecb41c1b85b10b3a040f0a5ce",
  "answer-key.json": "c93f9f19e7ce6ab22a2ffac7d4f7a494f40046cde4a2ff86f30e73bd323a6d6c"
}
