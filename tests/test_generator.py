import csv
import hashlib
import importlib.util
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts" / "generate_zvvnmod.py"
NAMES = ROOT / "data" / "zvvnmod-unicode-names.csv"
IR_FINA_RULES = ROOT / "data" / "ir-fina-replacements.csv"
MAPPING = ROOT / "data" / "zvvnmod-utn57-map.csv"
TARGETS = ROOT / "data" / "utn57-written-units.csv"
MAPPING_SHA256 = "5816e1d56e8b3fa7da7f2114562da463c4d449528aac3f9b73aade3afa157da0"
TARGETS_SHA256 = "a7635637c245f25144ee5d938a76c4dc83063953100bf7d7f8c61353826dfc26"
PARTICLE_RELATIONS_SHA256 = "396563dfa46cad6225fc92d07e7a6cc2e7c563f155bf70351ceb9448fbf75e5e"


def load_generator():
    spec = importlib.util.spec_from_file_location("generate_zvvnmod", GENERATOR)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def read_mapping_rows(path=MAPPING):
    with path.open(newline="", encoding="utf-8") as handle:
        rows = list(csv.DictReader(handle))
    return [
        {
            "id": row["id"],
            "note": row["note"],
            "sources": row["sources"].split(),
            "targets": row["targets"].split(),
        }
        for row in rows
    ]


def write_mapping_rows(path, rows):
    buffer = io.StringIO(newline="")
    writer = csv.DictWriter(
        buffer,
        fieldnames=["id", "sources", "targets", "note"],
        lineterminator="\n",
    )
    writer.writeheader()
    for row in rows:
        writer.writerow(
            {
                "id": row["id"],
                "sources": " ".join(row["sources"]),
                "targets": " ".join(row["targets"]),
                "note": row["note"],
            }
        )
    path.write_text(buffer.getvalue(), encoding="utf-8")


