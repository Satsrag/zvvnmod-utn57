# Changelog

All notable changes to this project are documented here.

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

[0.2.0]: https://github.com/Satsrag/zvvnmod-utn57/releases/tag/v0.2.0
