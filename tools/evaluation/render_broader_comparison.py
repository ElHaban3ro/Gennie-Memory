"""Render expanded comparisons in the existing README's Distill-inspired SVG style.

The figures keep the established cream ground, Georgia headings, restrained palette,
zero-based axes, two-panel layouts, direct labels, and explanatory footnotes.
"""

import json
import statistics
from pathlib import Path
from xml.sax.saxutils import escape

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "outputs/iromem/evaluation/broader-comparison-2026-10-16"
ORIGINAL = ROOT / "outputs/iromem/evaluation/full-comparison-2026-10-16/aggregate.json"
ASSETS = ROOT / "docs/evaluation/assets"

INK = "#25313b"
MUTED = "#617078"
GRID = "#d8dfdf"
COLORS = ["#18817e", "#b77a27", "#617db1", "#8a648f"]


def external(name):
    runs = [json.loads((DATA / f"search-{name}-{index}-summary.json").read_text())["overall"]
            for index in range(1, 5)]
    return {
        "top1": round(runs[0]["top_1_accuracy"] * 460),
        "top20": round(runs[0]["recall_at_20"] * 460),
        "p50": statistics.median(run["latency_ms"]["p50"] for run in runs),
        "p95": statistics.median(run["latency_ms"]["p95"] for run in runs),
        "p95_runs": [run["latency_ms"]["p95"] for run in runs],
    }


original = json.loads(ORIGINAL.read_text())
search = original["search"]
engines = [
    ("iROMEM", {"top1": 420, "top20": 457,
                "p50": search["iromem"]["median_run_p50_ms"],
                "p95": search["iromem"]["median_run_p95_ms"],
                "p95_runs": [run["p95_ms"] for run in search["iromem"]["runs"]]}),
    ("Engram", {"top1": 427, "top20": 456,
               "p50": search["engram"]["median_run_p50_ms"],
               "p95": search["engram"]["median_run_p95_ms"],
               "p95_runs": [run["p95_ms"] for run in search["engram"]["runs"]]}),
    ("Lodis", external("lodis-semantic")),
    ("Basic Memory", external("basic-memory")),
]


class Svg:
    def __init__(self, title, subtitle, height=650):
        self.height = height
        self.parts = [
            f'<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="{height}" '
            f'viewBox="0 0 1200 {height}" role="img" aria-label="{escape(title)}">',
            '<rect width="100%" height="100%" fill="#fffefa"/>',
        ]
        self.text(58, 75, title, 34, INK, "Georgia,serif")
        self.text(58, 110, subtitle, 17, MUTED)

    def text(self, x, y, value, size=16, color=INK, font="Arial,sans-serif", anchor="start"):
        self.parts.append(
            f'<text x="{x}" y="{y}" fill="{color}" text-anchor="{anchor}" '
            f'font-family="{font}" font-size="{size}">{escape(str(value))}</text>'
        )

    def line(self, x1, y1, x2, y2, color=GRID):
        self.parts.append(f'<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{color}" stroke-width="1"/>')

    def rect(self, x, y, width, height, color):
        self.parts.append(f'<rect x="{x}" y="{y}" width="{width}" height="{height}" fill="{color}"/>')

    def save(self, filename):
        (ASSETS / filename).write_text("\n".join([*self.parts, "</svg>"]) + "\n")


def label(svg, x, y, name):
    if name == "Basic Memory":
        svg.text(x, y, "Basic", 15, anchor="middle")
        svg.text(x, y + 18, "Memory", 15, anchor="middle")
    else:
        svg.text(x, y, name, 15, anchor="middle")


