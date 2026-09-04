use crate::zvvnmod_code;

/// The `U+202F` version 0.1.2 spelled a detached-suffix boundary with.
///
/// ZVVNMOD writes the boundary `U+0020`
/// ([`ZVVNMOD_SUFFIX_SEPARATOR`](crate::ZVVNMOD_SUFFIX_SEPARATOR)), so this is no
/// longer emitted. It stays classified as a boundary so that text 0.1.2 already
/// wrote still reads its `MVS` back.
const LEGACY_SUFFIX_SEPARATOR: char = '\u{202F}';

/// Semantic class used by the full-text conversion facade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZvvnmodTextCharacterKind {
    /// A member of the formal 139-code ZVVNMOD shape inventory.
    Shape,
    /// An excluded legacy ZVVNMOD FVS1-FVS4 or MVS control.
    LegacyControl,
    /// A character to read back as a UTN #57 detached-suffix boundary.
    SuffixSeparator,
    /// Text outside the formal ZVVNMOD shape inventory, preserved unchanged.
    Passthrough,
}

/// Classify one character for complete-text conversion containing ZVVNMOD shape runs.
///
/// The formal inventory includes ZVVNMOD's own Nirugu code. Standard Unicode
/// `U+180A MONGOLIAN NIRUGU` is outside both roles and passes through unchanged,
/// as do punctuation, numbers, and non-ZVVNMOD private-use values.
///
/// `U+0020` passes through as itself, and so this crate's own hub spelling of a
/// detached-suffix boundary is *not* reported as
/// [`SuffixSeparator`](ZvvnmodTextCharacterKind::SuffixSeparator). The asymmetry
/// with [`ZVVNMOD_SUFFIX_SEPARATOR`](crate::ZVVNMOD_SUFFIX_SEPARATOR) is the hub's
/// own: ZVVNMOD spells a suffix boundary and a word space alike, marking the
/// difference in the glyph that follows, so a classifier reading one character at
/// a time cannot tell them apart — and reading every space back as a `MVS` would
/// weld every pair of words together.
///
/// What is reported as a boundary is the pair of spellings this crate itself
/// emitted before 0.1.3 — `U+202F` from 0.1.2, and `U+180E MONGOLIAN VOWEL
/// SEPARATOR` from 0.1.1, which reaches the same `MVS` as passthrough — so that
/// stored text keeps its boundary.
pub fn classify_zvvnmod_text_character(character: char) -> ZvvnmodTextCharacterKind {
    if matches!(character as u32, 0xE140..=0xE144) {
        return ZvvnmodTextCharacterKind::LegacyControl;
    }
    if character == LEGACY_SUFFIX_SEPARATOR {
        return ZvvnmodTextCharacterKind::SuffixSeparator;
    }
    if zvvnmod_code(character).is_some() {
        return ZvvnmodTextCharacterKind::Shape;
    }
    ZvvnmodTextCharacterKind::Passthrough
}
