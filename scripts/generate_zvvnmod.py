#!/usr/bin/env python3
"""从已审核数据生成 Rust ZVVNMOD 定义、replacement 与 UTN57 mapping。

Generate Rust ZVVNMOD definitions, replacements, and UTN57 mapping from reviewed data.
"""

from __future__ import annotations

import argparse
import csv
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from strict_csv import parse_metadata_table, parse_table

POSITION_WORDS = {
    "i": ("INIT", "Init"),
    "m": ("MEDI", "Medi"),
    "f": ("FINA", "Fina"),
    "isol": ("ISOL", "Isol"),
}

REVIEWED_RUNTIME_BASELINE = (
    "sha256:83a60c3e1ac9df98a14c1a6d979f7c5c8733f1e70d52b81f41de1dd321ea5016"
)

REVIEWED_MAPPING_AMBIGUITIES = {
    ("AA_FINA",): {("Aa:isol",), ("Aa:fina",)},
    ("K_INIT",): {("K:init",), ("K2:init",)},
    ("K_MEDI",): {("K:medi",), ("K2:medi",)},
    ("K_FINA",): {("K:fina",), ("K2:fina",)},
}

@dataclass(frozen=True)
class ParsedCodeName:
    rust_name: str
    units: tuple[str, ...]
    position: str | None
    component_names: tuple[str, ...] = ()


@dataclass(frozen=True)
class InputRow:
    codepoint: int
    name: str
    source: str


@dataclass(frozen=True)
class CodeEntry:
    """One authoritative CSV row resolved to its generated Rust constant."""

    codepoint: int
    const_name: str
    source_name: str
    source: str


@dataclass(frozen=True)
class CodeDecomposition:
    merged: CodeEntry
    components: tuple[CodeEntry, ...]


@dataclass(frozen=True)
class IrFinaReplacement:
    prefix: CodeEntry
    suffix: CodeEntry
    result: CodeEntry
    source: str


@dataclass(frozen=True)
class Utn57Target:
    id: str
    unit: str
    position: str
    glyph: str
    order: int

    @property
    def const_name(self) -> str:
        suffix = "CONTROL" if self.position == "control" else self.position.upper()
        return f"UTN57_{_unit_identifier(self.unit)}_{suffix}"


@dataclass(frozen=True)
class Utn57MappingRule:
    id: str
    sources: tuple[CodeEntry, ...]
    targets: tuple[Utn57Target, ...]


@dataclass(frozen=True)
class Utn57MappingModel:
    targets: tuple[Utn57Target, ...]
    rules: tuple[Utn57MappingRule, ...]
    baseline: str


@dataclass
class Model:
    codes: list[CodeEntry]
    code_decompositions: list[CodeDecomposition]


def _unit_identifier(unit: str) -> str:
    """将 written-unit 名称转换为 Rust 标识符。 / Convert a written-unit name to a Rust identifier."""

    identifier = "".join(ch if ch.isalnum() else "_" for ch in unit).upper()
    identifier = "_".join(part for part in identifier.split("_") if part)
    if not identifier or identifier[0].isdigit():
        raise ValueError(f"invalid written-unit ID: {unit!r}")
    return identifier


