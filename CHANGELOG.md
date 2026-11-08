# Changelog

All notable changes to this project are documented here.

## [Unreleased]

### Changed

- Upgrade the normalization and shaping backend from `mongol-norm` 0.2.1 to `mongol-norm` 0.4.0 (`mng-canonical/5`).
- Handle `WrittenUnit::Zwnj` (`U+200C ZERO WIDTH NON-JOINER`) as a structural non-joining break: `convert_utn57_to_zvvnmod` shapes each Mongolian word as a whole first (so contextual rules such as vowel harmony look through ZWNJ, repeated `ZWJ`/`ZWNJ`/`MVS` controls merge, and redundant `ZWJ`/`ZWNJ` units are dropped), then splits the shaped sequence at surviving `Zwnj` breaks, converts and recomposes each segment independently, and emits `U+200C` between segments. On the forward path, `U+200C` passes through and splits adjacent ZVVNMOD runs so multi-code rules (such as `L_FINA AA_FINA → L:fina MVS Aa:isol` on `ᠠᠯ‌ᠠ᠌`) never match across a ZWNJ break.
- Consecutive `MVS` controls in UTN #57 input now collapse into a single `WrittenUnit::Mvs` during shaping under `mongol-norm` 0.4.0, so `convert_utn57_to_zvvnmod` emits one `U+202F` boundary just as `convert_zvvnmod_to_utn57` emits one `MVS` for repeated `U+202F`.
- Expand `Cr` (`U+1842`) to `O + O` in every position (`isol`, `init`, `medi`, `fina`).
- Under `mng-canonical/5`'s redundant-ZWJ rules, edge joiners whose joined and unjoined forms are reviewed as equivalent (such as `G_O_FINA` `U+E09C`, `G_O_MEDI` `U+E09B`, `U_FINA` `U+E011`, `I_FINA` `U+E00E`, and lone medial consonants like `B_MEDI` `U+E02A`) are dropped during normalization and no longer trigger `Utn57ConversionWarning::InventedZwj`; runs whose joined form differs from the unjoined form (such as `A_INIT` `U+E000`, `O_INIT` `U+E001`, `I_INIT` `U+E04D`, `A_MEDI` `U+E005`, `O_FINA` `U+E00F`, and `B_FINA` `U+E02B`) retain their invented ZWJ and warning.

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
