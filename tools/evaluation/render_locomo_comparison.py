#!/usr/bin/env python3
"""Render the README's established comparison figures with LoCoMo results."""

from __future__ import annotations

import argparse
import html
import json
import math
from pathlib import Path
from typing import Any

PAPER = "#fffefa"
INK = "#25313b"
MUTED = "#617078"
GRID = "#d8dfdf"
COLORS = {
    "Gennie": "#18817e",
    "Engram": "#b77a27",
    "Lodis": "#617db1",
    "Basic Memory": "#8a6f9b",
    "Decisions": "#b75948",
}
EXPECTED_CASES = 1982
EXPECTED_POSITIVES = 1978


def load(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def esc(value: str) -> str:
    return html.escape(value, quote=True)


def text(x: float, y: float, value: str, size: int = 16, color: str = INK,
         anchor: str = "start", font: str = "Arial,sans-serif") -> str:
    return (f'<text x="{x:.1f}" y="{y:.1f}" fill="{color}" text-anchor="{anchor}" '
            f'font-family="{font}" font-size="{size}">{esc(value)}</text>')


def start(title: str, subtitle: str, height: int = 650) -> list[str]:
    return [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="{height}" '
        f'viewBox="0 0 1200 {height}" role="img" aria-label="{esc(title)}">',
        f'<rect width="100%" height="100%" fill="{PAPER}"/>',
        text(58, 75, title, 34, INK, font="Georgia,serif"),
        text(58, 110, subtitle, 17, MUTED),
    ]


def save(lines: list[str], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join([*lines, "</svg>"]) + "\n", encoding="utf-8")


def tool_for(label: str) -> str:
    if label.startswith("Gennie"):
        return "Gennie"
    if label.startswith("Engram"):
        return "Engram"
    if label.startswith("Lodis"):
        return "Lodis"
    if label.startswith("Basic Memory"):
        return "Basic Memory"
    raise ValueError(f"herramienta sin color: {label}")


def validate_inputs(reports: dict[str, dict[str, Any]], decisions: dict[str, Any]) -> None:
    for label, report in reports.items():
        overall = report.get("overall", {})
        if overall.get("cases") != EXPECTED_CASES or overall.get("positive") != EXPECTED_POSITIVES:
            raise ValueError(f"{label}: tamaño de cohorte distinto de LoCoMo v1")
        if not all(key in overall for key in ("recall_at_20", "top_1_accuracy", "latency_ms")):
            raise ValueError(f"{label}: falta una métrica requerida")
        latency = overall["latency_ms"]
        if not all(isinstance(latency.get(key), (int, float)) for key in ("p50", "p95")):
            raise ValueError(f"{label}: faltan latencias p50/p95")
        if latency["p50"] < 0 or latency["p95"] < latency["p50"]:
            raise ValueError(f"{label}: latencias inconsistentes")
    if (decisions.get("provider") != "decisions" or decisions.get("cases") != EXPECTED_CASES
            or decisions.get("positive") != EXPECTED_POSITIVES or decisions.get("negative") != 4):
        raise ValueError("Decisions: tamaño o proveedor distinto de LoCoMo v1")
    required = ("candidate_recall", "positive_correct", "negative_empty", "classifier_p50_ms",
                "classifier_p95_ms", "service_total_p50_ms", "service_total_p95_ms",
                "search_p50_ms", "search_p95_ms", "reported_cost_usd")
    if not all(isinstance(decisions.get(key), (int, float)) for key in required):
        raise ValueError("Decisions: faltan métricas requeridas")
    if (decisions["classifier_p95_ms"] < decisions["classifier_p50_ms"]
            or decisions["service_total_p95_ms"] < decisions["service_total_p50_ms"]):
        raise ValueError("Decisions: latencias p50/p95 inconsistentes")


def axis_panel(lines: list[str], *, x: int, y: int, width: int, height: int,
               heading: str, subtitle: str, bars: list[tuple[str, float, str]],
               maximum: float, ticks: list[float], value_label: str,
               decimals: int = 0, suffix: str = "", run_labels: bool = False) -> None:
    lines.extend([
        text(x, y, heading, 22, INK, font="Georgia,serif"),
        text(x, y + 30, subtitle, 16, MUTED),
        text(x + width, y + 56, value_label, 14, MUTED, "end"),
    ])
    plot_left, plot_right = x + 52, x + width
    plot_top, plot_bottom = y + 70, y + height - 50
    for value in ticks:
        py = plot_bottom - (plot_bottom - plot_top) * value / maximum
        lines.append(f'<line x1="{plot_left}" y1="{py:.1f}" x2="{plot_right}" y2="{py:.1f}" '
                     f'stroke="{GRID}" stroke-width="1"/>')
        rendered = f"{value:.{decimals}f}{suffix}"
        lines.append(text(plot_left - 10, py + 5, rendered, 14, MUTED, "end"))

    count = len(bars)
    band_width = (plot_right - plot_left) / max(1, count)
    bar_width = min(64, band_width * 0.62)
    for index, (label, value, tool) in enumerate(bars):
        center = plot_left + band_width * (index + 0.5)
        bar_height = max(1, (plot_bottom - plot_top) * value / maximum)
        bar_y = plot_bottom - bar_height
        lines.append(f'<rect x="{center - bar_width / 2:.1f}" y="{bar_y:.1f}" '
                     f'width="{bar_width:.1f}" height="{bar_height:.1f}" fill="{COLORS[tool]}"/>')
        above_y = max(plot_top - 8, bar_y - 10)
        value_text = f"{value:.{decimals}f}{suffix}"
        lines.append(text(center, above_y, value_text, 15, INK, "middle"))
        if run_labels:
            lines.append(text(center, plot_bottom + 22, label, 13, INK, "middle"))
        else:
            label_parts = []
            for part in label.replace(" · ", "\n").split("\n"):
                words = part.split()
                current = ""
                for word in words:
                    if current and len(current) + 1 + len(word) > 10:
                        label_parts.append(current)
                        current = word
                    else:
                        current = f"{current} {word}".strip()
                if current:
                    label_parts.append(current)
            for line_index, part in enumerate(label_parts):
                lines.append(text(center, plot_bottom + 24 + 18 * line_index, part, 14, INK, "middle"))


def render_accuracy(path: Path, reports: dict[str, dict[str, Any]]) -> None:
    lines = start(
        "Recuerdos recuperados en las mismas preguntas",
        "LoCoMo · 1.978 positivas · 272 recuerdos de sesión · barras desde cero",
    )
    recall_bars = []
    top_one_bars = []
    for label, report in reports.items():
        overall = report["overall"]
        recall_bars.append((label, round(overall["recall_at_20"] * overall["positive"]), tool_for(label)))
        top_one_bars.append((label, round(overall["top_1_accuracy"] * overall["positive"]), tool_for(label)))
    axis_panel(lines, x=58, y=144, width=470, height=390,
               heading="Entre 20 candidatos", subtitle="Recuerdo esperado presente",
               bars=recall_bars, maximum=1978, ticks=[0, 989, 1978],
               value_label="aciertos / 1.978")
    axis_panel(lines, x=610, y=144, width=470, height=390,
               heading="Primer candidato", subtitle="Recuerdo esperado en posición 1",
               bars=top_one_bars, maximum=1978, ticks=[0, 989, 1978],
               value_label="aciertos / 1.978")
    lines.append(text(58, 574, "Búsqueda de candidatos: una lista de resultados no equivale a una semilla seleccionada.", 16, MUTED))
    lines.append(text(58, 606, "Gennie usa el mismo color en ambas configuraciones; las etiquetas las distinguen.", 16, MUTED))
    save(lines, path)


def render_latency(path: Path, reports: dict[str, dict[str, Any]]) -> None:
    lines = start(
        "Tiempo de búsqueda de candidatos",
        "LoCoMo · 1.982 consultas · una corrida completa por motor",
    )
    p50_bars = []
    p95_bars = []
    for label, report in reports.items():
        latency = report["overall"]["latency_ms"]
        p50_bars.append((label, latency["p50"], tool_for(label)))
        p95_bars.append((label, latency["p95"], tool_for(label)))
    axis_panel(lines, x=58, y=144, width=470, height=390,
               heading="Búsqueda: mediana", subtitle="p50 de la corrida",
               bars=p50_bars, maximum=50, ticks=[0, 25, 50], value_label="milisegundos", decimals=1)
    axis_panel(lines, x=610, y=144, width=470, height=390,
               heading="Búsqueda: p95", subtitle="p95 de la corrida",
               bars=p95_bars, maximum=140, ticks=[0, 70, 140], value_label="milisegundos", decimals=1)
    lines.append(text(58, 574, "Gennie, Engram y Lodis: procesos persistentes; Basic Memory: API local en proceso.", 16, MUTED))
    lines.append(text(58, 606, "Las escalas de ambos paneles empiezan en cero; la diferencia de transporte limita la comparación.", 16, MUTED))
    save(lines, path)


def render_p95_runs(path: Path, reports: dict[str, dict[str, Any]]) -> None:
    lines = start(
        "Búsqueda de candidatos: p95 por corrida",
        "LoCoMo · 1.982 preguntas por motor · milisegundos",
        height=730,
    )
    panels = []
    for label, report in reports.items():
        panels.append((label, report["overall"]["latency_ms"]["p95"], tool_for(label)))
    positions = [(58, 142), (442, 142), (826, 142), (250, 408), (634, 408)]
    for (label, value, tool), (x, y) in zip(panels, positions):
        maximum = max(10, math.ceil(value / 10) * 10)
        axis_panel(lines, x=x, y=y, width=350, height=220,
                   heading=label.replace(" · ", " / "), subtitle="p95 de corrida 1",
                   bars=[("1", value, tool)], maximum=maximum,
                   ticks=[0, maximum / 2, maximum], value_label=f"escala 0–{maximum:g} ms",
                   decimals=1, suffix=" ms", run_labels=True)
    lines.append(text(58, 698, "Cada panel usa su propia escala. Solo se midió una corrida; esto no muestra variación entre corridas.", 15, MUTED))
    save(lines, path)


def render_seed(path: Path, decisions: dict[str, Any]) -> None:
    lines = start(
        "Gennie: tiempo para preparar la semilla",
        "LoCoMo · 1.982 consultas · búsqueda + Decisions + composición · milisegundos",
    )
    for heading, classifier_key, total_key, maximum, ticks in [
        ("Mediana", "classifier_p50_ms", "service_total_p50_ms", 500, [0, 250, 500]),
        ("p95", "classifier_p95_ms", "service_total_p95_ms", 1000, [0, 500, 1000]),
    ]:
        axis_panel(lines, x=58 if heading == "Mediana" else 610, y=144, width=470, height=390,
                   heading=heading, subtitle="Etapa Decisions y recorrido total",
                   bars=[("Decisions", decisions[classifier_key], "Decisions"),
                         ("Semilla", decisions[total_key], "Gennie")],
                   maximum=maximum, ticks=ticks, value_label="milisegundos", decimals=1)
    lines.append(text(58, 574, f"Búsqueda dentro de recall: {decisions['search_p50_ms']:.2f} ms p50 y {decisions['search_p95_ms']:.2f} ms p95.", 16, MUTED))
    lines.append(text(58, 606, "Decisions es una etapa de la semilla; los percentiles de etapas distintas no se suman.", 16, MUTED))
    save(lines, path)


def render(args: argparse.Namespace) -> None:
    reports = {
        "Gennie · palabras clave": load(args.keywords),
        "Gennie · consulta natural": load(args.natural),
        "Engram 3.4.0": load(args.engram),
        "Lodis 0.6.0": load(args.lodis),
        "Basic Memory": load(args.basic),
    }
    decisions = load(args.decisions)
    validate_inputs(reports, decisions)
    args.output.mkdir(parents=True, exist_ok=True)
    for obsolete in ("locomo-all-evidence.svg", "locomo-decisions-seed.svg"):
        (args.output / obsolete).unlink(missing_ok=True)
    render_accuracy(args.output / "locomo-retrieval-accuracy.svg", reports)
    render_latency(args.output / "locomo-search-latency.svg", reports)
    render_p95_runs(args.output / "locomo-search-p95-runs.svg", reports)
    render_seed(args.output / "locomo-seed-preparation.svg", decisions)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--keywords", type=Path, required=True)
    parser.add_argument("--natural", type=Path, required=True)
    parser.add_argument("--engram", type=Path, required=True)
    parser.add_argument("--lodis", type=Path, required=True)
    parser.add_argument("--basic", type=Path, required=True)
    parser.add_argument("--decisions", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    render(args)
    print(f"figuras LoCoMo guardadas en {args.output}")


if __name__ == "__main__":
    main()