def parse_code_name(
    name: str, codepoint: int | None = None, source: str = "font"
) -> ParsedCodeName:
    """解析一行 ZVVNMOD shape 名称。 / Parse one ZVVNMOD shape name.

    示例 / Example: ``B i I m`` → ``B_I_INIT``.
    """

    name = name.strip()
    # 即使 CSV 误标为 font，旧 control 范围也不能重新进入正式 shape inventory。
    # Keep the legacy control range out of the formal shape inventory even if mislabeled as font.
    if codepoint is not None and 0xE140 <= codepoint <= 0xE144:
        raise ValueError(
            f"legacy control codepoint U+{codepoint:04X} is not a ZVVNMOD shape"
        )
    if source != "font":
        raise ValueError(f"unsupported source {source!r}")
    if not name:
        raise ValueError(f"missing name for U+{codepoint:04X}" if codepoint is not None else "missing name")
    if name == "Nirugu":
        return ParsedCodeName("NIRUGU", ("Nirugu",), None)

    parts = name.split()
    if len(parts) % 2:
        raise ValueError(f"name must contain unit/position pairs: {name!r}")

    units: list[str] = []
    short_positions: list[str] = []
    for index in range(0, len(parts), 2):
        unit, position = parts[index], parts[index + 1]
        if position not in POSITION_WORDS:
            raise ValueError(f"unknown position {position!r} in {name!r}")
        units.append(unit)
        short_positions.append(position)

    if len(units) > 1:
        # 多 shape 的首尾位置分别表示前后连接状态。
        # For a multi-part shape, the first and last positions encode its two joins.
        edge_positions = (short_positions[0], short_positions[-1])
        edge_variants = {
            ("i", "f"): ("ISOL", "Isol"),
            ("i", "m"): ("INIT", "Init"),
            ("m", "m"): ("MEDI", "Medi"),
            ("m", "f"): ("FINA", "Fina"),
            ("f", "f"): ("FINA", "Fina"),
        }
        if (
            edge_positions not in edge_variants
            or any(position != "m" for position in short_positions[1:-1])
        ):
            raise ValueError(f"invalid multi-shape positions in {name!r}")
        position_suffix, position_variant = edge_variants[edge_positions]
    else:
        position_suffix, position_variant = POSITION_WORDS[short_positions[-1]]

    rust_units = "_".join(_unit_identifier(unit) for unit in units)
    component_names = tuple(
        f"{_unit_identifier(unit)}_{POSITION_WORDS[short_position][0]}"
        for unit, short_position in zip(units, short_positions)
    )
    return ParsedCodeName(
        rust_name=f"{rust_units}_{position_suffix}",
        units=tuple(units),
        position=position_variant,
        component_names=component_names,
    )


def read_csv(path: Path) -> list[InputRow]:
    """读取并校验有效 CSV。 / Read and validate valid CSV."""

    with path.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.DictReader(handle)
        required = {"unicode", "name", "source"}
        if set(reader.fieldnames or ()) != required:
            raise ValueError(f"expected CSV header unicode,name,source; got {reader.fieldnames}")
        rows = []
        seen = set()
        for line_number, row in enumerate(reader, start=2):
            raw = row["unicode"].strip().lower().removeprefix("u+")
            try:
                codepoint = int(raw, 16)
            except ValueError as error:
                raise ValueError(f"line {line_number}: invalid codepoint {raw!r}") from error
            if codepoint in seen:
                raise ValueError(f"line {line_number}: duplicate U+{codepoint:04X}")
            seen.add(codepoint)
            rows.append(InputRow(codepoint, row["name"].strip(), row["source"].strip()))
    return rows


def build_model(rows: Iterable[InputRow]) -> Model:
    """构建 ZVVNMOD codes 和 decompositions。 / Build ZVVNMOD codes and decompositions."""

    parsed_rows: list[tuple[InputRow, ParsedCodeName]] = []
    for row in rows:
        parsed_rows.append((row, parse_code_name(row.name, row.codepoint, row.source)))

    codes: list[CodeEntry] = []
    codes_by_name: dict[str, CodeEntry] = {}
    code_by_value: dict[int, CodeEntry] = {}
    used_const_names: set[str] = set()
    for row, parsed in parsed_rows:
        const_name = parsed.rust_name
        if const_name in used_const_names:
            raise ValueError(f"duplicate generated code name {const_name}")
        used_const_names.add(const_name)
        entry = CodeEntry(row.codepoint, const_name, row.name, row.source)
        codes.append(entry)
        code_by_value[row.codepoint] = entry
        codes_by_name[const_name] = entry

    decompositions: list[CodeDecomposition] = []
    for row, parsed in parsed_rows:
        if len(parsed.component_names) <= 1:
            continue
        component_codes = [codes_by_name.get(name) for name in parsed.component_names]
        # CSV 中缺少 component code 时不补造 decomposition。
        # Do not invent a decomposition when a component code is absent from the CSV.
        if any(entry is None for entry in component_codes):
            continue
        components = tuple(entry for entry in component_codes if entry is not None)
        decompositions.append(
            CodeDecomposition(code_by_value[row.codepoint], components)
        )

    return Model(codes, decompositions)


