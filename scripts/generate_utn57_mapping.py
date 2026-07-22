#!/usr/bin/env python3
"""从 reviewed JSON 生成 Rust ZVVNMOD→UTN57 mapping relation。

Generate the Rust ZVVNMOD→UTN57 mapping relation from the reviewed JSON.
"""

import argparse
from pathlib import Path

from generate_zvvnmod import generate_utn57_mapping


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--names-input",
        type=Path,
        default=root / "data" / "zvvnmod-unicode-names.csv",
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
        "--output",
        type=Path,
        default=root / "src" / "generated" / "utn57_mapping.rs",
    )
    args = parser.parse_args()
    mapping = generate_utn57_mapping(
        args.names_input, args.targets_input, args.mapping_input, args.output
    )
    print(
        f"generated {len(mapping.targets)} UTN57 targets and "
        f"{len(mapping.rules)} reviewed mapping rows -> {args.output}"
    )


if __name__ == "__main__":
    main()
