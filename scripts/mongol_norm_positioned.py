#!/usr/bin/env python3
"""stdin JSON bridge from positioned UTN #57 records to Mongolian Unicode."""

import importlib.metadata
import json
from pathlib import Path
import sys

EXPECTED_VERSION = "0.0.4"


def write_normalized_runs(shaper, runs):
    if not isinstance(runs, list):
        raise ValueError("positioned_written_unit_runs must be a list")
    output = sys.stdout.buffer
    for records in runs:
        if not isinstance(records, list):
            raise ValueError("each positioned written unit run must be a list")
        normalized = shaper.normalize_positioned_written_units(records).encode("utf-8")
        output.write(str(len(normalized)).encode("ascii"))
        output.write(b"\n")
        output.write(normalized)


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
    if not isinstance(payload, dict) or set(payload) != {"positioned_written_unit_runs"}:
        raise ValueError("payload requires exactly positioned_written_unit_runs")
    shaper = mongol_norm.MongolianShaper("MNG")
    write_normalized_runs(shaper, payload.get("positioned_written_unit_runs"))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        sys.stderr.write("{}: {}\n".format(type(error).__name__, error))
        raise SystemExit(1)
