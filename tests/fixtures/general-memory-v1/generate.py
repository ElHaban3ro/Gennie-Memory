#!/usr/bin/env python3
"""Generate a deterministic, wholly synthetic multi-scope memory corpus."""

from __future__ import annotations

import hashlib
import json
from collections import Counter
from pathlib import Path


ROOT = Path(__file__).resolve().parent
AS_OF = "2026-10-15T12:00:00.000Z"


def topic(key, memory_type, statement, question, values, condition=None):
    return (key, memory_type, statement, question, values, condition)


PERSONAL = [
    topic("legal_name", "identity", "El nombre completo de {subject} es {value}.", "¿Cuál es el nombre completo de {subject}?", ["Aurelia Sol", "Bruno Mar", "Clara Luz", "Diego Río"]),
    topic("preferred_name", "preference", "{subject} prefiere que le llamen {value}.", "¿Cómo prefiere {subject} que le llamen?", ["Auri", "Bru", "Clar", "Digo"]),
    topic("language", "preference", "{subject} prefiere conversar en {value}.", "¿En qué idioma prefiere conversar {subject}?", ["español", "inglés", "portugués", "francés"]),
    topic("writing_tone", "preference", "{subject} prefiere respuestas de tono {value}.", "¿Qué tono prefiere {subject} en las respuestas?", ["directo", "cercano", "formal", "didáctico"]),
    topic("timezone", "fact", "La zona horaria de {subject} es {value}.", "¿Qué zona horaria usa {subject}?", ["America/Bogota", "America/Lima", "Europe/Madrid", "America/Mexico_City"]),
    topic("home_city", "fact", "{subject} vive en la ciudad ficticia de {value}.", "¿En qué ciudad ficticia vive {subject}?", ["Puerto Alba", "Valle Nube", "Monte Claro", "Costa Cedro"]),
    topic("work_role", "fact", "El cargo de {subject} es {value}.", "¿Qué cargo tiene {subject}?", ["diseñador de producto", "analista de datos", "editor técnico", "coordinador de operaciones"]),
    topic("team", "relationship", "{subject} pertenece al equipo {value}.", "¿A qué equipo pertenece {subject}?", ["Aurora", "Delta", "Lumen", "Bosque"]),
    topic("contact_channel", "preference", "{subject} prefiere recibir mensajes de trabajo por {value}.", "¿Por qué canal prefiere mensajes de trabajo {subject}?", ["correo", "chat", "tablero de tareas", "videollamada"], "Mensajes de trabajo"),
    topic("meeting_window", "preference", "{subject} prefiere reuniones a las {value}.", "¿A qué hora prefiere reuniones {subject}?", ["09:00", "10:30", "14:00", "16:00"]),
    topic("focus_window", "routine", "{subject} reserva {value} para trabajo concentrado.", "¿Cuándo reserva {subject} tiempo de concentración?", ["las mañanas", "las tardes", "los martes", "los jueves"]),
    topic("calendar_style", "preference", "{subject} quiere invitaciones de calendario con {value}.", "¿Qué debe incluir una invitación para {subject}?", ["agenda previa", "enlace de contexto", "duración estimada", "objetivo claro"]),
    topic("report_format", "preference", "{subject} prefiere informes en {value}.", "¿En qué formato prefiere informes {subject}?", ["Markdown", "PDF", "hoja de cálculo", "texto plano"]),
    topic("date_format", "preference", "{subject} prefiere fechas en formato {value}.", "¿Qué formato de fecha prefiere {subject}?", ["AAAA-MM-DD", "DD/MM/AAAA", "día mes año", "mes día año"]),
    topic("measurement_units", "preference", "{subject} prefiere medidas en {value}.", "¿Qué unidades prefiere {subject}?", ["sistema métrico", "sistema imperial", "metros y kilogramos", "centímetros y gramos"]),
    topic("accessibility", "preference", "{subject} necesita documentos con {value}.", "¿Qué ajuste de accesibilidad necesita {subject} en documentos?", ["alto contraste", "texto seleccionable", "títulos navegables", "descripciones de imágenes"]),
    topic("notification_rule", "constraint", "{subject} no quiere notificaciones durante {value}.", "¿Cuándo no quiere notificaciones {subject}?", ["el descanso nocturno", "las reuniones", "el trabajo profundo", "los fines de semana"], "Durante el período indicado"),
    topic("diet", "preference", "{subject} prefiere comidas {value}.", "¿Qué comida prefiere {subject}?", ["vegetarianas", "sin lácteos", "con poca sal", "sin picante"]),
    topic("drink", "preference", "{subject} suele pedir {value} en reuniones.", "¿Qué bebida suele pedir {subject} en reuniones?", ["agua", "té", "café descafeinado", "limonada"]),
    topic("travel_seat", "preference", "{subject} prefiere asiento de {value} al viajar.", "¿Qué asiento prefiere {subject} al viajar?", ["ventanilla", "pasillo", "primera fila", "zona tranquila"], "Al viajar"),
    topic("commute", "routine", "{subject} suele desplazarse en {value}.", "¿Cómo suele desplazarse {subject}?", ["tren", "bicicleta", "autobús", "a pie"]),
    topic("learning_goal", "goal", "{subject} quiere aprender {value} este año.", "¿Qué quiere aprender {subject} este año?", ["italiano", "fotografía", "programación", "cerámica"]),
    topic("hobby", "fact", "La afición de {subject} es {value}.", "¿Cuál es la afición de {subject}?", ["senderismo", "ajedrez", "jardinería", "música"]),
    topic("backup_rule", "procedure", "{subject} guarda copias de sus documentos en {value}.", "¿Dónde guarda copias {subject}?", ["disco externo", "carpeta cifrada", "servidor doméstico", "almacenamiento local"]),
    topic("review_cadence", "routine", "{subject} revisa sus objetivos {value}.", "¿Con qué frecuencia revisa objetivos {subject}?", ["cada semana", "cada mes", "cada trimestre", "cada semestre"]),
]

