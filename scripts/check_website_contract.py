#!/usr/bin/env python3
"""Verify that the merged website CSV artifacts are direct Rust generator inputs."""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import tempfile
from pathlib import Path

from generate_zvvnmod import generate_utn57_mapping, read_utn57_mapping_csv, read_utn57_targets_csv, build_model, read_csv

ROOT = Path(__file__).resolve().parents[1]
WEBSITE_MAP = "mapping/data/zvvnmod-utn57-map.csv"
WEBSITE_TARGETS = "mapping/data/utn57-written-units.csv"
EXPECTED_WEBSITE_REVISION = "0764c208e9b778339c95ea7f124afc3f2816bc01"


def resolve_revision(repository: Path, revision: str) -> str:
    result = subprocess.run(
        ["git", "rev-parse", f"{revision}^{{commit}}"],
        cwd=repository,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr)
    return result.stdout.strip()


def git_blob(repository: Path, revision: str, path: str) -> bytes:
    result = subprocess.run(
        ["git", "show", f"{revision}:{path}"],
        cwd=repository,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr.decode("utf-8", errors="replace"))
    return result.stdout


def check(website_root: Path, website_ref: str) -> None:
    revision = resolve_revision(website_root, website_ref)
    if revision != EXPECTED_WEBSITE_REVISION:
        raise ValueError(
            f"website revision differs from contract: expected {EXPECTED_WEBSITE_REVISION}, got {revision}"
        )
    producer_map = git_blob(website_root, revision, WEBSITE_MAP)
    producer_targets = git_blob(website_root, revision, WEBSITE_TARGETS)
    consumer_map = ROOT / "data" / "zvvnmod-utn57-map.csv"
    consumer_targets = ROOT / "data" / "utn57-written-units.csv"
    if producer_map != consumer_map.read_bytes():
        raise ValueError("website and Rust runtime mapping CSV bytes differ")
    if producer_targets != consumer_targets.read_bytes():
        raise ValueError("website and Rust target catalogue CSV bytes differ")

    with tempfile.TemporaryDirectory() as directory:
        temporary = Path(directory)
        copied_map = temporary / "zvvnmod-utn57-map.csv"
        copied_targets = temporary / "utn57-written-units.csv"
        generated = temporary / "utn57_mapping.rs"
        copied_map.write_bytes(producer_map)
        copied_targets.write_bytes(producer_targets)
        generate_utn57_mapping(
            ROOT / "data" / "zvvnmod-unicode-names.csv",
            copied_targets,
            copied_map,
            generated,
        )
        if generated.read_bytes() != (ROOT / "src" / "generated" / "utn57_mapping.rs").read_bytes():
            raise ValueError("website CSVs do not reproduce the checked-in Rust relation")
        model = build_model(read_csv(ROOT / "data" / "zvvnmod-unicode-names.csv"))
        targets = read_utn57_targets_csv(copied_targets)
        mapping = read_utn57_mapping_csv(copied_map, model, targets)
        if len(targets) != 97 or len(mapping.rules) != 147:
            raise ValueError("website CSV contract has unexpected target or relation counts")

    print(
        "website CSV contract passed: "
        f"map sha256={hashlib.sha256(producer_map).hexdigest()}, "
        f"targets sha256={hashlib.sha256(producer_targets).hexdigest()}, "
        "97 targets, 147 relations"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--website-root", type=Path, required=True)
    parser.add_argument("--website-ref", default=EXPECTED_WEBSITE_REVISION)
    args = parser.parse_args()
    check(args.website_root.resolve(), args.website_ref)


if __name__ == "__main__":
    main()
