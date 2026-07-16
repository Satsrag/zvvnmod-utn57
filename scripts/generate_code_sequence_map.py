#!/usr/bin/env python3
"""生成 Rust ZVVNMOD code-sequence replacement Map。

Generate the Rust ZVVNMOD code-sequence replacement map.
"""

import argparse
from pathlib import Path

from generate_zvvnmod import generate_code_sequence_map


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--input",
        type=Path,
        default=root / "data" / "zvvnmod-unicode-names.csv",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=root / "src" / "generated" / "code_sequence_map.rs",
    )
    args = parser.parse_args()
    model = generate_code_sequence_map(args.input, args.output)
    print(
        f"generated {len(model.sequence_replacements)} sequence replacements "
        f"-> {args.output}"
    )


if __name__ == "__main__":
    main()