PROJECT = [
    topic("owner", "relationship", "La persona responsable de {subject} es {value}.", "¿Quién es responsable de {subject}?", ["Nora", "Leo", "Iris", "Teo"]),
    topic("objective", "goal", "El objetivo de {subject} es {value}.", "¿Cuál es el objetivo de {subject}?", ["reducir tiempos de revisión", "ordenar los archivos", "mejorar la búsqueda", "automatizar informes"]),
    topic("deadline", "fact", "La entrega de {subject} vence el {value}.", "¿Cuándo vence la entrega de {subject}?", ["2027-01-15", "2027-02-20", "2027-03-10", "2027-04-05"]),
    topic("status", "fact", "El estado actual de {subject} es {value}.", "¿Cuál es el estado de {subject}?", ["en diseño", "en implementación", "en revisión", "en piloto"]),
    topic("budget", "constraint", "El presupuesto sintético de {subject} es {value}.", "¿Cuál es el presupuesto sintético de {subject}?", ["120 unidades", "240 unidades", "360 unidades", "480 unidades"]),
    topic("reporting", "procedure", "{subject} reporta avances {value}.", "¿Con qué frecuencia reporta avances {subject}?", ["cada lunes", "cada viernes", "cada dos semanas", "al cierre de cada fase"]),
    topic("repository", "fact", "El repositorio ficticio de {subject} se llama {value}.", "¿Cómo se llama el repositorio ficticio de {subject}?", ["atlas-demo", "brisa-demo", "cauce-demo", "duna-demo"]),
    topic("test_strategy", "procedure", "{subject} valida entregas con {value}.", "¿Cómo valida entregas {subject}?", ["pruebas unitarias", "revisión por pares", "pruebas de integración", "casos manuales"]),
    topic("release_gate", "constraint", "{subject} requiere {value} antes de publicar.", "¿Qué exige {subject} antes de publicar?", ["aprobación de diseño", "pruebas verdes", "revisión de seguridad", "visto bueno del cliente"], "Antes de publicar"),
    topic("client", "relationship", "El cliente ficticio de {subject} es {value}.", "¿Quién es el cliente ficticio de {subject}?", ["Estudio Nube", "Casa Río", "Grupo Pino", "Taller Sol"]),
    topic("deliverable", "fact", "El entregable principal de {subject} es {value}.", "¿Cuál es el entregable principal de {subject}?", ["una guía", "un prototipo", "un informe", "una biblioteca"]),
    topic("risk", "observation", "El riesgo principal registrado para {subject} es {value}.", "¿Qué riesgo principal se registró para {subject}?", ["retraso de datos", "cambio de alcance", "dependencia externa", "baja cobertura"]),
    topic("design_decision", "decision", "{subject} decidió usar {value} para el piloto.", "¿Qué decidió usar {subject} para el piloto?", ["SQLite", "archivos JSON", "un servicio local", "una cola simple"]),
    topic("dependency", "fact", "{subject} depende de {value}.", "¿De qué depende {subject}?", ["datos de prueba", "licencia de fuentes", "acceso al laboratorio", "revisión legal"]),
    topic("meeting_cadence", "routine", "{subject} celebra seguimiento {value}.", "¿Cuándo celebra seguimiento {subject}?", ["los martes", "los jueves", "cada dos semanas", "al final del mes"]),
    topic("scope_exclusion", "constraint", "{subject} excluye {value} de la primera fase.", "¿Qué excluye {subject} de la primera fase?", ["aplicación móvil", "despliegue público", "sincronización externa", "pagos"], "Primera fase"),
    topic("storage", "decision", "{subject} guardará los artefactos en {value}.", "¿Dónde guardará artefactos {subject}?", ["carpeta local", "almacén interno", "disco compartido", "repositorio privado"]),
    topic("performance_goal", "goal", "{subject} busca una respuesta inferior a {value}.", "¿Qué tiempo de respuesta busca {subject}?", ["200 ms", "500 ms", "1 s", "2 s"]),
    topic("privacy_rule", "constraint", "{subject} permite en pruebas solo {value}.", "¿Qué datos permite {subject} en pruebas?", ["datos sintéticos", "datos anonimizados", "datos públicos", "datos de laboratorio"], "En pruebas"),
    topic("reviewer", "relationship", "La revisión de {subject} corresponde a {value}.", "¿Quién revisa {subject}?", ["Mara", "Saúl", "Olga", "Eli"]),
    topic("documentation", "procedure", "{subject} documenta decisiones en {value}.", "¿Dónde documenta decisiones {subject}?", ["README", "registro de decisiones", "bitácora", "wiki interna"]),
    topic("milestone", "event", "El siguiente hito de {subject} es {value}.", "¿Cuál es el siguiente hito de {subject}?", ["ensayo cerrado", "revisión técnica", "demo interna", "entrega piloto"]),
    topic("interface", "decision", "La primera interfaz de {subject} será {value}.", "¿Cuál será la primera interfaz de {subject}?", ["CLI", "API local", "panel web", "aplicación de escritorio"]),
    topic("language", "constraint", "La documentación de {subject} se redacta en {value}.", "¿En qué idioma se documenta {subject}?", ["español", "inglés", "portugués", "francés"]),
    topic("support_contact", "relationship", "El contacto de soporte de {subject} es {value}.", "¿Quién es el contacto de soporte de {subject}?", ["Ada", "Ian", "Uma", "Noé"]),
    topic("source_of_truth", "decision", "La fuente de verdad de {subject} es {value}.", "¿Cuál es la fuente de verdad de {subject}?", ["base local", "documento firmado", "registro de eventos", "repositorio"]),
    topic("rollback", "procedure", "{subject} revierte cambios con {value}.", "¿Cómo revierte cambios {subject}?", ["respaldo previo", "versión anterior", "migración inversa", "restauración de snapshot"]),
    topic("quality_gate", "constraint", "{subject} exige {value} de cobertura mínima.", "¿Qué cobertura mínima exige {subject}?", ["80 %", "85 %", "90 %", "95 %"]),
    topic("handoff", "procedure", "El traspaso de {subject} se hace mediante {value}.", "¿Cómo se hace el traspaso de {subject}?", ["reunión", "documento", "ticket", "video breve"]),
    topic("archive", "procedure", "{subject} archiva los resultados en {value}.", "¿Dónde archiva resultados {subject}?", ["carpeta de cierre", "repositorio", "base documental", "almacén cifrado"]),
]

