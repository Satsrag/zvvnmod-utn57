#!/usr/bin/env python3
"""Derive a draft UTN #57 → ZVVNMOD reverse map from the reviewed forward map.

The forward map (`data/zvvnmod-utn57-map.csv`) is many-to-one: several ZVVNMOD
code sequences can name the same positioned written unit. Reverse conversion
needs exactly one ZVVNMOD spelling per unit, so this script picks one by rule
kind and writes the result to `data/utn57-zvvnmod-map.csv` for review.

Precedence, highest first:

1. `target:` rows — the reviewer named the row after its target, so it is the
   canonical ZVVNMOD spelling of that unit.
2. `source:` rows — one code, one unit.
3. `chachlag:` rows — the `X:fina MVS Aa:isol` triples.

`context:` and `particle:` rows are forward-only. They exist to accept
sequences users actually type (`A_MEDI AA_FINA`, `N_MEDI N_MEDI`) and are not
canonical spellings. Every inventory unit that no rule reaches gets a row with
empty `targets`, which the runtime treats as "no ZVVNMOD glyph".

Rows whose choice is a judgement call carry a `REVIEW:` note.
"""

from __future__ import annotations

import hashlib
import sys
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from strict_csv import parse_metadata_table, parse_table  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
FORWARD = ROOT / "data" / "zvvnmod-utn57-map.csv"
INVENTORY = ROOT / "data" / "utn57-written-units.csv"
OUTPUT = ROOT / "data" / "utn57-zvvnmod-map.csv"

PRECEDENCE = {"target": 0, "source": 1, "chachlag": 2}

# Golden-corpus witnesses for units that only FVS-forced spellings produce.
UNREPRESENTABLE_WITNESSES = {
    "Gx:init": "ᠬ᠏ (h+FVS4)",
    "Gx:medi": "ᠰᠠᠬ᠏ᠠᠯ (h+FVS4 medial)",
    "Hx:fina": "ᠪᠠᠳᠠᠭ᠍ (g+FVS3 final); representable only inside chachlag as HX_AA_FINA",
    "Ix:isol": "ᠢ᠌ (i+FVS2)",
    "N:fina": "ᠣᠨ᠋ (n+FVS1 final); plain final n shapes as A:fina; representable only inside chachlag as N_AA_FINA",
    "Sz:fina": "ᠬᠦᠮᠦᠰ᠋ (s+FVS1 final)",
    "Ux:isol": "ᠲᠠᠨ᠎ᠤ᠌ (u+FVS2 after MVS)",
}


def main() -> None:
    forward_text = FORWARD.read_text(encoding="utf-8")
    _, forward_rows = parse_metadata_table(
        forward_text, ["id", "sources", "targets", "note"], ["schema", "baseline"]
    )
    inventory = parse_table(
        INVENTORY.read_text(encoding="utf-8"), ["id", "unit", "position", "glyph"]
    )

    candidates: dict[tuple[str, ...], list[tuple[str, str]]] = defaultdict(list)
    for row in forward_rows:
        kind = row["id"].split(":", 1)[0]
        if kind not in PRECEDENCE:
            continue
        candidates[tuple(row["targets"].split(" "))].append((row["id"], row["sources"]))

    # Forward-only spellings, reported in notes so the reviewer sees what was not chosen.
    alternatives: dict[tuple[str, ...], list[tuple[str, str]]] = defaultdict(list)
    for row in forward_rows:
        kind = row["id"].split(":", 1)[0]
        if kind in ("context",):
            alternatives[tuple(row["targets"].split(" "))].append((row["id"], row["sources"]))

    def choose(key: tuple[str, ...]) -> tuple[str, str] | None:
        options = candidates.get(key)
        if not options:
            return None
        options = sorted(options, key=lambda item: PRECEDENCE[item[0].split(":", 1)[0]])
        return options[0]

    def note_for(key: tuple[str, ...], chosen: tuple[str, str] | None) -> str:
        unit_id = " ".join(key)
        if chosen is None:
            witness = UNREPRESENTABLE_WITNESSES.get(unit_id, "not reached by any forward rule")
            return f"REVIEW: no ZVVNMOD glyph; seen only as {witness}"
        rule_id, sources = chosen
        parts = [f"from {rule_id}"]
        others = sorted(
            {src for rid, src in candidates[key] if src != sources}
            | {src for rid, src in alternatives.get(key, [])}
        )
        if others:
            ids = [rid for rid, src in candidates[key] + alternatives.get(key, []) if src != sources]
            parts.insert(0, "REVIEW: alternative spelling " + " / ".join(others) + f" ({'; '.join(ids)}) not chosen")
        if len(sources.split(" ")) > 1 and len(key) == 1:
            parts.append("composite glyph sequence")
        return "; ".join(parts)

    lines: list[str] = []
    digest = hashlib.sha256(forward_text.encode("utf-8")).hexdigest()
    lines.append(
        '# metadata={"schema":"utn57-zvvnmod-runtime-map-v1","baseline":"sha256:%s"}' % digest
    )
    lines.append("id,sources,targets,note")

    def emit(row_id: str, sources: str, targets: str, note: str) -> None:
        for value in (row_id, sources, targets, note):
            if "," in value or '"' in value:
                raise ValueError(f"field must not contain commas or quotes: {value!r}")
        lines.append(f"{row_id},{sources},{targets},{note}")

    # One row per inventory entry, in inventory order.
    for entry in inventory:
        unit_id = entry["id"]
        chosen = choose((unit_id,))
        note = note_for((unit_id,), chosen)
        if unit_id == "MVS":
            emit("control:MVS", unit_id, "", "structural; passed through as its source character unless a chachlag row consumes it")
            continue
        kind = "control" if entry["position"] == "control" else "unit"
        if unit_id in ("K:init", "K:medi", "K:fina"):
            note += f"; K2:{entry['position']} shares this glyph"
        if unit_id == "Dd:fina":
            note += "; particle:25 spells O:medi Dd:fina with one O_MEDI so ᠨᠤᠭᠤᠳ reverses with one more O_MEDI than that particle row"
        if unit_id == "Hx:medi":
            note = "REVIEW: particle:05/32/44 spell this unit N_MEDI N_MEDI (not chosen); " + note
        emit(f"{kind}:{unit_id}", unit_id, chosen[1] if chosen else "", note)

    # Multi-unit rows: the chachlag triples, in forward-row order.
    for row in forward_rows:
        key = tuple(row["targets"].split(" "))
        if len(key) < 2:
            continue
        kind = row["id"].split(":", 1)[0]
        if kind not in PRECEDENCE:
            continue
        chosen = choose(key)
        assert chosen is not None
        if chosen[0] != row["id"]:
            continue  # a higher-precedence row already emitted this key
        emit("chachlag:" + key[0], " ".join(key), chosen[1], f"from {row['id']}; MVS is implicit between the two glyphs")

    text = "\n".join(lines) + "\n"
    # The reviewed file must round-trip through the same strict parser the generators use.
    parse_metadata_table(text, ["id", "sources", "targets", "note"], ["schema", "baseline"])
    OUTPUT.write_text(text, encoding="utf-8")
    print(f"wrote {OUTPUT.relative_to(ROOT)}: {len(lines) - 2} rows")


if __name__ == "__main__":
    main()
