#!/usr/bin/env python3
"""生成 Rust Ir_fina 替换 API。

Generate the Rust Ir_fina replacement API.
"""

import argparse
from pathlib import Path

from generate_zvvnmod import generate_ir_fina


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--names-input",
        type=Path,
        default=root / "data" / "zvvnmod-unicode-names.csv",
    )
    parser.add_argument(
        "--rules-input",
        type=Path,
        default=root / "data" / "ir-fina-replacements.csv",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=root / "src" / "generated" / "ir_fina.rs",
    )
    args = parser.parse_args()
    rules = generate_ir_fina(args.names_input, args.rules_input, args.output)
    print(f"generated {len(rules)} Ir_fina replacements -> {args.output}")


if __name__ == "__main__":
    main()