def panel(svg, x, title, subtitle, unit, maximum, values, decimals=2, top=215, base=497):
    """A zero-origin, four-bar panel matching the prior Rust SVG component."""
    left, right = x + 52, x + 470
    svg.text(x, 144, title, 22, INK, "Georgia,serif")
    svg.text(x, 174, subtitle, 16, MUTED)
    svg.text(right, 199, unit, 14, MUTED, anchor="end")
    for fraction in (0, 0.5, 1):
        y = round(base - (base - top) * fraction)
        svg.line(left, y, right, y)
        tick = maximum * fraction
        svg.text(left - 10, y + 5, f"{tick:.0f}" if decimals == 0 else f"{tick:.1f}", 14, MUTED, anchor="end")
    svg.line(left, top, left, base, INK)
    for index, (name, value, color) in enumerate(values):
        center = x + (120 + index * 105 if len(values) == 4 else 140 + index * 135)
        height = round((base - top) * value / maximum)
        svg.rect(center - 32, base - height, 64, height, color)
        svg.text(center, base - height - 12, f"{value:.{decimals}f}", 16, anchor="middle")
        label(svg, center, base + 29, name)


def entries(key):
    return [(name, values[key], COLORS[index]) for index, (name, values) in enumerate(engines)]


svg = Svg("Recuerdos recuperados en las mismas preguntas",
          "Cohorte 014–017 · 460 positivas · 2.800 recuerdos sintéticos · barras desde cero")
panel(svg, 58, "Entre 20 candidatos", "Recuerdo esperado presente", "aciertos / 460", 460, entries("top20"), 0)
panel(svg, 610, "Primer candidato", "Recuerdo esperado en posición 1", "aciertos / 460", 460, entries("top1"), 0)
svg.text(58, 578, "Búsqueda de candidatos; una lista de resultados no equivale a una semilla seleccionada.", 16, MUTED)
svg.text(58, 608, "iROMEM + Decisions obtuvo 441/460 semillas correctas y dejó 80/80 negativos sin semilla.", 16, MUTED)
svg.save("broader-comparison-accuracy-2026-10-16.svg")

svg = Svg("Tiempo de búsqueda de candidatos",
          "Cohorte 014–017 · 540 preguntas idénticas · mediana de cuatro corridas por motor")
panel(svg, 58, "Búsqueda: mediana", "Mediana de cuatro p50", "milisegundos", 30, entries("p50"))
panel(svg, 610, "Búsqueda: p95", "Mediana de cuatro p95", "milisegundos", 140, entries("p95"))
svg.text(58, 578, "iROMEM, Engram y Lodis: procesos persistentes; Basic Memory: API local en proceso.", 16, MUTED)
svg.text(58, 608, "Las escalas de ambos paneles empiezan en cero; la diferencia de transporte limita la comparación.", 16, MUTED)
svg.save("broader-comparison-latency-2026-10-16.svg")

# Four small multiples preserve a readable zero-origin scale for each engine.
svg = Svg("Búsqueda de candidatos: p95 por corrida",
          "540 preguntas sintéticas idénticas por corrida · cuatro motores · milisegundos", 730)
for index, (name, values) in enumerate(engines):
    col, row = index % 2, index // 2
    x, top = 58 + col * 552, 150 + row * 260
    base, left, right = top + 175, x + 55, x + 492
    maximum = max(15, 20 if name in ("iROMEM", "Engram") else
                  (200 if name == "Lodis" else 500))
    svg.text(x, top - 8, name, 22, INK, "Georgia,serif")
    svg.text(x + 490, top - 8, f"escala 0–{maximum} ms", 14, MUTED, anchor="end")
    for fraction in (0, 0.5, 1):
        y = round(base - (base - top) * fraction)
        svg.line(left, y, right, y)
        svg.text(left - 10, y + 5, f"{maximum * fraction:.0f}", 13, MUTED, anchor="end")
    svg.line(left, top, left, base, INK)
    for run, value in enumerate(values["p95_runs"]):
        center = x + 118 + run * 98
        height = round((base - top) * value / maximum)
        svg.rect(center - 26, base - height, 52, height, COLORS[index])
        svg.text(center, base - height - 9, f"{value:.2f}", 14, anchor="middle")
        svg.text(center, base + 22, str(run + 1), 15, anchor="middle")
