# Changelog

All notable changes to this project are documented here.

## [Unreleased]

### Changed

- Upgrade the normalization and shaping backend to `mongol-norm` 0.4.0 (`mng-canonical/5`). This crate's version is unchanged.
- Consume `mongol-norm`'s public written-unit stream, which now treats `Zwnj` as a structural joining break, drops redundant ZWJ/ZWNJ, merges repeated identical controls, and expands `Cr` in every position.
- Reverse conversion shapes a word that contains ZWNJ as one run so context looks through the break, and positions each side independently. Surviving `Zwnj` is not written back as `U+200C`: ZVVNMOD PUA codes already encode the unjoined presentation forms.

### Canonical encoding (intentional)

These outputs follow `mongol-norm` 0.4's selection policy; the ZVVNMOD inventory and this crate's mapping tables are unchanged.

- A lone `U:fina` (`U+E011`) encodes as bare `U+1824 U+180B`, not `ZWJ + OE + FVS2`, because the leading ZWJ is redundant with `U:isol`.
- A lone `G:medi O:fina` (`U+E09C`) encodes the same way as `G:init O:fina` (`U+E096`); the leading ZWJ that used to pin the medial reading is redundant with `G:init`. Inside a word the medial ligature is unchanged.
- Two consecutive MVS in reverse-direction input collapse to one `U+202F` suffix separator, matching the forward direction's one-boundary rule.

### Testing

- Add regressions for ZWNJ structural splitting, unjoined-vs-joined ZVVNMOD spellings, and positioned-unit roundtrips.
- Update ZWJ-spelling expectations that `mng-canonical/5` no longer invents.

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