CONVERSATION = [
    topic("current_task", "fact", "En {subject}, la tarea actual es {value}.", "¿Cuál es la tarea actual de {subject}?", ["revisar el borrador", "comparar dos opciones", "preparar una lista", "corregir un ejemplo"]),
    topic("next_step", "decision", "En {subject}, se acordó {value} como siguiente paso.", "¿Qué siguiente paso se acordó en {subject}?", ["leer el informe", "validar las cifras", "escribir un resumen", "hacer una llamada"]),
    topic("temporary_constraint", "constraint", "En {subject}, se pidió {value} durante esta conversación.", "¿Qué límite temporal se pidió en {subject}?", ["no publicar", "no usar datos reales", "no cambiar el diseño", "no enviar correos"], "Durante esta conversación"),
    topic("open_question", "fact", "La pregunta pendiente en {subject} es {value}.", "¿Qué pregunta quedó pendiente en {subject}?", ["quién revisará", "cuándo entregar", "qué formato usar", "qué opción elegir"]),
    topic("reference_file", "fact", "El archivo ficticio mencionado en {subject} es {value}.", "¿Qué archivo se mencionó en {subject}?", ["resumen-a.md", "plan-b.md", "datos-c.csv", "notas-d.txt"]),
    topic("chosen_option", "decision", "En {subject}, se eligió {value}.", "¿Qué opción se eligió en {subject}?", ["opción A", "opción B", "opción C", "opción D"]),
    topic("excluded_option", "decision", "En {subject}, se descartó {value}.", "¿Qué opción se descartó en {subject}?", ["opción W", "opción X", "opción Y", "opción Z"]),
    topic("review_rule", "procedure", "En {subject}, el resultado debe revisarse con {value}.", "¿Cómo debe revisarse el resultado de {subject}?", ["una lista de control", "una segunda lectura", "una muestra", "una comparación"]),
    topic("response_style", "preference", "En {subject}, se pidió responder con {value}.", "¿Qué estilo de respuesta se pidió en {subject}?", ["una tabla", "dos párrafos", "pasos numerados", "un ejemplo"]),
    topic("handoff_note", "fact", "La nota de traspaso de {subject} indica {value}.", "¿Qué indica la nota de traspaso de {subject}?", ["esperar un archivo", "revisar una cifra", "continuar mañana", "pedir un dato"]),
    topic("agreed_owner", "relationship", "En {subject}, {value} asumió la siguiente revisión.", "¿Quién asumió la revisión en {subject}?", ["Alicia", "Berto", "Celia", "Damián"]),
    topic("agreed_date", "event", "En {subject}, se acordó revisar el resultado el {value}.", "¿Qué fecha de revisión se acordó en {subject}?", ["2027-01-05", "2027-02-12", "2027-03-19", "2027-04-26"]),
    topic("correction", "correction", "En {subject}, se corrigió el dato a {value}.", "¿A qué valor se corrigió el dato en {subject}?", ["18 unidades", "26 unidades", "34 unidades", "42 unidades"]),
    topic("decision_reason", "decision", "En {subject}, la decisión se justificó por {value}.", "¿Por qué se tomó la decisión en {subject}?", ["menor complejidad", "mejor accesibilidad", "menos dependencias", "mayor claridad"]),
    topic("changed_assumption", "observation", "En {subject}, dejó de asumirse {value}.", "¿Qué supuesto se abandonó en {subject}?", ["que el archivo era final", "que había acceso de red", "que existía una copia", "que el dato era reciente"]),
    topic("blocked_by", "fact", "En {subject}, el avance quedó bloqueado por {value}.", "¿Qué bloqueó el avance en {subject}?", ["una confirmación", "un archivo ausente", "una prueba fallida", "un permiso pendiente"]),
    topic("completed_step", "event", "En {subject}, ya se completó {value}.", "¿Qué paso se completó en {subject}?", ["la lectura inicial", "la comparación", "la validación local", "la entrega del borrador"]),
    topic("deferred_step", "decision", "En {subject}, se aplazó {value}.", "¿Qué se aplazó en {subject}?", ["la publicación", "la prueba extensa", "la migración", "la aprobación final"]),
    topic("requested_example", "preference", "En {subject}, se pidió un ejemplo de {value}.", "¿De qué se pidió un ejemplo en {subject}?", ["un caso positivo", "un caso negativo", "una excepción", "un conflicto"]),
    topic("audience", "fact", "En {subject}, el resultado se prepara para {value}.", "¿Para quién se prepara el resultado en {subject}?", ["el equipo", "la dirección", "el cliente", "la comunidad"]),
    topic("attachment", "fact", "En {subject}, se adjuntó el archivo ficticio {value}.", "¿Qué archivo se adjuntó en {subject}?", ["muestra-uno.json", "esquema-dos.sql", "captura-tres.png", "guia-cuatro.md"]),
    topic("source_pointer", "fact", "En {subject}, la fuente señalada fue {value}.", "¿Qué fuente se señaló en {subject}?", ["sección 2 del informe", "fila 8 de la tabla", "página 4 del manual", "nota 3 del acta"]),
    topic("tool_result", "observation", "En {subject}, la herramienta devolvió {value}.", "¿Qué devolvió la herramienta en {subject}?", ["12 coincidencias", "cero errores", "un archivo", "una advertencia"]),
    topic("unverified_claim", "observation", "En {subject}, quedó sin verificar {value}.", "¿Qué quedó sin verificar en {subject}?", ["el coste", "la fecha", "la autoría", "la cobertura"]),
    topic("scope_boundary", "constraint", "En {subject}, el trabajo se limitó a {value}.", "¿A qué se limitó el trabajo en {subject}?", ["los ejemplos", "la documentación", "la revisión local", "una sola consulta"], "Durante esta conversación"),
    topic("priority", "decision", "En {subject}, se priorizó {value}.", "¿Qué se priorizó en {subject}?", ["corregir el error", "medir latencia", "revisar datos", "entender el flujo"]),
    topic("answer_language", "preference", "En {subject}, se pidió explicar el resultado en {value}.", "¿En qué idioma se pidió explicar el resultado en {subject}?", ["español", "inglés", "portugués", "francés"]),
    topic("disagreement", "observation", "En {subject}, hubo desacuerdo sobre {value}.", "¿Sobre qué hubo desacuerdo en {subject}?", ["el orden de pasos", "el tamaño de muestra", "la interpretación", "la fecha de corte"]),
    topic("feedback", "observation", "En {subject}, el comentario recibido fue {value}.", "¿Qué comentario se recibió en {subject}?", ["aclarar el ejemplo", "simplificar la tabla", "medir otra fase", "añadir contexto"]),
    topic("closing_state", "fact", "En {subject}, se cerró con {value} pendiente.", "¿Qué quedó pendiente al cerrar {subject}?", ["una revisión", "una respuesta", "un enlace", "una prueba"]),
]

