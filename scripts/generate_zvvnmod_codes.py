#!/usr/bin/env python3
"""生成 Rust ZVVNMOD 编码及合并 shape 定义。

Generate Rust ZVVNMOD code and merged-shape definitions.
"""

import argparse
from pathlib import Path

from generate_zvvnmod import generate_codes


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, default=root / "data" / "zvvnmod-unicode-names.csv")
    parser.add_argument("--output", type=Path, default=root / "src" / "generated" / "zvvnmod_codes.rs")
    args = parser.parse_args()
    model = generate_codes(args.input, args.output)
    print(f"generated {len(model.codes)} codes, {len(model.shapes)} shapes -> {args.output}")


if __name__ == "__main__":
    main()