def read_ir_fina_csv(path: Path, model: Model) -> list[IrFinaReplacement]:
    """读取并校验 Ir_fina 替换表。 / Read and validate the Ir_fina replacement table."""

    code_by_name = {entry.const_name: entry for entry in model.codes}
    with path.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.DictReader(handle)
        required = {"prefix_name", "ir_fina_name", "result_name", "source"}
        if set(reader.fieldnames or ()) != required:
            raise ValueError(
                "expected Ir_fina CSV header "
                f"prefix_name,ir_fina_name,result_name,source; got {reader.fieldnames}"
            )

        rules: list[IrFinaReplacement] = []
        seen_pairs: set[tuple[str, str]] = set()
        for line_number, row in enumerate(reader, start=2):
            names = {
                column: row[column].strip()
                for column in ("prefix_name", "ir_fina_name", "result_name")
            }
            entries: dict[str, CodeEntry] = {}
            for column, name in names.items():
                try:
                    entries[column] = code_by_name[name]
                except KeyError as error:
                    raise ValueError(
                        f"line {line_number}: unknown generated {column} {name!r}"
                    ) from error

            pair = (names["prefix_name"], names["ir_fina_name"])
            if pair in seen_pairs:
                raise ValueError(
                    f"line {line_number}: duplicate Ir_fina replacement "
                    f"{pair[0]} + {pair[1]}"
                )
            seen_pairs.add(pair)

            suffix = entries["ir_fina_name"]
            if suffix.const_name != "IR_FINA":
                raise ValueError(
                    f"line {line_number}: expected IR_FINA suffix, got {suffix.const_name}"
                )
            source = row["source"].strip()
            if source != "user-confirmed":
                raise ValueError(f"line {line_number}: unsupported source {source!r}")
            rules.append(
                IrFinaReplacement(
                    entries["prefix_name"],
                    suffix,
                    entries["result_name"],
                    source,
                )
            )
    return rules


def read_utn57_targets_csv(path: Path) -> tuple[Utn57Target, ...]:
    """读取 typed UTN57 target inventory。 / Read the typed UTN57 target inventory."""

    expected_fields = ["id", "unit", "position", "glyph"]
    try:
        rows = parse_table(path.read_text(encoding="utf-8"), expected_fields)
    except ValueError as error:
        raise ValueError(f"UTN57 target CSV: {error}") from error
    targets: list[Utn57Target] = []
    seen_ids: set[str] = set()
    seen_const_names: set[str] = set()
    for order, row in enumerate(rows, start=0):
        target_id = row["id"]
        unit = row["unit"]
        position = row["position"]
        glyph = row["glyph"]
        if not target_id or any(character.isspace() for character in target_id) or target_id in seen_ids:
            raise ValueError(f"UTN57 target row {order}: invalid or duplicate ID {target_id!r}")
        if re.fullmatch(r"[A-Z][A-Za-z0-9]*", unit) is None:
            raise ValueError(f"UTN57 target row {order}: invalid unit {unit!r}")
        if position not in {"isol", "init", "medi", "fina", "control"}:
            raise ValueError(f"UTN57 target row {order}: invalid position {position!r}")
        if not glyph:
            raise ValueError(f"UTN57 target row {order}: glyph must be non-empty")
        expected_id = unit if position == "control" else f"{unit}:{position}"
        if target_id != expected_id:
            raise ValueError(f"UTN57 target row {order}: expected ID {expected_id!r}")
        seen_ids.add(target_id)
        target = Utn57Target(target_id, unit, position, glyph, order)
        if target.const_name in seen_const_names:
            raise ValueError(
                f"UTN57 target row {order}: duplicate Rust constant {target.const_name}"
            )
        seen_const_names.add(target.const_name)
        targets.append(target)
    if not targets:
        raise ValueError("UTN57 target CSV must contain at least one target")
    return tuple(targets)