AGENT = [
    topic("output_language", "preference", "El perfil ficticio de {subject} registra {value} como idioma de salida.", "¿Qué idioma de salida registra el perfil de {subject}?", ["español", "inglés", "portugués", "francés"]),
    topic("answer_length", "preference", "El perfil ficticio de {subject} registra respuestas {value}.", "¿Qué longitud de respuesta registra el perfil de {subject}?", ["breves", "detalladas", "de un párrafo", "con ejemplos"]),
    topic("evidence", "procedure", "El flujo documentado de {subject} contrasta afirmaciones con {value}.", "¿Con qué contrasta afirmaciones el flujo de {subject}?", ["documentos fuente", "resultados de prueba", "registros", "citas"]),
    topic("failure_handling", "procedure", "El flujo documentado de {subject} trata fallos mediante {value}.", "¿Cómo trata fallos el flujo de {subject}?", ["registro del error", "parada de la secuencia", "solicitud de datos", "un reintento"]),
    topic("tool_order", "procedure", "La secuencia de investigación registrada para {subject} empieza por {value}.", "¿Por dónde empieza la secuencia registrada de {subject}?", ["la fuente local", "el registro", "la documentación", "una prueba breve"]),
    topic("privacy", "constraint", "El perfil ficticio de {subject} marca {value} como dato excluido de informes.", "¿Qué dato marca como excluido el perfil de {subject}?", ["claves", "datos de contacto", "rutas privadas", "identificadores personales"], "En informes"),
    topic("uncertainty", "procedure", "El formato de informe registrado para {subject} señala {value} cuando falta evidencia.", "¿Qué señala el informe registrado de {subject} si falta evidencia?", ["incertidumbre", "la fuente faltante", "el supuesto", "el límite de la prueba"]),
    topic("citation", "preference", "El perfil ficticio de {subject} coloca referencias {value}.", "¿Dónde coloca referencias el perfil de {subject}?", ["junto a cada afirmación", "al final del párrafo", "en notas", "en una tabla"]),
    topic("schedule", "constraint", "El perfil ficticio de {subject} registra planificación {value}.", "¿Qué condición de planificación registra el perfil de {subject}?", ["con solicitud expresa", "entre semana", "tras aprobación", "al final de la sesión"], "Al planificar tareas"),
    topic("handoff", "procedure", "El procedimiento ficticio de {subject} incluye {value} en el traspaso.", "¿Qué incluye el traspaso de {subject}?", ["estado y pendientes", "un resumen breve", "enlaces a archivos", "riesgos abiertos"]),
    topic("active_module", "fact", "El módulo activo registrado para {subject} es {value}.", "¿Qué módulo activo se registró para {subject}?", ["búsqueda", "indexación", "resumen", "validación"]),
    topic("tool_available", "fact", "El entorno ficticio de {subject} ofrece la herramienta {value}.", "¿Qué herramienta ofrece el entorno de {subject}?", ["lector local", "comparador", "validador", "inspector"]),
    topic("tool_unavailable", "constraint", "En el entorno ficticio de {subject}, no está disponible {value}.", "¿Qué no está disponible en el entorno de {subject}?", ["conexión externa", "editor visual", "compilador", "navegador"]),
    topic("last_success", "event", "La última operación exitosa registrada para {subject} fue {value}.", "¿Cuál fue la última operación exitosa de {subject}?", ["leer un archivo", "validar JSON", "crear un índice", "comparar resultados"]),
    topic("last_failure", "event", "El último fallo registrado para {subject} fue {value}.", "¿Cuál fue el último fallo registrado de {subject}?", ["archivo ausente", "respuesta incompleta", "consulta inválida", "tiempo agotado"]),
    topic("checkpoint", "fact", "El último punto de control ficticio de {subject} es {value}.", "¿Cuál es el punto de control de {subject}?", ["fase uno", "fase dos", "fase tres", "fase cuatro"]),
    topic("pending_operation", "fact", "La operación pendiente registrada para {subject} es {value}.", "¿Qué operación tiene pendiente {subject}?", ["revisar un archivo", "comparar salidas", "generar un resumen", "cerrar un informe"]),
    topic("artifact", "fact", "El artefacto ficticio producido por {subject} se llama {value}.", "¿Qué artefacto produjo {subject}?", ["informe-a.md", "tabla-b.csv", "índice-c.json", "nota-d.txt"]),
    topic("source_inspected", "event", "La fuente ficticia inspeccionada por {subject} fue {value}.", "¿Qué fuente inspeccionó {subject}?", ["manual interno", "archivo de datos", "registro de prueba", "documento de diseño"]),
    topic("error_code", "observation", "El código de error observado por {subject} fue {value}.", "¿Qué código de error observó {subject}?", ["E101", "E202", "E303", "E404"]),
    topic("retry_count", "fact", "El registro ficticio de {subject} indica {value} reintentos.", "¿Cuántos reintentos indica el registro de {subject}?", ["cero", "uno", "dos", "tres"]),
    topic("workaround", "observation", "La solución provisional registrada por {subject} fue {value}.", "¿Qué solución provisional registró {subject}?", ["usar un archivo local", "reducir el lote", "validar por partes", "repetir la lectura"]),
    topic("environment", "fact", "El entorno ficticio registrado para {subject} es {value}.", "¿Qué entorno se registró para {subject}?", ["Mac local", "contenedor", "máquina de prueba", "sesión aislada"]),
    topic("latency", "observation", "La última latencia ficticia medida por {subject} fue {value}.", "¿Qué latencia midió {subject}?", ["120 ms", "250 ms", "500 ms", "900 ms"]),
    topic("cache_state", "fact", "La caché ficticia observada por {subject} estaba {value}.", "¿En qué estado estaba la caché observada por {subject}?", ["vacía", "parcial", "completa", "caducada"]),
    topic("evaluation_set", "fact", "El conjunto ficticio de evaluación de {subject} se llamó {value}.", "¿Cómo se llamó el conjunto de evaluación de {subject}?", ["lote alfa", "lote beta", "lote gamma", "lote delta"]),
    topic("known_limit", "constraint", "El límite conocido registrado por {subject} fue {value}.", "¿Qué límite conocido registró {subject}?", ["sin red", "sin índices", "sin permisos", "sin datos finales"]),
    topic("next_check", "goal", "La siguiente comprobación registrada para {subject} es {value}.", "¿Cuál es la siguiente comprobación de {subject}?", ["abrir un archivo", "contar los casos", "medir latencia", "revisar etiquetas"]),
    topic("handoff_target", "relationship", "El traspaso ficticio de {subject} quedó asignado a {value}.", "¿A quién se asignó el traspaso de {subject}?", ["agente A", "agente B", "equipo C", "revisor D"]),
    topic("completion_state", "fact", "El estado de cierre registrado para {subject} fue {value}.", "¿Con qué estado cerró {subject}?", ["completo", "incompleto", "bloqueado", "en revisión"]),
]


