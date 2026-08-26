#!/usr/bin/env python3
"""stdin JSON bridge from positioned UTN #57 records to Mongolian Unicode."""

import importlib.metadata
import json
from pathlib import Path
import sys

EXPECTED_VERSION = "0.0.4"
PROTOCOL_VERSION = 1


def main():
    if len(sys.argv) != 2:
        raise ValueError("exactly one mongol-norm install directory is required")
    install_path = Path(sys.argv[1]).resolve(strict=True)
    if not install_path.is_dir():
        raise ValueError("mongol-norm install path is not a directory")

    # Python is invoked with -I -S, so its initial search path contains only
    # standard-library locations. Add exactly the selected hash-pinned install.
    sys.path.append(str(install_path))
    import mongol_norm  # pyright: ignore[reportMissingImports]

    module_version = mongol_norm.__version__
    metadata_version = importlib.metadata.version("mongol-norm")
    if module_version != EXPECTED_VERSION:
        raise RuntimeError(
            "mongol_norm.__version__ is {}; exactly {} is required".format(
                module_version, EXPECTED_VERSION
            )
        )
    if metadata_version != EXPECTED_VERSION:
        raise RuntimeError(
            "mongol-norm distribution metadata version is {}; exactly {} is required".format(
                metadata_version, EXPECTED_VERSION
            )
        )

    if mongol_norm.__file__ is None:
        raise RuntimeError("mongol_norm module has no filesystem origin")
    module_origin = Path(mongol_norm.__file__).resolve(strict=True)
    distribution = importlib.metadata.distribution("mongol-norm")
    distribution_origin = Path(str(distribution.locate_file(""))).resolve(strict=True)
    for label, origin in (
        ("mongol_norm module", module_origin),
        ("mongol-norm distribution", distribution_origin),
    ):
        try:
            origin.relative_to(install_path)
        except ValueError:
            raise RuntimeError(
                "{} originated outside selected install directory: {}".format(label, origin)
            )

    payload = json.load(sys.stdin)
    if not isinstance(payload, dict) or payload.get("protocol") != PROTOCOL_VERSION:
        raise ValueError("unsupported positioned-unit bridge protocol")
    records = payload.get("records")
    if not isinstance(records, list):
        raise ValueError("records must be a list")

    output = mongol_norm.MongolianShaper("MNG").normalize_positioned_written_units(records)
    sys.stdout.write(output)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        sys.stderr.write("{}: {}\n".format(type(error).__name__, error))
        raise SystemExit(1)
