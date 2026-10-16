"""Render editorial benchmark figures from frozen synthetic evaluation JSON."""

import json
import statistics
from decimal import Decimal, ROUND_HALF_UP
from pathlib import Path
from xml.sax.saxutils import escape

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "docs/evaluation/assets"
SPEED_OUTPUT = ASSETS / "speed-comparison-v1.svg"
ACCURACY_OUTPUT = ASSETS / "accuracy-comparison-v1.svg"
ENGRAM_MISSED_CASE = "gmv1_c31971ae7655de9d9d89"
INK, MUTED, GRID = "#20252b", "#657078", "#d9dedb"
TEAL, GOLD, PAPER = "#187d7b", "#b77b26", "#fffefa"


def t(x, y, value, size=17, weight=400, fill=INK, anchor="start"):
    return (f'<text x="{x}" y="{y}" text-anchor="{anchor}" '
            f'font-family="Arial,Helvetica,sans-serif" font-size="{size}" '
            f'font-weight="{weight}" fill="{fill}">{escape(str(value))}</text>')


def line(x1, y1, x2, y2, color=GRID, width=1):
    return (f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" '
            f'stroke="{color}" stroke-width="{width}"/>')


def number(value):
    return str(Decimal(str(value)).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)).replace(".", ",")


def shell(title, deck, description, figure_number):
    return [
        '<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="700" viewBox="0 0 1200 700" role="img" aria-labelledby="title desc">',
        f'<title id="title">{escape(title)}</title>',
        f'<desc id="desc">{escape(description)}</desc>',
        f'<rect width="1200" height="700" fill="{PAPER}"/>',
        line(64, 18, 1136, 18, INK, 2),
        t(64, 49, f"iROMEM  /  FIGURA {figure_number}", 13, 700, TEAL),
        t(64, 95, title, 35, 700),
        t(64, 133, deck, 19, 400, MUTED),
        line(64, 177, 1136, 177),
        line(601, 202, 601, 645),
    ]


def axis(x, end_x, top, baseline, minimum, maximum, ticks):
    result = []
    for tick in ticks:
        y = round(baseline - (tick - minimum) * (baseline - top) / (maximum - minimum), 2)
        result.append(line(x, y, end_x, y, INK if tick == minimum else GRID, 1.4 if tick == minimum else 1))
        result.append(t(x - 14, y + 5, tick, 14, 400, MUTED, "end"))
    result.append(line(x, top, x, baseline, INK, 1.4))
    return result


def column(center, baseline, top, value, maximum, color, readout, label, sublabel=""):
    height = round((baseline - top) * value / maximum, 2)
    result = [
        f'<rect x="{center - 51}" y="{baseline - height}" width="102" height="{height}" fill="{color}" opacity="0.83"/>',
        t(center, baseline - height - 14, readout, 19, 700, color, "middle"),
        t(center, baseline + 32, label, 17, 700, INK, "middle"),
    ]
    if sublabel:
        result.append(t(center, baseline + 56, sublabel, 15, 400, MUTED, "middle"))
    return result


def point(center, baseline, top, value, minimum, maximum, color, readout, label, sublabel=""):
    y = round(baseline - (value - minimum) * (baseline - top) / (maximum - minimum), 2)
    result = [
        line(center, baseline, center, y, color, 3),
        f'<circle cx="{center}" cy="{y}" r="10" fill="{color}"/>',
        t(center, y - 20, readout, 19, 700, color, "middle"),
        t(center, baseline + 32, label, 17, 700, INK, "middle"),
    ]
    if sublabel:
        result.append(t(center, baseline + 56, sublabel, 15, 400, MUTED, "middle"))
    return result


def render_speed(local, engram_local, engram_turns, seeded_turns):
    keyword_ms = local["after_keywords"]["p50_ms"]
    engram_ms = engram_local["p50_ms"]
    seed_s = statistics.median(row["final_answer_ms"] for row in seeded_turns) / 1000
    engram_s = statistics.median(row["final_answer_ms"] for row in engram_turns) / 1000
    parts = shell(
        "¿Cuánto tarda en llegar la respuesta?",
        "Dos pasos medidos por separado, con sus propias unidades y escalas.",
        "Búsqueda local mediana: iROMEM 5,23 milisegundos, Engram 5,52. Turno de Codex mediano: semilla inicial simulada 5,12 segundos, Engram MCP 9,49. Los ejes empiezan en cero. La preparación de la semilla no está incluida en el turno de Codex.",
        "01",
    )
    parts += [
        t(80, 221, "01 / Encontrar candidatos", 22, 700),
        t(80, 250, "2.800 recuerdos · 540 consultas · p50", 16, 400, MUTED),
        t(128, 295, "ms", 14, 700, MUTED),
        *axis(128, 544, 315, 515, 0, 6, [0, 2, 4, 6]),
        *column(270, 515, 315, keyword_ms, 6, TEAL, f"{number(keyword_ms)} ms", "iROMEM", "palabras clave"),
        *column(435, 515, 315, engram_ms, 6, GOLD, f"{number(engram_ms)} ms", "Engram", "palabras clave"),
        t(665, 221, "02 / Responder con Codex", 22, 700),
        t(665, 250, "6 preguntas sintéticas · p50", 16, 400, MUTED),
        t(710, 295, "s", 14, 700, MUTED),
        *axis(710, 1125, 315, 515, 0, 10, [0, 2, 4, 6, 8, 10]),
        *column(852, 515, 315, seed_s, 10, TEAL, f"{number(seed_s)} s", "Semilla inicial", "simulada"),
        *column(1017, 515, 315, engram_s, 10, GOLD, f"{number(engram_s)} s", "Engram MCP"),
        line(80, 604, 1140, 604),
        t(80, 633, "LECTURA", 13, 700, TEAL),
        t(171, 633, "La diferencia local es pequeña; el turno con semilla termina antes en esta muestra.", 17),
        t(80, 668, "La semilla tardó 0,73 s p50 en prepararse aparte. Inyectarla automáticamente en Codex sigue pendiente.", 15, 400, MUTED),
        '</svg>',
    ]
    SPEED_OUTPUT.write_text("\n".join(parts) + "\n")
    print(SPEED_OUTPUT)


