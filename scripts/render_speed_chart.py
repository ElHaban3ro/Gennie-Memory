"""Render README speed and accuracy charts from frozen synthetic benchmarks."""

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


def median(values):
    return statistics.median(values)


def text(x, y, value, size=19, weight=400, fill="#17233b"):
    return (f'<text x="{x}" y="{y}" font-family="Arial,Helvetica,sans-serif" '
            f'font-size="{size}" font-weight="{weight}" fill="{fill}">'
            f'{escape(value)}</text>')


def bar(x, y, width, label, value, unit, color, scale):
    actual = round(width * value / scale, 2)
    display = Decimal(str(value)).quantize(Decimal("0.01"), rounding=ROUND_HALF_UP)
    return "\n".join([
        text(x, y - 10, label, 17, 600),
        f'<rect x="{x}" y="{y}" width="{width}" height="17" rx="8.5" fill="#e8edf3"/>',
        f'<rect x="{x}" y="{y}" width="{actual}" height="17" rx="8.5" fill="{color}"/>',
        text(x + width, y - 10, f"{display} {unit}", 17, 700, color).replace(
            f'x="{x + width}"', f'x="{x + width}" text-anchor="end"'),
    ])


def accuracy_bar(x, y, width, label, correct, total, color):
    if not 0 <= correct <= total:
        raise ValueError(f"Invalid accuracy count for {label}")
    actual = round(width * correct / total, 2)
    percentage = Decimal(100 * correct / total).quantize(Decimal("0.1"), rounding=ROUND_HALF_UP)
    return "\n".join([
        text(x, y - 10, label, 17, 600),
        text(x + width, y - 10, f"{correct}/{total} · {percentage}%", 17, 700, color).replace(
            f'x="{x + width}"', f'x="{x + width}" text-anchor="end"'),
        f'<rect x="{x}" y="{y}" width="{width}" height="22" rx="11" fill="#e8edf3"/>',
        f'<rect x="{x}" y="{y}" width="{actual}" height="22" rx="11" fill="{color}"/>',
    ])


def render_speed(local, engram_local, engram_turns, seeded_turns):
    values = {
        "keywords": local["after_keywords"]["p50_ms"],
        "engram_local": engram_local["p50_ms"],
        "engram_codex": median([row["final_answer_ms"] for row in engram_turns]) / 1000,
        "seed_codex": median([row["final_answer_ms"] for row in seeded_turns]) / 1000,
    }
    parts = [
        '<svg xmlns="http://www.w3.org/2000/svg" width="1120" height="455" viewBox="0 0 1120 455" role="img" aria-labelledby="title desc">',
        '<title id="title">Latencia medida de recuperación y respuesta</title>',
        '<desc id="desc">La búsqueda local de iROMEM por palabras clave tarda 5,23 milisegundos y Engram tarda 5,52. Codex con Engram tarda 9,49 segundos hasta respuesta y Codex con una semilla sintética inicial, 5,12 segundos; preparar esa semilla lleva tiempo adicional.</desc>',
        '<rect width="1120" height="455" rx="20" fill="#f5f8fc"/>',
        text(35, 48, "iROMEM · velocidad medida", 27, 700),
        '<rect x="30" y="72" width="520" height="318" rx="16" fill="#ffffff" stroke="#dce5ee"/>',
        '<rect x="570" y="72" width="520" height="318" rx="16" fill="#ffffff" stroke="#dce5ee"/>',
        text(55, 111, "Búsqueda local de candidatos", 20, 700),
        text(55, 136, "2.800 recuerdos · 540 preguntas · p50", 15, 400, "#52627a"),
        bar(55, 201, 465, "iROMEM palabras clave", values["keywords"], "ms", "#1a7f74", 6),
        bar(55, 289, 465, "Engram palabras clave", values["engram_local"], "ms", "#64748b", 6),
        text(595, 111, "Codex: pregunta → respuesta", 20, 700),
        text(595, 136, "6 preguntas sintéticas · p50", 15, 400, "#52627a"),
        bar(595, 201, 465, "Codex + Engram MCP", values["engram_codex"], "s", "#64748b", 10),
        bar(595, 289, 465, "Codex + semilla inicial simulada", values["seed_codex"], "s", "#1a7f74", 10),
        text(35, 422, "La semilla inicial requiere preparación aparte (~0,73 s p50); todavía no hay integración con Codex.", 16, 400, "#52627a"),
        '</svg>',
    ]
    SPEED_OUTPUT.write_text("\n".join(parts) + "\n")
    print(SPEED_OUTPUT)