def read_utn57_mapping_csv(
    path: Path, model: Model, targets: tuple[Utn57Target, ...]
) -> Utn57MappingModel:
    """读取并验证 reviewed mapping CSV。 / Read and validate the reviewed mapping CSV."""

    code_by_name = {entry.const_name: entry for entry in model.codes}
    targets_by_id = {target.id: target for target in targets}
    expected_fields = ["id", "sources", "targets", "note"]
    try:
        metadata, rows = parse_metadata_table(
            path.read_text(encoding="utf-8"),
            expected_fields,
            ["schema", "baseline"],
        )
    except ValueError as error:
        raise ValueError(f"UTN57 mapping CSV: {error}") from error
    if (
        metadata["schema"] != "zvvnmod-utn57-runtime-map-v1"
        or metadata["baseline"] != REVIEWED_RUNTIME_BASELINE
    ):
        raise ValueError("UTN57 mapping CSV metadata differs from schema")
    rules: list[Utn57MappingRule] = []
    row_ids: set[str] = set()
    for index, row in enumerate(rows):
        row_id = row["id"]
        if (
            re.fullmatch(r"[A-Za-z0-9:_-]+", row_id) is None
            or row_id in row_ids
        ):
            raise ValueError(f"mapping {index}: invalid or duplicate ID {row_id!r}")
        row_ids.add(row_id)
        if row["sources"] == "" or row["targets"] == "":
            raise ValueError(f"mapping {row_id}: source and target sequences must be non-empty")
        source_ids = tuple(row["sources"].split(" "))
        target_ids = tuple(row["targets"].split(" "))
        if (
            any(not item or any(character.isspace() for character in item) for item in source_ids)
            or any(not item or any(character.isspace() for character in item) for item in target_ids)
        ):
            raise ValueError(f"mapping {row_id}: source and target sequences must use single spaces")
        try:
            source_entries = tuple(code_by_name[item] for item in source_ids)
        except KeyError as error:
            raise ValueError(f"mapping {row_id}: unknown source {error.args[0]!r}") from error
        try:
            target_entries = tuple(targets_by_id[item] for item in target_ids)
        except KeyError as error:
            raise ValueError(f"mapping {row_id}: unknown target {error.args[0]!r}") from error
        rules.append(Utn57MappingRule(row_id, source_entries, target_entries))

    if not rules:
        raise ValueError("UTN57 mapping CSV must contain at least one relation")
    targets_by_sources: dict[tuple[str, ...], set[tuple[str, ...]]] = {}
    for rule in rules:
        sources_key = tuple(entry.const_name for entry in rule.sources)
        targets_key = tuple(target.id for target in rule.targets)
        targets_by_sources.setdefault(sources_key, set()).add(targets_key)
    ambiguities = {
        sources_key: target_sequences
        for sources_key, target_sequences in targets_by_sources.items()
        if len(target_sequences) > 1
    }
    if ambiguities != REVIEWED_MAPPING_AMBIGUITIES:
        raise ValueError(
            "unsupported ambiguous mapping set: "
            f"expected {REVIEWED_MAPPING_AMBIGUITIES!r}, got {ambiguities!r}"
        )

    return Utn57MappingModel(tuple(targets), tuple(rules), metadata["baseline"])


def _render_code_list(entries: list[CodeEntry]) -> str:
    return ", ".join(entry.const_name for entry in entries)


# Derive the source sequence's overall position from its outer joining edges.
# 通过source sequence最外侧的连接边界推导整体位置。
def _source_sequence_position(entries: tuple[CodeEntry, ...]) -> str | None:
    positions = [
        parsed.position
        for entry in entries
        if (parsed := parse_code_name(entry.source_name, entry.codepoint, entry.source)).position
        is not None
    ]
    if not positions:
        return None
    left_connected = positions[0] in {"Medi", "Fina"}
    right_connected = positions[-1] in {"Init", "Medi"}
    return {
        (False, False): "Isol",
        (False, True): "Init",
        (True, True): "Medi",
        (True, False): "Fina",
    }[(left_connected, right_connected)]


