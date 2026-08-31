use crate::zvvnmod_code;

/// Semantic class used by the full-text conversion facade.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZvvnmodTextCharacterKind {
    /// A member of the formal 139-code ZVVNMOD shape inventory.
    Shape,
    /// An excluded legacy ZVVNMOD FVS1-FVS4 or MVS control.
    LegacyControl,
    /// Text outside the formal ZVVNMOD shape inventory, preserved unchanged.
    Passthrough,
}

/// Classify one character for complete-text conversion containing ZVVNMOD shape runs.
///
/// The formal inventory includes ZVVNMOD's own Nirugu code. Standard Unicode
/// `U+180A MONGOLIAN NIRUGU`, `U+180E MONGOLIAN VOWEL SEPARATOR`, and `U+202F
/// NARROW NO-BREAK SPACE` are outside that inventory and pass through unchanged.
/// All other characters, including punctuation, numbers, and non-ZVVNMOD
/// private-use values, also pass through unchanged.
pub fn classify_zvvnmod_text_character(character: char) -> ZvvnmodTextCharacterKind {
    if matches!(character as u32, 0xE140..=0xE144) {
        return ZvvnmodTextCharacterKind::LegacyControl;
    }
    if zvvnmod_code(character).is_some() {
        return ZvvnmodTextCharacterKind::Shape;
    }
    ZvvnmodTextCharacterKind::Passthrough
}