def render_accuracy(local, engram_local, engram_turns, seeded_turns):
    positives = local["after_keywords"]["positives"]
    iromem_hits = local["after_keywords"]["top20"]
    engram_hits = engram_local["recall_at_20"]
    engram_by_id = {row["case_id"]: row for row in engram_turns}
    seeded_by_id = {row["case_id"]: row for row in seeded_turns}
    if positives != 460 or len(engram_by_id) != 6 or engram_by_id.keys() != seeded_by_id.keys():
        raise ValueError("Unexpected benchmark cohort")
    if sum(row["positive"] for row in engram_turns) != 5:
        raise ValueError("Unexpected positive/negative split")
    # Reviewed saved answers against the synthetic expected facts.
    missed = engram_by_id[ENGRAM_MISSED_CASE]
    if not missed["positive"] or missed["final_answer"] != "No encuentro ese recuerdo":
        raise ValueError("The reviewed Engram miss changed")
    if any("No encuentro" in seeded_by_id[row["case_id"]]["final_answer"]
           for row in engram_turns if row["positive"]):
        raise ValueError("A reviewed seeded positive changed")
    parts = shell(
        "¿Cuántas preguntas acierta cada ruta?",
        "Dos pruebas distintas: encontrar el recuerdo y responder con él.",
        "La recuperación de candidatos usa un eje ampliado de 450 a 460 aciertos: iROMEM 452 y Engram 454 sobre 460 positivas. La respuesta final usa un eje desde cero: Codex con semilla inicial simulada acierta seis de seis y Codex con Engram cinco de seis. No se equiparan las dos pruebas.",
        "02",
    )
    parts += [
        t(80, 221, "01 / Recuerdo entre 20 candidatos", 22, 700),
        t(80, 250, "460 consultas positivas · detalle ampliado", 16, 400, MUTED),
        t(128, 295, "aciertos", 14, 700, MUTED),
        *axis(128, 544, 315, 515, 450, 460, [450, 452, 454, 456, 458, 460]),
        *point(270, 515, 315, iromem_hits, 450, 460, TEAL, f"{iromem_hits}/{positives}", "iROMEM", "palabras clave"),
        *point(435, 515, 315, engram_hits, 450, 460, GOLD, f"{engram_hits}/{positives}", "Engram", "palabras clave"),
        t(665, 221, "02 / Respuesta correcta de Codex", 22, 700),
        t(665, 250, "6 preguntas · 5 positivas y 1 negativa", 16, 400, MUTED),
        t(710, 295, "aciertos", 14, 700, MUTED),
        *axis(710, 1125, 315, 515, 0, 6, [0, 1, 2, 3, 4, 5, 6]),
        *point(852, 515, 315, 6, 0, 6, TEAL, "6/6", "Semilla inicial", "simulada"),
        *point(1017, 515, 315, 5, 0, 6, GOLD, "5/6", "Engram MCP"),
        line(80, 604, 1140, 604),
        t(80, 633, "LECTURA", 13, 700, TEAL),
        t(171, 633, "Engram encuentra 2 recuerdos más; la semilla logra 1 respuesta más en seis casos.", 17),
        t(80, 668, "El eje izquierdo empieza en 450 para revelar 2 casos de diferencia; el derecho empieza en 0. Muestras distintas.", 15, 400, MUTED),
        '</svg>',
    ]
    ACCURACY_OUTPUT.write_text("\n".join(parts) + "\n")
    print(ACCURACY_OUTPUT)


def main():
    local = json.loads((ROOT / "outputs/iromem/evaluation/keyword-preflight-v1/final.json").read_text())["summary"]
    engram_local = json.loads((ROOT / "outputs/iromem/evaluation/engram-comparison-v1/final-keywords.json").read_text())["summary"]["any"]
    engram_turns = json.loads((ROOT / "outputs/iromem/evaluation/engram-codex-end-to-end-v1/codex-engram.json").read_text())["rows"]
    seeded_turns = json.loads((ROOT / "outputs/iromem/evaluation/codex-initial-seed-simulation-v1/summary.json").read_text())["rows"]
    ASSETS.mkdir(parents=True, exist_ok=True)
    render_speed(local, engram_local, engram_turns, seeded_turns)
    render_accuracy(local, engram_local, engram_turns, seeded_turns)


if __name__ == "__main__":
    main()
