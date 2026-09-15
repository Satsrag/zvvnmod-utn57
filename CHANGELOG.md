# Changelog

All notable changes to this project are documented here.

## [0.3.2] - 2026-09-15

### Fixed

- A lone initial A or I (`U+E000`, `U+E04D`) converts to UTN #57 with a trailing ZWJ and reads back as the same glyph; written bare it read back as the isolated form (`A:isol` is `U+E000 U+E00D`, `I:isol` is `U+E01A`). The spelling comes from `mongol-norm` 0.2.1, now the minimum version, which gives a lone `A:init` / `I:init` the trailing ZWJ it already gave `O:init`; a lone initial consonant stays bare, since its isolated form is the initial unit (Satsrag/meco-rust#45).

## [0.3.1] - 2026-09-12

### Fixed

- Two or more `U+202F` in a row mark one detached-suffix boundary and are written as one `MVS`; `convert_zvvnmod_to_utn57_with_warnings` reports the repetition as `Utn57ConversionWarning::CollapsedSuffixSeparators`. Previously every separator became an `MVS`, which UTN #57 does not allow (Satsrag/meco-rust#40).

## [0.3.0] - 2026-09-12

### Added

- `U+E096` (`G i O f`, `G_O_ISOL`): the word-initial G + O-final ligature the font-derived inventory lacked. It decomposes to `G_INIT O_FINA`, converts to `ᠭ᠌ᠥ᠌` without an invented ZWJ, and the reverse direction recomposes `G:init O:fina` into it (Satsrag/meco-rust#32).
- `convert_zvvnmod_to_utn57_with_warnings`, returning the converted text together with a `Utn57ConversionWarning::InventedZwj` for every run the normalizer could only spell by inventing a ZWJ — the signature of a hub inventory gap. `convert_zvvnmod_to_utn57` is unchanged. The `zvvnmod-to-utn57` binary prints each warning to stderr.

### Changed

- The formal ZVVNMOD shape inventory is 140 codes; the decomposition map has 60 entries.

## [0.2.0] - 2026-09-06

### Changed

- Upgrade the normalization and shaping backend to `mongol-norm` 0.2.0.
- Consume `mongol-norm`'s public duplicate-free written-unit stream when shaping Mongolian text for ZVVNMOD output.
- Expand the three conformant composite collisions (`Dd:medi`, `Dd:fina`, and `H:medi`) to their canonical two-unit readings.
- Contract final `A + Aa` only when an immediately preceding bowed written unit licenses the shared glyph; intervening, structural, and non-bowed contexts remain decomposed.
- Preserve the reverse ZVVNMOD spelling contract: duplicate unit streams still recompose to the same shared ZVVNMOD glyphs.

### Testing

- Add regressions for all three conformant collisions and the bowed/non-bowed `A + Aa` boundary.
- Keep the generated-source, reverse-row, Rust, Python, documentation, package, and wasm gates release-blocking.

[0.3.2]: https://github.com/Satsrag/zvvnmod-utn57/releases/tag/v0.3.2
[0.3.1]: https://github.com/Satsrag/zvvnmod-utn57/releases/tag/v0.3.1
[0.3.0]: https://github.com/Satsrag/zvvnmod-utn57/releases/tag/v0.3.0
[0.2.0]: https://github.com/Satsrag/zvvnmod-utn57/releases/tag/v0.2.0