def render_utn57_mapping_rust(
    mapping: Utn57MappingModel,
    names_source: str,
    targets_source: str,
    mapping_source: str,
) -> str:
    units = list(dict.fromkeys(target.unit for target in mapping.targets))
    lines = [
        "// Generated by scripts/generate_utn57_mapping.py — DO NOT EDIT.",
        f"// Sources: {names_source}, {targets_source}, {mapping_source}",
        "",
        "use super::zvvnmod_codes::*;",
        "",
        "/// A semantic UTN #57 written-unit identity.",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]",
        "pub enum Utn57WrittenUnit {",
    ]
    lines.extend(f"    {unit}," for unit in units)
    lines.extend(
        [
            "}",
            "",
            "impl Utn57WrittenUnit {",
            "    /// Stable unit spelling used by the public positioned-record contract.",
            "    pub const fn contract_name(self) -> &'static str {",
            "        match self {",
        ]
    )
    lines.extend(
        f'            Self::{unit} => "{"Mvs" if unit == "MVS" else unit}",'
        for unit in units
    )
    lines.extend(
        [
            "        }",
            "    }",
            "}",
            "",
            "/// A UTN #57 joining position or non-positional control kind.",
            "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]",
            "pub enum Utn57Position {",
            "    Isol,",
            "    Init,",
            "    Medi,",
            "    Fina,",
            "    Control,",
            "}",
            "",
            "impl Utn57Position {",
            "    /// Stable position spelling used by the public positioned-record contract.",
            "    pub const fn contract_name(self) -> &'static str {",
            "        match self {",
            '            Self::Isol => "isol",',
            '            Self::Init => "init",',
            '            Self::Medi => "medi",',
            '            Self::Fina => "fina",',
            '            Self::Control => "control",',
            "        }",
            "    }",
            "}",
            "",
            "/// One positioned UTN #57 written unit.",
            "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]",
            "pub struct Utn57PositionedWrittenUnit {",
            "    /// Semantic unit identity.",
            "    pub written_unit: Utn57WrittenUnit,",
            "    /// Joining position, or `Control` for a non-positional control.",
            "    pub position: Utn57Position,",
            "}",
            "",
            "impl Utn57PositionedWrittenUnit {",
            "    /// Construct a positioned UTN #57 written unit.",
            "    pub const fn new(written_unit: Utn57WrittenUnit, position: Utn57Position) -> Self {",
            "        Self {",
            "            written_unit,",
            "            position,",
            "        }",
            "    }",
            "}",
            "",
        ]
    )
    for target in mapping.targets:
        position = "Control" if target.position == "control" else target.position.title()
        lines.append(f"/// Reviewed target `{target.id}`.")
        declaration = (
            f"pub const {target.const_name}: Utn57PositionedWrittenUnit = "
            f"Utn57PositionedWrittenUnit::new(Utn57WrittenUnit::{target.unit}, Utn57Position::{position});"
        )
        if len(declaration) <= 100:
            lines.append(declaration)
        else:
            lines.append(f"pub const {target.const_name}: Utn57PositionedWrittenUnit =")
            lines.append(
                f"    Utn57PositionedWrittenUnit::new(Utn57WrittenUnit::{target.unit}, Utn57Position::{position});"
            )
    lines.extend(
        [
            "",
            "/// Complete reviewed UTN #57 positioned-written-unit inventory.",
            "pub static UTN57_POSITIONED_WRITTEN_UNITS: &[Utn57PositionedWrittenUnit] = &[",
        ]
    )
    lines.extend(f"    {target.const_name}," for target in mapping.targets)
    lines.extend(
        [
            "];",
            "",
            "/// One reviewed ZVVNMOD sequence → UTN #57 sequence relation row.",
            "#[derive(Clone, Copy, Debug, PartialEq, Eq)]",
            "pub struct ZvvnmodToUtn57Mapping {",
            "    /// Stable reviewed relation row ID.",
            "    pub id: &'static str,",
            "    /// Ordered ZVVNMOD source sequence.",
            "    pub sources: &'static [ZvvnmodCode],",
            "    /// Ordered UTN #57 target sequence.",
            "    pub targets: &'static [Utn57PositionedWrittenUnit],",
            "    /// Overall joining position implied by the source sequence.",
            "    pub intrinsic_position: Option<Utn57Position>,",
            "}",
            "",
            "/// Complete non-empty reviewed mapping relation.",
            "///",
            "/// The relation intentionally preserves contextual and K/K2 alternatives;",
            "/// executable conversion applies the documented resolution policy.",
            "pub static ZVVNMOD_TO_UTN57_MAPPINGS: &[ZvvnmodToUtn57Mapping] = &[",
        ]
    )
    for rule in mapping.rules:
        source_names = [entry.const_name for entry in rule.sources]
        target_names = [target.const_name for target in rule.targets]
        intrinsic_position = _source_sequence_position(rule.sources)
        lines.append(f"    // {rule.id}")
        lines.append("    ZvvnmodToUtn57Mapping {")
        lines.append(f'        id: "{rule.id}",')
        for field, names in (("sources", source_names), ("targets", target_names)):
            inline = f"        {field}: &[{', '.join(names)}],"
            if len(inline) <= 88:
                lines.append(inline)
            else:
                lines.append(f"        {field}: &[")
                lines.extend(f"            {name}," for name in names)
                lines.append("        ],")
        position_value = (
            f"Some(Utn57Position::{intrinsic_position})"
            if intrinsic_position is not None
            else "None"
        )
        lines.append(f"        intrinsic_position: {position_value},")
        lines.append("    },")
    lines.extend(["];", ""])
    return "\n".join(lines)