GROUPS = [
    ("user", 40, PERSONAL),
    ("project", 20, PROJECT),
    ("conversation", 20, CONVERSATION),
    ("agent", 20, AGENT),
]


def identifier(value):
    return "gmv1_" + hashlib.sha256(value.encode("utf-8")).hexdigest()[:20]


def generate():
    memories = []
    cases = []
    for scope_kind, count, topics in GROUPS:
        for number in range(1, count + 1):
            scope_id = f"synthetic_{scope_kind}_{number:03d}"
            subject = f"{scope_kind} ficticio {number:03d}"
            for index, (key, memory_type, statement, question, values, condition) in enumerate(topics):
                value = values[(number + index) % len(values)]
                memory_id = identifier(f"{scope_kind}/{scope_id}/{key}")
                summary = statement.format(subject=subject, value=value)
                memories.append({
                    "id": memory_id,
                    "scope_kind": scope_kind,
                    "scope_id": scope_id,
                    "entity_id": None,
                    "revision": 1,
                    "memory_type": memory_type,
                    "original_text": f"Ejemplo completamente sintético. {summary}",
                    "summary": summary,
                    "conditions": condition,
                    "source": {
                        "kind": "user" if scope_kind in ("user", "conversation") else "document" if scope_kind == "project" else "tool",
                        "ref": f"synthetic://general-memory-v1/{scope_kind}/{number:03d}/{key}",
                    },
                    "state": "active",
                    "valid_from": None,
                    "valid_until": None,
                })
                cases.append({
                    "case_id": identifier(f"positive/{scope_kind}/{scope_id}/{key}"),
                    "query": question.format(subject=subject),
                    "scope_kind": scope_kind,
                    "scope_id": scope_id,
                    "entity_id": None,
                    "as_of": AS_OF,
                    "limit": 20,
                    "expected_memory_ids": [memory_id],
                    "label": "positive",
                    "topic": key,
                })
            for n, (question, topic_key) in enumerate([
                (f"¿Cuál es el código secreto de {subject}?", "unknown_secret"),
                (f"¿Cuál es la dirección postal exacta de {subject}?", "unknown_address"),
                (f"¿Qué contraseña usa {subject}?", "unknown_password"),
                (f"¿Qué decisión tomó {subject} mañana?", "unknown_future"),
                (f"¿Cuál es el color favorito de {subject}?", "unknown_color"),
            ]):
                cases.append({
                    "case_id": identifier(f"negative/{scope_kind}/{scope_id}/{n}"),
                    "query": question,
                    "scope_kind": scope_kind,
                    "scope_id": scope_id,
                    "entity_id": None,
                    "as_of": AS_OF,
                    "limit": 20,
                    "expected_memory_ids": [],
                    "label": "negative",
                    "topic": topic_key,
                })
    assert len(memories) == 2800 and len(cases) == 3300
    assert len({memory["id"] for memory in memories}) == len(memories)
    assert len({case["case_id"] for case in cases}) == len(cases)
    memory_ids = {memory["id"] for memory in memories}
    assert all(set(case["expected_memory_ids"]) <= memory_ids for case in cases)
    by_scope_topic = {(case["scope_kind"], case["scope_id"], case["topic"]): case for case in cases if case["label"] == "positive"}
    unscoped = []
    for number in range(1, 41):
        scope_id = f"synthetic_user_{number:03d}"
        for key, query in [("legal_name", "Hola, ¿cómo me llamo?"), ("language", "¿En qué idioma prefiero conversar?")]:
            expected = by_scope_topic[("user", scope_id, key)]["expected_memory_ids"]
            unscoped.append({"case_id": identifier(f"unscoped/user/{number}/{key}"), "query": query, "host_context": {"actor_scope_id": scope_id}, "expected_scope_kind": "user", "expected_scope_id": scope_id, "expected_memory_ids": expected})
    for scope_kind, count, key, query, context_key in [
        ("project", 20, "objective", "¿Cuál es el objetivo de este proyecto?", "active_project_id"),
        ("conversation", 20, "next_step", "¿Qué acordamos hacer después?", "active_conversation_id"),
        ("agent", 20, "output_language", "¿En qué idioma debes responder?", "active_agent_id"),
    ]:
        for number in range(1, count + 1):
            scope_id = f"synthetic_{scope_kind}_{number:03d}"
            expected = by_scope_topic[(scope_kind, scope_id, key)]["expected_memory_ids"]
            unscoped.append({"case_id": identifier(f"unscoped/{scope_kind}/{number}/{key}"), "query": query, "host_context": {context_key: scope_id}, "expected_scope_kind": scope_kind, "expected_scope_id": scope_id, "expected_memory_ids": expected})
    assert len(unscoped) == 140
    (ROOT / "memories.json").write_text(json.dumps(memories, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (ROOT / "queries.jsonl").write_text("".join(json.dumps(case, ensure_ascii=False) + "\n" for case in cases), encoding="utf-8")
    (ROOT / "unscoped-queries.jsonl").write_text("".join(json.dumps(case, ensure_ascii=False) + "\n" for case in unscoped), encoding="utf-8")
    manifest = {
        "version": "general-memory-v1",
        "synthetic": True,
        "as_of": AS_OF,
        "memories": len(memories),
        "queries": len(cases),
        "positive_queries": sum(case["label"] == "positive" for case in cases),
        "negative_queries": sum(case["label"] == "negative" for case in cases),
        "unscoped_queries": len(unscoped),
        "memories_by_scope": dict(sorted(Counter(memory["scope_kind"] for memory in memories).items())),
        "memories_by_type": dict(sorted(Counter(memory["memory_type"] for memory in memories).items())),
    }
    (ROOT / "manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    generate()
