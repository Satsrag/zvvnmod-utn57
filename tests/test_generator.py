import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts" / "generate_zvvnmod.py"
NAMES = ROOT / "data" / "zvvnmod-unicode-names.csv"
IR_FINA_RULES = ROOT / "data" / "ir-fina-replacements.csv"
MAPPING = ROOT / "data" / "zvvnmod-utn57-map.json"
MAPPING_SHA256 = "fca0e9568e061a87fc2c0dc96f98a045af2872147091a00c4e0cca7804d053c8"


def load_generator():
    spec = importlib.util.spec_from_file_location("generate_zvvnmod", GENERATOR)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


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

    def test_mapping_omits_redundant_source_catalogue(self):
        payload = json.loads(MAPPING.read_text(encoding="utf-8"))
        self.assertEqual(
            set(payload),
            {"schema", "description", "targets", "mappings"},
        )

    def test_mapping_sources_are_validated_against_authoritative_csv(self):
        gen = load_generator()
        payload = json.loads(MAPPING.read_text(encoding="utf-8"))
        payload["mappings"][0]["sources"] = ["UNKNOWN_SOURCE"]
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "mapping.json"
            path.write_text(json.dumps(payload), encoding="utf-8")
            model = gen.build_model(gen.read_csv(NAMES))
            with self.assertRaisesRegex(ValueError, "unknown source 'UNKNOWN_SOURCE'"):
                gen.read_utn57_mapping_json(path, model)

    def test_unreviewed_mapping_ambiguity_is_rejected(self):
        gen = load_generator()
        payload = json.loads(MAPPING.read_text(encoding="utf-8"))
        payload["mappings"].append(
            {
                "id": "test:conflict",
                "note": "",
                "sources": ["B_INIT"],
                "targets": ["C:init"],
            }
        )
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "mapping.json"
            path.write_text(json.dumps(payload), encoding="utf-8")
            model = gen.build_model(gen.read_csv(NAMES))
            with self.assertRaisesRegex(ValueError, "unsupported ambiguous mapping"):
                gen.read_utn57_mapping_json(path, model)

    def test_reviewed_mapping_artifact_and_generated_relation_are_locked(self):
        gen = load_generator()
        self.assertEqual(hashlib.sha256(MAPPING.read_bytes()).hexdigest(), MAPPING_SHA256)
        model = gen.build_model(gen.read_csv(NAMES))
        mapping = gen.read_utn57_mapping_json(MAPPING, model)
        self.assertEqual(len(mapping.targets), 96)
        self.assertEqual(len(mapping.rules), 91)
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
            gen.generate_utn57_mapping(NAMES, MAPPING, generated)
            self.assertEqual(generated.read_bytes(), checked_in.read_bytes())


if __name__ == "__main__":
    unittest.main()