svg.text(58, 683, "Ejes desde cero con escala propia por motor; compara cifras, no alturas, entre paneles.", 15, MUTED)
svg.text(58, 708, "Cada barra es el p95 de una corrida completa, no el tiempo de una consulta individual.", 15, MUTED)
svg.save("broader-comparison-p95-2026-10-16.svg")

codex = original["codex"]
lodis_rounds = []
lodis_tokens = []
basic_rounds = []
basic_tokens = []
for round_number in (1, 2):
    rows = json.loads((DATA / f"codex-lodis-valid-{round_number}/summary.json").read_text())["rows"]
    lodis_rounds.append(statistics.median(row["final_answer_ms"] for row in rows) / 1000)
    lodis_tokens.append(statistics.median(row["usage"]["input_tokens"] for row in rows))
    basic_rows = json.loads((DATA / f"codex-basic-memory-valid-{round_number}/summary.json").read_text())["rows"]
    basic_rounds.append(statistics.median(row["final_answer_ms"] for row in basic_rows) / 1000)
    basic_tokens.append(statistics.median(row["usage"]["input_tokens"] for row in basic_rows))


def original_median(kind, field):
    return statistics.median(codex[str(index)][kind][field] for index in (1, 2))


seed_codex = original_median("seed", "median_final_answer_ms") / 1000
engram_codex = original_median("engram", "median_final_answer_ms") / 1000
lodis_codex = statistics.median(lodis_rounds)
basic_codex = statistics.median(basic_rounds)
seed_tokens = original_median("seed", "median_input_tokens")
engram_tokens = original_median("engram", "median_input_tokens")
lodis_input_tokens = statistics.median(lodis_tokens)
basic_input_tokens = statistics.median(basic_tokens)
colors = COLORS
names = ["iROMEM", "Engram", "Lodis", "Basic Memory"]

def codex_values(numbers):
    return [(name, number, color) for name, number, color in zip(names, numbers, colors)]


svg = Svg("Codex: turno con memoria",
          "Cohorte 014–017 · 2 rondas de 6 turnos · mediana de las medianas de cada ronda")
panel(svg, 58, "Tiempo del turno Codex", "Semilla en prompt o búsqueda MCP", "segundos", 20,
      codex_values([seed_codex, engram_codex, lodis_codex, basic_codex]))
panel(svg, 610, "Tokens de entrada informados", "Incluyen contexto e instrucciones", "tokens", 64000,
      codex_values([seed_tokens, engram_tokens, lodis_input_tokens, basic_input_tokens]), 0)
svg.text(58, 578, "iROMEM empieza con la semilla lista; su preparación se muestra en otra figura.", 16, MUTED)
svg.text(58, 608, "Los tokens incluyen instrucciones y caché; no equivalen al coste marginal de la recuperación.", 16, MUTED)
svg.save("broader-comparison-codex-2026-10-16.svg")

svg = Svg("Tiempo hasta respuesta: recorrido calculado",
          "Dos rondas de seis preguntas · iROMEM incluye preparación de semilla con Decisions")
for index in (1, 2):
    x = 58 if index == 1 else 610
    values = [codex[str(index)]["calculated_seed_plus_recall_median_ms"] / 1000,
              codex[str(index)]["engram"]["median_final_answer_ms"] / 1000,
              lodis_rounds[index - 1], basic_rounds[index - 1]]
    panel(svg, x, f"Ronda {index}", "Mediana de seis casos emparejados", "segundos", 20,
          codex_values(values))
svg.text(58, 578, "iROMEM: suma por caso de recall + Codex medidos en ejecuciones separadas.", 16, MUTED)
svg.text(58, 608, "Otros motores: turno Codex con MCP. Falta medir el hook y la entrega real de iROMEM.", 16, MUTED)
svg.save("broader-comparison-calculated-flow-2026-10-16.svg")
