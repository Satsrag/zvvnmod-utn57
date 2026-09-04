//! Which reading a bowl and the tooth behind it carry.
//!
//! ZVVNMOD writes `Dd` — the devsger ᠳ that closes a syllable — as a bowl and a
//! tooth, the same two codes as an `O:medi` vowel followed by a tooth of its
//! own. `data/utn57-zvvnmod-map.csv` records both readings as conformant UTN,
//! with 17 witnesses for `O:medi + A:medi` and 51 for `O:medi + A:fina`, and
//! leaves the choice to this direction: "reverse conversion emits the shared
//! spelling either way; picking between them is the forward direction's
//! problem". Until now the forward direction never picked — the two-code `Dd`
//! rules outrank the one-code bowl by longest match, so every bowl before a
//! tooth read as `Dd` and its vowel was lost. Reported against `U+E083`
//! B_O_MEDI as Satsrag/meco-rust#26.
//!
//! `mongol-norm` decides it by rule III.2e: ᠳ is devsger only behind a vowel,
//! which is what makes it a coda. A bowl behind a consonant is that consonant's
//! own vowel.
//!
//! The expected letters are not round trips. Each is the canonical encoding of
//! the written units `mongol-norm` shapes the genuine word into, so a test fails
//! if this crate disagrees with the backend about what the word is.

use zvvnmod_utn57::{
    convert_zvvnmod_run, convert_zvvnmod_to_utn57, A_MEDI, B_MEDI, B_O_MEDI, O_MEDI,
};

/// Spell a string as its code points.
fn hex(text: &str) -> String {
    text.chars()
        .map(|character| format!("{:04X}", character as u32))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Convert ZVVNMOD code points to canonical UTN #57, spelled as code points.
fn convert(codes: &[u32]) -> String {
    let text: String = codes
        .iter()
        .map(|&code| char::from_u32(code).expect("ZVVNMOD codes are scalar values"))
        .collect();
    hex(&convert_zvvnmod_to_utn57(&text).unwrap())
}

#[test]
fn the_bowl_of_a_merged_code_stays_a_vowel_when_a_tooth_follows() {
    // The reported bisection. U+E083 is medial B together with its bowl vowel,
    // and alone it was already right.
    // B:medi O:medi.
    assert_eq!(convert(&[0xE083]), "200D 182A 1823 200D");
    // B:medi O:medi A:medi. The bowl was reading as Dd:medi, 1833 180C.
    assert_eq!(convert(&[0xE083, 0xE005]), "200D 182A 1823 1820 200D");
    // B:medi O:medi H:medi — the two teeth behind the bowl are a unit of their
    // own, and that one is behind a vowel, so it keeps its composite reading.
    assert_eq!(
        convert(&[0xE083, 0xE005, 0xE005]),
        "200D 182A 1823 182C 180D 200D"
    );
    // B:medi O:medi A:fina. The bowl was reading as Dd:fina.
    assert_eq!(convert(&[0xE083, 0xE00C]), "200D 182A 1823 1820 180C");
}

#[test]
fn the_reported_word_spells_the_written_units_its_hub_codes_carry() {
    // ᠠᠪᠤᠭᠰᠠᠨ as meco's hub text, units
    // A:init A:medi B:medi O:medi H:medi S:medi A:medi A:fina — what mongol-norm
    // shapes the genuine ᠠᠪᠤᠭᠰᠠᠨ into. The medial ᠭ before a consonant is written
    // as the tooth pair E005 E005 and returns as the H:medi that pair spells,
    // since ZVVNMOD cannot carry the two apart.
    assert_eq!(
        convert(&[0xE000, 0xE005, 0xE083, 0xE005, 0xE005, 0xE03D, 0xE005, 0xE00C]),
        "1820 180B 1820 182A 1823 182C 180D 1830 1820 1820 180C"
    );
}

#[test]
fn a_bowl_behind_a_consonant_is_that_consonants_vowel() {
    // No merged code is involved in any of these: the defect is the bowl, not
    // U+E083. Each hub text is what the reverse direction spells the word as.

    // ᠮᠣᠩᠭᠣᠯ — M:init O:medi A:medi G:medi Hx:medi O:medi L:fina. The bowl behind
    // M:init was eating the tooth of ᠩ as Dd:medi.
    assert_eq!(
        convert(&[0xE036, 0xE008, 0xE005, 0xE031, 0xE028, 0xE028, 0xE008, 0xE03B]),
        "182E 1823 1820 182D 180C 182C 180B 1823 182F"
    );
    // ᠬᠦᠮᠦᠨ — G:init O:medi I:medi M:medi O:medi A:fina. Named in the reviewed
    // data as a witness for the O:medi + A:fina reading.
    assert_eq!(
        convert(&[0xE09A, 0xE037, 0xE008, 0xE00C]),
        "182D 180C 1825 180D 1822 180D 182E 1823 1820 180C"
    );
    // ᠮᠣᠳᠣᠨ — M:init O:medi D:medi O:medi A:fina. The second bowl sits behind
    // D:medi, a consonant, so it is a vowel and not a second ᠳ.
    assert_eq!(
        convert(&[0xE036, 0xE008, 0xE046, 0xE008, 0xE00C]),
        "182E 1823 1832 180C 1823 1820 180C"
    );
}

#[test]
fn a_bowl_behind_a_vowel_still_closes_the_syllable() {
    // The other half of the contract: a genuine devsger ᠳ must survive. Both
    // words end in one, and in both the bowl that carries it sits behind a
    // vowel — which is what makes it a coda.

    // ᠨᠤᠭᠤᠳ — N:init O:medi Hx:medi O:medi Dd:fina.
    assert_eq!(
        convert(&[0xE027, 0xE008, 0xE028, 0xE028, 0xE008, 0xE008, 0xE00C]),
        "1828 1823 182C 180B 1823 1833"
    );
    // ᠤᠯᠤᠰᠤᠳ — A:init O:medi L:medi O:medi S:medi O:medi Dd:fina.
    assert_eq!(
        convert(&[0xE000, 0xE008, 0xE03A, 0xE008, 0xE03D, 0xE008, 0xE008, 0xE00C]),
        "1820 180B 1823 182F 1823 1830 1823 1833"
    );
}

#[test]
fn a_merged_code_leaves_no_trace_the_reading_depends_on() {
    // Decomposition is complete: the reverse direction recomposes merged glyphs
    // over the flat component stream, so B_O_MEDI + A_MEDI and
    // B_MEDI + O_MEDI + A_MEDI are one and the same hub text and have to read
    // back the same way. This is why the defect was never about U+E083.
    assert_eq!(
        convert_zvvnmod_run(&[B_O_MEDI, A_MEDI]).unwrap(),
        convert_zvvnmod_run(&[B_MEDI, O_MEDI, A_MEDI]).unwrap(),
    );
}
