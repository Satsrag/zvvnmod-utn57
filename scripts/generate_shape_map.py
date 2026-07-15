#!/usr/bin/env python3
"""生成 Rust ZVVNMOD Shape → 编码别名 Map。

Generate the Rust ZVVNMOD Shape → code aliases map.
"""

import argparse
from pathlib import Path

from generate_zvvnmod import generate_shape_map


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=root / "data" / "zvvnmod-unicode-names.csv")
    parser.add_argument("--output", type=Path, default=root / "src" / "generated" / "shape_map.rs")
    args = parser.parse_args()
    model = generate_shape_map(args.input, args.output)
    aliases = sum(len(codes) - 1 for codes in model.shape_to_codes.values())
    print(f"generated {len(model.shapes)} shapes, {aliases} aliases -> {args.output}")


if __name__ == "__main__":
    main()
