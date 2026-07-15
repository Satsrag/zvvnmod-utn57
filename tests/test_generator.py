import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GENERATOR = ROOT / "scripts" / "generate_zvvnmod.py"


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
        parsed = gen.parse_shape_name("A i")
        self.assertEqual(parsed.rust_name, "A_INIT")
        self.assertEqual(parsed.units, ("A",))
        self.assertEqual(parsed.position, "Init")

    def test_init_to_fina_multi_shape_becomes_isol(self):
        gen = load_generator()
        parsed = gen.parse_shape_name("B i I f")
        self.assertEqual(parsed.rust_name, "B_I_ISOL")
        self.assertEqual(parsed.units, ("B", "I"))
        self.assertEqual(parsed.position, "Isol")

    def test_multi_shape_ending_in_medi_becomes_medi(self):
        gen = load_generator()
        self.assertEqual(gen.parse_shape_name("B i I m").rust_name, "B_I_MEDI")
        self.assertEqual(gen.parse_shape_name("B m I m").rust_name, "B_I_MEDI")

    def test_controls_have_fixed_names(self):
        gen = load_generator()
        self.assertEqual(gen.parse_shape_name("", codepoint=0xE140).rust_name, "FVS1")
        self.assertEqual(gen.parse_shape_name("", codepoint=0xE141).rust_name, "FVS2")
        self.assertEqual(gen.parse_shape_name("", codepoint=0xE142).rust_name, "FVS3")
        self.assertEqual(gen.parse_shape_name("", codepoint=0xE143).rust_name, "MVS")

    def test_duplicate_shapes_keep_all_codes_and_first_is_canonical(self):
        gen = load_generator()
        rows = [
            gen.InputRow(0xE07F, "B i I m", "font"),
            gen.InputRow(0xE080, "B m I m", "font"),
        ]
        model = gen.build_model(rows)
        aliases = model.shape_to_codes["B_I_MEDI"]
        self.assertEqual([item.codepoint for item in aliases], [0xE07F, 0xE080])
        self.assertEqual(aliases[0].const_name, "B_I_MEDI")
        self.assertEqual(aliases[1].const_name, "B_I_MEDI_ALT_1")

    def test_code_generator_contains_names_but_not_maps(self):
        gen = load_generator()
        rows = [
            gen.InputRow(0xE000, "A i", "font"),
            gen.InputRow(0xE07F, "B i I m", "font"),
            gen.InputRow(0xE080, "B m I m", "font"),
            gen.InputRow(0xE140, "", "control-table"),
        ]
        output = gen.render_codes_rust(gen.build_model(rows), source_name="fixture.csv")
        self.assertIn("pub const A_INIT: ZvvnmodCode", output)
        self.assertIn("pub const B_I_MEDI_ALT_1: ZvvnmodCode", output)
        self.assertIn("pub enum ZvvnmodShape", output)
        self.assertIn("pub const FVS1: ZvvnmodCode", output)
        self.assertNotIn("CODE_TO_SHAPE", output)

    def test_map_generator_contains_bidirectional_maps(self):
        gen = load_generator()
        rows = [
            gen.InputRow(0xE000, "A i", "font"),
            gen.InputRow(0xE07F, "B i I m", "font"),
            gen.InputRow(0xE080, "B m I m", "font"),
        ]
        output = gen.render_shape_map_rust(gen.build_model(rows), source_name="fixture.csv")
        self.assertIn("use super::zvvnmod_codes::*;", output)
        self.assertIn("pub static CODE_TO_SHAPE", output)
        self.assertIn("pub fn shape_to_zvvnmod_map()", output)
        self.assertIn("B_I_MEDI_ALT_1", output)


if __name__ == "__main__":
    unittest.main()
