use crate::{zvvnmod_code, ZVVNMOD_SUFFIX_SEPARATOR};

/// Semantic class used by the full-text conversion facade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZvvnmodTextCharacterKind {
    /// A member of the formal 139-code ZVVNMOD shape inventory.
    Shape,
    /// An excluded legacy ZVVNMOD FVS1-FVS4 or MVS control.
    LegacyControl,
    /// The `U+202F` ZVVNMOD writes a detached-suffix boundary with.
    SuffixSeparator,
    /// Text outside the formal ZVVNMOD shape inventory, preserved unchanged.
    Passthrough,
}

/// Classify one character for complete-text conversion containing ZVVNMOD shape runs.
///
/// The formal inventory includes ZVVNMOD's own Nirugu code. `U+202F NARROW
/// NO-BREAK SPACE` is not a shape either, but it is not passthrough: ZVVNMOD
/// writes a detached-suffix boundary with it, and the conversion facade reads it
/// back as UTN #57 `MVS`. Standard Unicode `U+180A MONGOLIAN NIRUGU` and
/// `U+180E MONGOLIAN VOWEL SEPARATOR` are outside both roles and pass through
/// unchanged, as does the `U+0020` ZVVNMOD writes between words. All other
/// characters, including punctuation, numbers, and non-ZVVNMOD private-use
/// values, also pass through unchanged.
pub fn classify_zvvnmod_text_character(character: char) -> ZvvnmodTextCharacterKind {
    if matches!(character as u32, 0xE140..=0xE144) {
        return ZvvnmodTextCharacterKind::LegacyControl;
    }
    if character as u32 == ZVVNMOD_SUFFIX_SEPARATOR.0 {
        return ZvvnmodTextCharacterKind::SuffixSeparator;
    }
    if zvvnmod_code(character).is_some() {
        return ZvvnmodTextCharacterKind::Shape;
    }
    ZvvnmodTextCharacterKind::Passthrough
}