def render_accuracy(local, engram_local, engram_turns, seeded_turns):
    positives = local["after_keywords"]["positives"]
    if positives != 460 or engram_local["recall_at_20"] > positives:
        raise ValueError("Unexpected candidate evaluation cohort")
    engram_by_id = {row["case_id"]: row for row in engram_turns}
    seeded_by_id = {row["case_id"]: row for row in seeded_turns}
    if len(engram_by_id) != 6 or engram_by_id.keys() != seeded_by_id.keys():
        raise ValueError("Codex runs do not contain the same six cases")
    if sum(row["positive"] for row in engram_turns) != 5:
        raise ValueError("Unexpected positive/negative split")
    # The saved answers were reviewed against each synthetic expected_fact.
    # Engram missed the changed-assumption positive; both runs answered the negative correctly.
    missed = engram_by_id[ENGRAM_MISSED_CASE]
    if not missed["positive"] or missed["final_answer"] != "No encuentro ese recuerdo":
        raise ValueError("The reviewed Engram miss changed")
    if any("No encuentro" in seeded_by_id[row["case_id"]]["final_answer"]
           for row in engram_turns if row["positive"]):
        raise ValueError("A reviewed seeded positive changed")
    parts = [
        '<svg xmlns="http://www.w3.org/2000/svg" width="1120" height="455" viewBox="0 0 1120 455" role="img" aria-labelledby="title desc">',
        '<title id="title">Aciertos observados en búsqueda y respuesta</title>',
        '<desc id="desc">En 460 consultas positivas, iROMEM por palabras clave incluyó el recuerdo correcto entre veinte candidatos en 452 casos y Engram en 454. En seis preguntas, Codex con semilla inicial respondió correctamente seis y Codex con Engram cinco.</desc>',
        '<rect width="1120" height="455" rx="20" fill="#f5f8fc"/>',
        text(35, 48, "iROMEM · aciertos observados", 27, 700),
        '<rect x="30" y="72" width="520" height="318" rx="16" fill="#ffffff" stroke="#dce5ee"/>',
        '<rect x="570" y="72" width="520" height="318" rx="16" fill="#ffffff" stroke="#dce5ee"/>',
        text(55, 111, "Recuerdo correcto entre 20", 20, 700),
        text(55, 136, "460 consultas positivas · 2.800 recuerdos", 15, 400, "#52627a"),
        accuracy_bar(55, 201, 465, "iROMEM palabras clave", local["after_keywords"]["top20"], positives, "#1a7f74"),
        accuracy_bar(55, 289, 465, "Engram palabras clave", engram_local["recall_at_20"], positives, "#64748b"),
        text(595, 111, "Respuesta correcta de Codex", 20, 700),
        text(595, 136, "6 preguntas · 5 con recuerdo y 1 sin él", 15, 400, "#52627a"),
        accuracy_bar(595, 201, 465, "Semilla inicial simulada", len(seeded_turns), len(seeded_turns), "#1a7f74"),
        accuracy_bar(595, 289, 465, "Engram MCP", len(engram_turns) - 1, len(engram_turns), "#64748b"),
        text(35, 422, "Las dos barras de cada panel comparten casos; los paneles miden etapas distintas.", 16, 400, "#52627a"),
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