def render_codes_rust(model: Model, source_name: str) -> str:
    lines = [
        "// Generated by scripts/generate_zvvnmod_codes.py — DO NOT EDIT.",
        f"// Source: {source_name}",
        "",
        "/// A ZVVNMOD code value.",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]",
        "pub struct ZvvnmodCode(pub u32);",
        "",
        "impl ZvvnmodCode {",
        "    /// Return the Unicode code point.",
        "    pub const fn codepoint(self) -> u32 {",
        "        self.0",
        "    }",
        "    /// Convert to a Rust `char`.",
        "    pub fn as_char(self) -> Option<char> {",
        "        char::from_u32(self.0)",
        "    }",
        "}",
        "",
    ]

    for entry in model.codes:
        comment = entry.source_name or entry.const_name
        lines.append(f"/// Code U+{entry.codepoint:04X}: {comment} ({entry.source}).")
        lines.append(f"pub const {entry.const_name}: ZvvnmodCode = ZvvnmodCode(0x{entry.codepoint:04X});")
    lines.extend(
        [
            "",
            "/// Complete reviewed formal ZVVNMOD shape inventory.",
            "pub static ZVVNMOD_CODES: &[ZvvnmodCode] = &[",
        ]
    )
    inventory_entries = sorted(model.codes, key=lambda entry: entry.codepoint)
    inventory_width = max(len(entry.const_name) + 1 for entry in inventory_entries) + 1
    for entry in inventory_entries:
        token = f"{entry.const_name},"
        lines.append(f"    {token:<{inventory_width}}// U+{entry.codepoint:04X}")
    lines.extend(
        [
            "];",
            "",
            "/// Look up a character in the formal ZVVNMOD shape inventory.",
            "pub fn zvvnmod_code(character: char) -> Option<ZvvnmodCode> {",
            "    let codepoint = character as u32;",
            "    ZVVNMOD_CODES",
            "        .binary_search_by_key(&codepoint, |code| code.codepoint())",
            "        .ok()",
            "        .map(|index| ZVVNMOD_CODES[index])",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def render_code_decomposition_map_rust(model: Model, source_name: str) -> str:
    lines = [
        "// Generated by scripts/generate_code_decomposition_map.py — DO NOT EDIT.",
        f"// Source: {source_name}",
        "",
        "use super::zvvnmod_codes::*;",
        "use std::collections::HashMap;",
        "",
        "/// Merged ZVVNMOD codes and their component code sequences.",
        "pub static ZVVNMOD_CODE_DECOMPOSITIONS: &[(ZvvnmodCode, &[ZvvnmodCode])] = &[",
    ]
    for decomposition in model.code_decompositions:
        lines.append(
            f"    ({decomposition.merged.const_name}, "
            f"&[{_render_code_list(list(decomposition.components))}]),"
        )
    lines.extend(
        [
            "];",
            "",
            "/// Build merged ZVVNMOD code → component ZVVNMOD code sequence.",
            "pub fn zvvnmod_code_decomposition_map() -> HashMap<ZvvnmodCode, &'static [ZvvnmodCode]> {",
            "    ZVVNMOD_CODE_DECOMPOSITIONS.iter().copied().collect()",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def render_ir_fina_rust(
    rules: list[IrFinaReplacement], names_source: str, rules_source: str
) -> str:
    lines = [
        "// Generated by scripts/generate_ir_fina.py — DO NOT EDIT.",
        f"// Sources: {names_source}, {rules_source}",
        "",
        "use super::zvvnmod_codes::*;",
        "",
        "/// `Ir_fina` replacement rules as `(preceding code, result code)`.",
        "///",
        "/// `Ir_fina` is a ZVVNMOD helper with no standalone UTN counterpart. It changes",
        "/// the preceding form into a specific final form, so it must be consumed before",
        "/// later written-form or UTN conversion.",
        "pub static IR_FINA_REPLACEMENTS: &[(ZvvnmodCode, ZvvnmodCode)] = &[",
    ]
    for rule in rules:
        lines.append(
            f"    // {rule.prefix.const_name} + {rule.suffix.const_name} → "
            f"{rule.result.const_name} ({rule.source})"
        )
        lines.append(f"    ({rule.prefix.const_name}, {rule.result.const_name}),")
    lines.extend(
        [
            "];",
            "",
            "/// An `Ir_fina` helper without a supported preceding replacement.",
            "#[derive(Clone, Copy, Debug, PartialEq, Eq)]",
            "pub struct IrFinaReplacementError {",
            "    /// Index of `Ir_fina` in the input code stream.",
            "    pub index: usize,",
            "    /// Preceding code, or `None` when `Ir_fina` is the first code.",
            "    pub preceding: Option<ZvvnmodCode>,",
            "}",
            "",
            "impl std::fmt::Display for IrFinaReplacementError {",
            "    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {",
            "        match self.preceding {",
            "            Some(code) => write!(",
            "                formatter,",
            "                \"unmatched Ir_fina at index {} after U+{:04X}\",",
            "                self.index,",
            "                code.codepoint()",
            "            ),",
            "            None => write!(",
            "                formatter,",
            "                \"unmatched Ir_fina at index {} without a preceding code\",",
            "                self.index",
            "            ),",
            "        }",
            "    }",
            "}",
            "",
            "impl std::error::Error for IrFinaReplacementError {}",
            "",
            "/// Replace each `preceding + Ir_fina` helper sequence with its specific final form.",
            "///",
            "/// This replacement runs before decomposition, written-form, or UTN conversion",
            "/// because `Ir_fina` has no standalone UTN counterpart; it only modifies the",
            "/// preceding ZVVNMOD form.",
            "pub fn replace_ir_fina(input: &[ZvvnmodCode]) -> Result<Vec<ZvvnmodCode>, IrFinaReplacementError> {",
            "    let mut output = Vec::with_capacity(input.len());",
            "    for (index, &code) in input.iter().enumerate() {",
            "        if code != IR_FINA {",
            "            output.push(code);",
            "            continue;",
            "        }",
            "",
            "        let preceding = output.last().copied();",
            "        let Some(prefix) = preceding else {",
            "            return Err(IrFinaReplacementError { index, preceding });",
            "        };",
            "        let Some((_, result)) = IR_FINA_REPLACEMENTS",
            "            .iter()",
            "            .find(|(candidate, _)| *candidate == prefix)",
            "        else {",
            "            return Err(IrFinaReplacementError { index, preceding });",
            "        };",
            "        output.pop();",
            "        output.push(*result);",
            "    }",
            "    Ok(output)",
            "}",
            "",
        ]
    )
    return "\n".join(lines)


def generate_codes(input_path: Path, output_path: Path) -> Model:
    model = build_model(read_csv(input_path))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(render_codes_rust(model, input_path.name), encoding="utf-8")
    return model


def generate_code_decomposition_map(input_path: Path, output_path: Path) -> Model:
    model = build_model(read_csv(input_path))
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        render_code_decomposition_map_rust(model, input_path.name), encoding="utf-8"
    )
    return model


def generate_ir_fina(
    names_path: Path, rules_path: Path, output_path: Path
) -> list[IrFinaReplacement]:
    """生成 Ir_fina 替换 Rust API。 / Generate the Rust Ir_fina replacement API."""

    model = build_model(read_csv(names_path))
    rules = read_ir_fina_csv(rules_path, model)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        render_ir_fina_rust(rules, names_path.name, rules_path.name),
        encoding="utf-8",
    )
    return rules


def generate_utn57_mapping(
    names_path: Path, targets_path: Path, mapping_path: Path, output_path: Path
) -> Utn57MappingModel:
    """生成 reviewed UTN57 mapping Rust API。 / Generate the reviewed UTN57 mapping Rust API."""

    model = build_model(read_csv(names_path))
    targets = read_utn57_targets_csv(targets_path)
    mapping = read_utn57_mapping_csv(mapping_path, model, targets)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        render_utn57_mapping_rust(
            mapping, names_path.name, targets_path.name, mapping_path.name
        ),
        encoding="utf-8",
    )
    return mapping


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=root / "data" / "zvvnmod-unicode-names.csv")
    parser.add_argument("--codes-output", type=Path, default=root / "src" / "generated" / "zvvnmod_codes.rs")
    parser.add_argument(
        "--decomposition-map-output",
        type=Path,
        default=root / "src" / "generated" / "code_decomposition_map.rs",
    )
    parser.add_argument(
        "--ir-fina-input",
        type=Path,
        default=root / "data" / "ir-fina-replacements.csv",
    )
    parser.add_argument(
        "--ir-fina-output",
        type=Path,
        default=root / "src" / "generated" / "ir_fina.rs",
    )
    parser.add_argument(
        "--targets-input",
        type=Path,
        default=root / "data" / "utn57-written-units.csv",
    )
    parser.add_argument(
        "--mapping-input",
        type=Path,
        default=root / "data" / "zvvnmod-utn57-map.csv",
    )
    parser.add_argument(
        "--mapping-output",
        type=Path,
        default=root / "src" / "generated" / "utn57_mapping.rs",
    )
    args = parser.parse_args()
    model = generate_codes(args.input, args.codes_output)
    generate_code_decomposition_map(args.input, args.decomposition_map_output)
    rules = generate_ir_fina(args.input, args.ir_fina_input, args.ir_fina_output)
    mapping = generate_utn57_mapping(
        args.input, args.targets_input, args.mapping_input, args.mapping_output
    )
    print(
        f"generated {len(model.codes)} codes, "
        f"{len(model.code_decompositions)} code decompositions, "
        f"{len(rules)} Ir_fina replacements, "
        f"{len(mapping.targets)} UTN57 targets, and {len(mapping.rules)} mapping rows -> "
        f"{args.codes_output}, {args.decomposition_map_output}, {args.ir_fina_output}, "
        f"{args.mapping_output}"
    )


if __name__ == "__main__":
    main()