class ShapeNamingTests(unittest.TestCase):
    def test_single_shape_uses_expanded_position(self):
        gen = load_generator()
        parsed = gen.parse_code_name("A i")
        self.assertEqual(parsed.rust_name, "A_INIT")
        self.assertEqual(parsed.units, ("A",))
        self.assertEqual(parsed.position, "Init")

    def test_init_to_fina_multi_shape_becomes_isol(self):
        gen = load_generator()
        parsed = gen.parse_code_name("B i I f")
        self.assertEqual(parsed.rust_name, "B_I_ISOL")
        self.assertEqual(parsed.units, ("B", "I"))
        self.assertEqual(parsed.position, "Isol")

    def test_fina_to_fina_multi_code_becomes_fina(self):
        gen = load_generator()
        parsed = gen.parse_code_name("N f Aa f")
        self.assertEqual(parsed.rust_name, "N_AA_FINA")
        self.assertEqual(parsed.units, ("N", "Aa"))
        self.assertEqual(parsed.position, "Fina")
        self.assertEqual(parsed.component_names, ("N_FINA", "AA_FINA"))

    def test_multi_shape_position_uses_both_edges(self):
        gen = load_generator()
        self.assertEqual(gen.parse_code_name("B i I f").rust_name, "B_I_ISOL")
        self.assertEqual(gen.parse_code_name("B i I m").rust_name, "B_I_INIT")
        self.assertEqual(gen.parse_code_name("B m I m").rust_name, "B_I_MEDI")
        self.assertEqual(gen.parse_code_name("B m I f").rust_name, "B_I_FINA")
        self.assertEqual(gen.parse_code_name("G i O m I m").rust_name, "G_O_I_INIT")

    def test_multi_shape_rejects_invalid_position_sequence(self):
        gen = load_generator()
        with self.assertRaisesRegex(ValueError, "invalid multi-shape positions"):
            gen.parse_code_name("B f I m")

    def test_authoritative_inventory_excludes_legacy_controls(self):
        gen = load_generator()
        rows = gen.read_csv(NAMES)
        self.assertTrue(all(row.source == "font" for row in rows))
        self.assertTrue(
            {row.codepoint for row in rows}.isdisjoint(range(0xE140, 0xE144))
        )

    def test_control_table_rows_are_rejected(self):
        gen = load_generator()
        with self.assertRaisesRegex(ValueError, "unsupported source 'control-table'"):
            gen.parse_code_name("Joiner", codepoint=0xE200, source="control-table")

    def test_reserved_legacy_control_codepoints_are_rejected_as_font_rows(self):
        gen = load_generator()
        with self.assertRaisesRegex(
            ValueError, "legacy control codepoint U\\+E140 is not a ZVVNMOD shape"
        ):
            gen.build_model([gen.InputRow(0xE140, "A i", "font")])

    def test_merged_zvvnmod_code_maps_to_component_sequence(self):
        gen = load_generator()
        rows = [
            gen.InputRow(0xE006, "I m", "font"),
            gen.InputRow(0xE029, "B i", "font"),
            gen.InputRow(0xE07F, "B i I m", "font"),
        ]
        model = gen.build_model(rows)
        self.assertEqual(len(model.code_decompositions), 1)
        decomposition = model.code_decompositions[0]
        self.assertEqual(decomposition.merged.codepoint, 0xE07F)
        self.assertEqual(
            [item.codepoint for item in decomposition.components],
            [0xE029, 0xE006],
        )

    def test_missing_component_code_does_not_create_decomposition(self):
        gen = load_generator()
        model = gen.build_model([gen.InputRow(0xE07F, "B i I m", "font")])
        self.assertEqual(model.code_decompositions, [])

    def test_duplicate_generated_code_name_is_rejected(self):
        gen = load_generator()
        rows = [
            gen.InputRow(0xE029, "B i", "font"),
            gen.InputRow(0xE02A, "B i", "font"),
        ]
        with self.assertRaisesRegex(ValueError, "duplicate generated code name B_INIT"):
            gen.build_model(rows)

    def test_code_generator_contains_names_but_not_maps(self):
        gen = load_generator()
        rows = [
            gen.InputRow(0xE000, "A i", "font"),
            gen.InputRow(0xE006, "I m", "font"),
            gen.InputRow(0xE029, "B i", "font"),
            gen.InputRow(0xE07F, "B i I m", "font"),
        ]
        output = gen.render_codes_rust(gen.build_model(rows), source_name="fixture.csv")
        self.assertIn("pub const A_INIT: ZvvnmodCode", output)
        self.assertIn("pub const B_I_INIT: ZvvnmodCode", output)
        self.assertNotIn("ZvvnmodShape", output)
        self.assertNotIn("pub const FVS1: ZvvnmodCode", output)
        self.assertIn("Generated by scripts/generate_zvvnmod_codes.py", output)
        self.assertNotIn("自动生成", output)
        self.assertNotIn("编码值", output)
        self.assertNotIn("CODE_TO_SHAPE", output)

    def test_map_generator_maps_merged_codes_to_component_sequences(self):
        gen = load_generator()
        rows = [
            gen.InputRow(0xE000, "A i", "font"),
            gen.InputRow(0xE006, "I m", "font"),
            gen.InputRow(0xE029, "B i", "font"),
            gen.InputRow(0xE07F, "B i I m", "font"),
        ]
        output = gen.render_code_decomposition_map_rust(
            gen.build_model(rows), source_name="fixture.csv"
        )
        self.assertIn("use super::zvvnmod_codes::*;", output)
        self.assertIn("pub static ZVVNMOD_CODE_DECOMPOSITIONS", output)
        self.assertIn("(B_I_INIT, &[B_INIT, I_MEDI])", output)
        self.assertIn("pub fn zvvnmod_code_decomposition_map()", output)
        self.assertNotIn("ZvvnmodShape", output)
        self.assertNotIn("CODE_TO_SHAPE", output)
        self.assertNotIn("所有具名", output)

    def test_corrected_hx_and_chachleg_rows_use_final_components(self):
        gen = load_generator()
        rows = gen.read_csv(NAMES)
        by_codepoint = {row.codepoint: row for row in rows}
        self.assertEqual(by_codepoint[0xE034].name, "Hx i")
        self.assertEqual(by_codepoint[0xE077].name, "N f Aa f")
        self.assertEqual(by_codepoint[0xE09D].name, "Hx f Aa f")

        model = gen.build_model(rows)
        decomposed_codes = {
            decomposition.merged.codepoint for decomposition in model.code_decompositions
        }
        self.assertNotIn(0xE077, decomposed_codes)
        self.assertNotIn(0xE09D, decomposed_codes)

    def test_checked_in_code_decomposition_map_is_fresh(self):
        gen = load_generator()
        checked_in = ROOT / "src" / "generated" / "code_decomposition_map.rs"
        with tempfile.TemporaryDirectory() as directory:
            generated = Path(directory) / "code_decomposition_map.rs"
            gen.generate_code_decomposition_map(NAMES, generated)
            self.assertEqual(generated.read_bytes(), checked_in.read_bytes())

    def test_checked_in_code_definitions_are_fresh(self):
        gen = load_generator()
        checked_in = ROOT / "src" / "generated" / "zvvnmod_codes.rs"
        with tempfile.TemporaryDirectory() as directory:
            generated = Path(directory) / "zvvnmod_codes.rs"
            gen.generate_codes(NAMES, generated)
            self.assertEqual(generated.read_bytes(), checked_in.read_bytes())

    def test_ir_fina_rules_resolve_readable_names_from_the_main_table(self):
        gen = load_generator()
        model = gen.build_model(gen.read_csv(NAMES))
        rules = gen.read_ir_fina_csv(IR_FINA_RULES, model)
        self.assertEqual(len(rules), 30)
        self.assertEqual(rules[0].prefix.const_name, "O_MEDI")
        self.assertEqual(rules[0].suffix.const_name, "IR_FINA")
        self.assertEqual(rules[0].result.const_name, "UE_FINA")
        self.assertTrue(all(rule.source == "user-confirmed" for rule in rules))
        self.assertNotIn("e008", IR_FINA_RULES.read_text(encoding="utf-8"))

    def test_ir_fina_generator_emits_english_only_rust(self):
        gen = load_generator()
        model = gen.build_model(gen.read_csv(NAMES))
        rules = gen.read_ir_fina_csv(IR_FINA_RULES, model)
        output = gen.render_ir_fina_rust(
            rules,
            names_source=NAMES.name,
            rules_source=IR_FINA_RULES.name,
        )
        self.assertIn("pub static IR_FINA_REPLACEMENTS", output)
        self.assertIn("(O_MEDI, UE_FINA)", output)
        self.assertIn("pub fn replace_ir_fina", output)
        self.assertIn("has no standalone UTN counterpart", output)
        self.assertIn("runs before decomposition", output)
        self.assertNotRegex(output, r"[\u3400-\u9fff]")

    def test_ir_fina_rules_reject_duplicate_pairs(self):
        gen = load_generator()
        model = gen.build_model(gen.read_csv(NAMES))
        content = (
            "prefix_name,ir_fina_name,result_name,source\n"
            "O_MEDI,IR_FINA,UE_FINA,user-confirmed\n"
            "O_MEDI,IR_FINA,UE_FINA,user-confirmed\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "duplicate.csv"
            path.write_text(content, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "duplicate Ir_fina replacement"):
                gen.read_ir_fina_csv(path, model)

    def test_checked_in_ir_fina_file_is_fresh(self):
        gen = load_generator()
        checked_in = ROOT / "src" / "generated" / "ir_fina.rs"
        with tempfile.TemporaryDirectory() as directory:
            generated = Path(directory) / "ir_fina.rs"
            gen.generate_ir_fina(NAMES, IR_FINA_RULES, generated)
            self.assertEqual(generated.read_bytes(), checked_in.read_bytes())

    def test_target_csv_is_the_typed_utn57_authority(self):
        gen = load_generator()
        self.assertEqual(hashlib.sha256(TARGETS.read_bytes()).hexdigest(), TARGETS_SHA256)
        targets = gen.read_utn57_targets_csv(TARGETS)
        self.assertEqual(len(targets), 96)
        self.assertEqual(targets[0].id, "A:isol")
        self.assertEqual(targets[-1].id, "Nirugu")
        self.assertEqual(targets[-1].position, "control")

    def test_target_csv_rejects_surplus_columns_and_rust_name_collisions(self):
        gen = load_generator()
        cases = (
            (
                "id,unit,position\nA:init,A,init,unexpected\n",
                "malformed CSV row",
            ),
            (
                "id,unit,position\nAa:init,Aa,init\nAA:init,AA,init\n",
                "duplicate Rust constant UTN57_AA_INIT",
            ),
        )
        for content, message in cases:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "targets.csv"
                path.write_text(content, encoding="utf-8")
                with self.assertRaisesRegex(ValueError, message):
                    gen.read_utn57_targets_csv(path)

    def test_mapping_csv_rows_are_all_non_empty_ordered_relations(self):
        rows = read_mapping_rows()
        self.assertEqual(len(rows), 138)
        self.assertTrue(all(row["sources"] and row["targets"] for row in rows))
        particle_37 = next(row for row in rows if row["id"] == "particle:37")
        self.assertEqual(
            particle_37["sources"], ["D_INIT", "A_MEDI", "I_MEDI", "AA_FINA"]
        )

    def test_particle_relation_subset_is_complete_and_locked(self):
        particles = [
            row for row in read_mapping_rows() if row["id"].startswith("particle:")
        ]
        self.assertEqual(
            [row["id"] for row in particles],
            [f"particle:{index:02d}" for index in range(1, 48)],
        )
        canonical = json.dumps(
            particles, ensure_ascii=False, separators=(",", ":")
        ).encode("utf-8")
        self.assertEqual(
            hashlib.sha256(canonical).hexdigest(), PARTICLE_RELATIONS_SHA256
        )
        self.assertEqual(
            len(
                {
                    (tuple(row["sources"]), tuple(row["targets"]))
                    for row in particles
                }
            ),
            34,
        )
        corrections = {
            row["id"]: (row["sources"], row["targets"])
            for row in particles
            if row["id"]
            in {
                "particle:05",
                "particle:15",
                "particle:16",
                "particle:25",
                "particle:32",
                "particle:37",
                "particle:44",
            }
        }
        self.assertEqual(len(corrections), 7)
        self.assertEqual(
            corrections["particle:37"],
            (
                ["D_INIT", "A_MEDI", "I_MEDI", "AA_FINA"],
                ["D:init", "A:medi", "G:fina"],
            ),
        )

    def test_mapping_csv_is_fail_closed(self):
        gen = load_generator()
        model = gen.build_model(gen.read_csv(NAMES))
        targets = gen.read_utn57_targets_csv(TARGETS)
        cases = (
            (
                "id,sources,targets,note\ntest,UNKNOWN_SOURCE,B:init,\n",
                "unknown source 'UNKNOWN_SOURCE'",
            ),
            (
                "id,sources,targets,note\ntest,B_INIT,B:init,,surplus\n",
                "malformed CSV row",
            ),
            (
                "id,sources,targets,note\ntest,,B:init,\n",
                "source and target sequences must be non-empty",
            ),
        )
        for content, message in cases:
            with self.subTest(message=message), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "mapping.csv"
                path.write_text(content, encoding="utf-8")
                with self.assertRaisesRegex(ValueError, message):
                    gen.read_utn57_mapping_csv(path, model, targets)

    def test_unreviewed_mapping_ambiguity_is_rejected(self):
        gen = load_generator()
        rows = read_mapping_rows()
        rows.append(
            {
                "id": "test:conflict",
                "note": "",
                "sources": ["B_INIT"],
                "targets": ["C:init"],
            }
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "mapping.csv"
            write_mapping_rows(path, rows)
            model = gen.build_model(gen.read_csv(NAMES))
            with self.assertRaisesRegex(ValueError, "unsupported ambiguous mapping"):
                gen.read_utn57_mapping_csv(
                    path, model, gen.read_utn57_targets_csv(TARGETS)
                )

    def test_reviewed_mapping_artifact_and_generated_relation_are_locked(self):
        gen = load_generator()
        self.assertEqual(hashlib.sha256(MAPPING.read_bytes()).hexdigest(), MAPPING_SHA256)
        model = gen.build_model(gen.read_csv(NAMES))
        mapping = gen.read_utn57_mapping_csv(
            MAPPING, model, gen.read_utn57_targets_csv(TARGETS)
        )
        self.assertEqual(len(mapping.targets), 96)
        self.assertEqual(len(mapping.rules), 138)
        self.assertEqual(mapping.targets[-1].id, "Nirugu")
        self.assertEqual(mapping.targets[-1].position, "control")
        nirugu_rule = next(rule for rule in mapping.rules if rule.id == "source:NIRUGU")
        self.assertEqual([entry.const_name for entry in nirugu_rule.sources], ["NIRUGU"])
        self.assertEqual([target.id for target in nirugu_rule.targets], ["Nirugu"])

    def test_checked_in_utn57_mapping_is_fresh(self):
        gen = load_generator()
        checked_in = ROOT / "src" / "generated" / "utn57_mapping.rs"
        with tempfile.TemporaryDirectory() as directory:
            generated = Path(directory) / "utn57_mapping.rs"
            gen.generate_utn57_mapping(NAMES, TARGETS, MAPPING, generated)
            self.assertEqual(generated.read_bytes(), checked_in.read_bytes())


if __name__ == "__main__":
    unittest.main()
