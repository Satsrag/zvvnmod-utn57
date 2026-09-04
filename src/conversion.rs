use crate::{
    discard_legacy_controls, replace_ir_fina, IrFinaReplacementError, Utn57Position,
    Utn57PositionedWrittenUnit, Utn57WrittenUnit, ZvvnmodCode, ZvvnmodToUtn57Mapping,
    ZVVNMOD_CODE_DECOMPOSITIONS, ZVVNMOD_TO_UTN57_MAPPINGS,
};

/// A ZVVNMOD code that has no reviewed UTN #57 mapping at an input index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utn57MappingError {
    /// Index in the input run after preprocessing and decomposition.
    pub index: usize,
    /// Unmapped ZVVNMOD code.
    pub code: ZvvnmodCode,
}

impl std::fmt::Display for Utn57MappingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "unmapped ZVVNMOD code U+{:04X} at index {}",
            self.code.codepoint(),
            self.index
        )
    }
}

impl std::error::Error for Utn57MappingError {}

/// Equal-longest mapping candidates that no registered policy could resolve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Utn57AmbiguityError {
    /// Index in the normalized run where the ambiguous sequence starts.
    pub index: usize,
    /// Equal-longest reviewed source sequence.
    pub sources: &'static [ZvvnmodCode],
    /// Sorted stable row IDs of the unresolved candidates.
    pub candidate_ids: Vec<&'static str>,
}

impl std::fmt::Display for Utn57AmbiguityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "unresolved UTN #57 mapping ambiguity at index {} for {} source code(s): {}",
            self.index,
            self.sources.len(),
            self.candidate_ids.join(", ")
        )
    }
}

impl std::error::Error for Utn57AmbiguityError {}

/// Failure in an ordered ZVVNMOD → UTN #57 conversion stage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Utn57ConversionError {
    /// `Ir_fina` could not replace its preceding form.
    IrFina(IrFinaReplacementError),
    /// A preprocessed/decomposed code had no reviewed mapping.
    Mapping(Utn57MappingError),
    /// Equal-longest candidates could not be uniquely resolved.
    UnresolvedAmbiguity(Utn57AmbiguityError),
}

impl std::fmt::Display for Utn57ConversionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IrFina(error) => error.fmt(formatter),
            Self::Mapping(error) => error.fmt(formatter),
            Self::UnresolvedAmbiguity(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for Utn57ConversionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IrFina(error) => Some(error),
            Self::Mapping(error) => Some(error),
            Self::UnresolvedAmbiguity(error) => Some(error),
        }
    }
}

impl From<IrFinaReplacementError> for Utn57ConversionError {
    fn from(error: IrFinaReplacementError) -> Self {
        Self::IrFina(error)
    }
}

impl From<Utn57MappingError> for Utn57ConversionError {
    fn from(error: Utn57MappingError) -> Self {
        Self::Mapping(error)
    }
}

impl From<Utn57AmbiguityError> for Utn57ConversionError {
    fn from(error: Utn57AmbiguityError) -> Self {
        Self::UnresolvedAmbiguity(error)
    }
}

/// Resolution for the UTN #57 K/K2 units that share ZVVNMOD shapes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Utn57KVariant {
    /// Emit K, the canonical default.
    #[default]
    K,
    /// Emit K2 when the caller has nominal/context information requiring it.
    K2,
}

/// Options for reviewed ZVVNMOD → UTN #57 mapping replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Utn57ConversionOptions {
    /// Resolve the shared K/K2 ZVVNMOD shapes.
    pub k_variant: Utn57KVariant,
}

fn position_connections(position: Utn57Position) -> Option<(bool, bool)> {
    match position {
        Utn57Position::Isol => Some((false, false)),
        Utn57Position::Init => Some((false, true)),
        Utn57Position::Medi => Some((true, true)),
        Utn57Position::Fina => Some((true, false)),
        Utn57Position::Control => None,
    }
}

fn position_from_connections(left: bool, right: bool) -> Utn57Position {
    match (left, right) {
        (false, false) => Utn57Position::Isol,
        (false, true) => Utn57Position::Init,
        (true, true) => Utn57Position::Medi,
        (true, false) => Utn57Position::Fina,
    }
}

fn target_sequence_position(targets: &[Utn57PositionedWrittenUnit]) -> Option<Utn57Position> {
    let mut positions = targets
        .iter()
        .filter_map(|target| position_connections(target.position));
    let first = positions.next()?;
    let last = match positions.next_back() {
        Some(last) => last,
        None => first,
    };
    Some(position_from_connections(first.0, last.1))
}

fn match_position(start: usize, end: usize, run_len: usize) -> Utn57Position {
    position_from_connections(start > 0, end < run_len)
}

fn unique_candidate_for_position<'a>(
    candidates: &[&'a ZvvnmodToUtn57Mapping],
    position: Utn57Position,
) -> Result<Option<&'a ZvvnmodToUtn57Mapping>, ()> {
    let mut matches = candidates
        .iter()
        .copied()
        .filter(|rule| target_sequence_position(rule.targets) == Some(position));
    let first = matches.next();
    if matches.next().is_some() {
        Err(())
    } else {
        Ok(first)
    }
}

fn resolve_by_position<'a>(
    candidates: &[&'a ZvvnmodToUtn57Mapping],
    actual_position: Utn57Position,
) -> Option<&'a ZvvnmodToUtn57Mapping> {
    let intrinsic_position = candidates.first()?.intrinsic_position?;
    if candidates
        .iter()
        .any(|rule| rule.intrinsic_position != Some(intrinsic_position))
    {
        return None;
    }
    let fallback = unique_candidate_for_position(candidates, intrinsic_position).ok()??;
    match unique_candidate_for_position(candidates, actual_position) {
        Ok(Some(actual)) => Some(actual),
        Ok(None) => Some(fallback),
        Err(()) => None,
    }
}

fn resolve_k_variant<'a>(
    candidates: &[&'a ZvvnmodToUtn57Mapping],
    variant: Utn57KVariant,
) -> Option<&'a ZvvnmodToUtn57Mapping> {
    let mut k = None;
    let mut k2 = None;
    for &rule in candidates {
        let [target] = rule.targets else {
            return None;
        };
        let slot = match target.written_unit {
            Utn57WrittenUnit::K => &mut k,
            Utn57WrittenUnit::K2 => &mut k2,
            _ => return None,
        };
        if slot.replace(rule).is_some() {
            return None;
        }
    }
    match variant {
        Utn57KVariant::K if k2.is_some() => k,
        Utn57KVariant::K2 if k.is_some() => k2,
        _ => None,
    }
}

fn resolve_candidates<'a>(
    candidates: &[&'a ZvvnmodToUtn57Mapping],
    actual_position: Utn57Position,
    options: Utn57ConversionOptions,
    index: usize,
) -> Result<&'a ZvvnmodToUtn57Mapping, Utn57AmbiguityError> {
    let first = candidates
        .first()
        .copied()
        .expect("equal-longest candidate collection cannot be empty");
    if candidates.len() == 1 || candidates.iter().all(|rule| rule.targets == first.targets) {
        return Ok(first);
    }
    if let Some(rule) = resolve_k_variant(candidates, options.k_variant) {
        return Ok(rule);
    }
    if let Some(rule) = resolve_by_position(candidates, actual_position) {
        return Ok(rule);
    }
    let mut candidate_ids: Vec<_> = candidates.iter().map(|rule| rule.id).collect();
    candidate_ids.sort_unstable();
    Err(Utn57AmbiguityError {
        index,
        sources: first.sources,
        candidate_ids,
    })
}

/// Is this written unit a vowel?
///
/// An exhaustive `match` rather than a lookup, for the reason
/// [`crate::normalize`] gives for its own: `utn57_mapping.rs` is generated, so a
/// unit the generator adds has to fail this crate's build rather than default
/// to a consonant and change how a bowl beside it reads.
const fn is_vowel(unit: Utn57WrittenUnit) -> bool {
    match unit {
        Utn57WrittenUnit::A
        | Utn57WrittenUnit::Aa
        | Utn57WrittenUnit::I
        | Utn57WrittenUnit::Ix
        | Utn57WrittenUnit::O
        | Utn57WrittenUnit::U
        | Utn57WrittenUnit::Ue
        | Utn57WrittenUnit::Ux => true,
        Utn57WrittenUnit::B
        | Utn57WrittenUnit::B2
        | Utn57WrittenUnit::C
        | Utn57WrittenUnit::Ch
        | Utn57WrittenUnit::Cr
        | Utn57WrittenUnit::D
        | Utn57WrittenUnit::Dd
        | Utn57WrittenUnit::F
        | Utn57WrittenUnit::G
        | Utn57WrittenUnit::Gx
        | Utn57WrittenUnit::H
        | Utn57WrittenUnit::Hr
        | Utn57WrittenUnit::Hx
        | Utn57WrittenUnit::J
        | Utn57WrittenUnit::K
        | Utn57WrittenUnit::K2
        | Utn57WrittenUnit::L
        | Utn57WrittenUnit::M
        | Utn57WrittenUnit::N
        | Utn57WrittenUnit::P
        | Utn57WrittenUnit::R
        | Utn57WrittenUnit::Rh
        | Utn57WrittenUnit::S
        | Utn57WrittenUnit::Sh
        | Utn57WrittenUnit::Sz
        | Utn57WrittenUnit::T
        | Utn57WrittenUnit::W
        | Utn57WrittenUnit::Y
        | Utn57WrittenUnit::Z
        | Utn57WrittenUnit::Zr
        | Utn57WrittenUnit::Nirugu
        | Utn57WrittenUnit::MVS => false,
    }
}

/// Does every `Dd` this rule spells sit behind a vowel?
///
/// ZVVNMOD writes `Dd` — the devsger ᠳ that closes a syllable — as a bowl and a
/// tooth, the same two codes as an `O:medi` vowel followed by a tooth of its
/// own. `data/utn57-zvvnmod-map.csv` records both readings as conformant UTN,
/// with 17 witnesses for `O:medi + A:medi` and 51 for `O:medi + A:fina`, and
/// leaves the choice here: "reverse conversion emits the shared spelling either
/// way; picking between them is the forward direction's problem". Longest match
/// alone never picks — the two-code `Dd` rules simply outrank the one-code bowl,
/// so every bowl before a tooth read as `Dd` and its vowel was lost.
///
/// `mongol-norm` decides it by rule III.2e: ᠳ is devsger only behind a vowel,
/// which is what makes it a coda. So a bowl behind a consonant is that
/// consonant's own vowel — ᠮᠣᠩᠭᠣᠯ, ᠬᠦᠮᠦᠨ, ᠮᠣᠳᠣᠨ — and only a bowl behind a vowel
/// closes a syllable — ᠨᠤᠭᠤᠳ, ᠤᠯᠤᠰᠤᠳ.
///
/// `preceding` is the last unit already emitted, and a `Dd` a rule spells after
/// a unit of its own is judged against that one, so the reviewed particle rows
/// that carry their own vowel ahead of a `Dd` need no exception.
fn devsger_readings_hold(
    rule: &ZvvnmodToUtn57Mapping,
    preceding: Option<Utn57PositionedWrittenUnit>,
) -> bool {
    rule.targets
        .iter()
        .enumerate()
        .filter(|(_, target)| target.written_unit == Utn57WrittenUnit::Dd)
        .all(|(offset, _)| {
            offset
                .checked_sub(1)
                .map(|previous| rule.targets[previous])
                .or(preceding)
                .is_some_and(|unit| is_vowel(unit.written_unit))
        })
}

/// Convert one ZVVNMOD written-form run to reviewed UTN #57 units.
///
/// The function discards legacy controls, applies `Ir_fina` replacement,
/// decomposes general merged codes, and then applies reviewed longest-match
/// mapping among the rules whose reading holds where they matched. Equal-longest
/// candidates use positional fallback/override and registered semantic
/// resolution; unresolved families return a typed error. Reviewed MVS targets
/// are preserved. The function does not yet reconstruct ZWJ or particle
/// boundaries.
pub fn convert_zvvnmod_run(
    input: &[ZvvnmodCode],
) -> Result<Vec<Utn57PositionedWrittenUnit>, Utn57ConversionError> {
    convert_zvvnmod_run_with_options(input, Utn57ConversionOptions::default())
}

/// Convert a run while applying explicit K/K2 ambiguity options.
pub fn convert_zvvnmod_run_with_options(
    input: &[ZvvnmodCode],
    options: Utn57ConversionOptions,
) -> Result<Vec<Utn57PositionedWrittenUnit>, Utn57ConversionError> {
    let cleaned = discard_legacy_controls(input);
    let replaced = replace_ir_fina(&cleaned)?;
    let mut decomposed = Vec::with_capacity(replaced.len());
    for &code in &replaced {
        if let Some((_, components)) = ZVVNMOD_CODE_DECOMPOSITIONS
            .iter()
            .find(|(merged, _)| *merged == code)
        {
            decomposed.extend_from_slice(components);
        } else {
            decomposed.push(code);
        }
    }
    let input = decomposed.as_slice();
    let mut output = Vec::new();
    let mut index = 0;
    while index < input.len() {
        // Every ZVVNMOD code has a single-code rule, and a single-code rule
        // spells no `Dd`, so a rule is always left to match and the loop always
        // advances.
        let applies = |rule: &ZvvnmodToUtn57Mapping| {
            input[index..].starts_with(rule.sources)
                && devsger_readings_hold(rule, output.last().copied())
        };
        let longest = ZVVNMOD_TO_UTN57_MAPPINGS
            .iter()
            .filter(|rule| applies(rule))
            .map(|rule| rule.sources.len())
            .max()
            .ok_or(Utn57MappingError {
                index,
                code: input[index],
            })?;
        let candidates: Vec<_> = ZVVNMOD_TO_UTN57_MAPPINGS
            .iter()
            .filter(|rule| rule.sources.len() == longest && applies(rule))
            .collect();
        let rule = resolve_candidates(
            &candidates,
            match_position(index, index + longest, input.len()),
            options,
            index,
        )?;
        output.extend_from_slice(rule.targets);
        index += rule.sources.len();
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        A_FINA, B_INIT, K_INIT, O_MEDI, UTN57_A_FINA, UTN57_B_INIT, UTN57_C_INIT, UTN57_K2_FINA,
        UTN57_K_INIT, UTN57_MVS_CONTROL, UTN57_O_INIT, UTN57_O_MEDI,
    };

    static C_SOURCES: &[ZvvnmodCode] = &[O_MEDI, A_FINA];
    static ISOL_TARGETS: &[Utn57PositionedWrittenUnit] = &[UTN57_O_INIT, UTN57_A_FINA];
    static FINA_TARGETS: &[Utn57PositionedWrittenUnit] = &[UTN57_O_MEDI, UTN57_A_FINA];
    static C_ISOL: ZvvnmodToUtn57Mapping = ZvvnmodToUtn57Mapping {
        id: "test:c-isol",
        sources: C_SOURCES,
        targets: ISOL_TARGETS,
        intrinsic_position: Some(Utn57Position::Fina),
    };
    static C_FINA: ZvvnmodToUtn57Mapping = ZvvnmodToUtn57Mapping {
        id: "test:c-fina",
        sources: C_SOURCES,
        targets: FINA_TARGETS,
        intrinsic_position: Some(Utn57Position::Fina),
    };

    #[test]
    fn k_variant_semantics_are_resolved_before_position() {
        static SOURCES: &[ZvvnmodCode] = &[K_INIT];
        static K_TARGETS: &[Utn57PositionedWrittenUnit] = &[UTN57_K_INIT];
        static K2_TARGETS: &[Utn57PositionedWrittenUnit] = &[UTN57_K2_FINA];
        static K: ZvvnmodToUtn57Mapping = ZvvnmodToUtn57Mapping {
            id: "test:k-init",
            sources: SOURCES,
            targets: K_TARGETS,
            intrinsic_position: Some(Utn57Position::Fina),
        };
        static K2: ZvvnmodToUtn57Mapping = ZvvnmodToUtn57Mapping {
            id: "test:k2-fina",
            sources: SOURCES,
            targets: K2_TARGETS,
            intrinsic_position: Some(Utn57Position::Fina),
        };
        let selected = resolve_candidates(
            &[&K, &K2],
            Utn57Position::Init,
            Utn57ConversionOptions {
                k_variant: Utn57KVariant::K2,
            },
            0,
        )
        .unwrap();
        assert_eq!(selected.id, "test:k2-fina");
    }

    #[test]
    fn actual_position_overrides_the_intrinsic_fallback_otherwise_fallback_is_used() {
        let candidates = [&C_ISOL, &C_FINA];
        let options = Utn57ConversionOptions::default();
        assert_eq!(
            resolve_candidates(&candidates, Utn57Position::Isol, options, 0)
                .unwrap()
                .id,
            "test:c-isol",
        );
        for actual in [
            Utn57Position::Init,
            Utn57Position::Medi,
            Utn57Position::Fina,
        ] {
            assert_eq!(
                resolve_candidates(&candidates, actual, options, 0)
                    .unwrap()
                    .id,
                "test:c-fina",
            );
        }
    }

    #[test]
    fn positional_candidate_selection_does_not_depend_on_relation_order() {
        let candidates = [&C_FINA, &C_ISOL];
        assert_eq!(
            resolve_candidates(
                &candidates,
                Utn57Position::Isol,
                Utn57ConversionOptions::default(),
                0,
            )
            .unwrap()
            .id,
            "test:c-isol",
        );
    }

    #[test]
    fn controls_do_not_bear_target_sequence_position() {
        assert_eq!(
            target_sequence_position(&[UTN57_O_INIT, UTN57_MVS_CONTROL, UTN57_A_FINA]),
            Some(Utn57Position::Isol),
        );
        assert_eq!(
            target_sequence_position(&[UTN57_O_MEDI, UTN57_MVS_CONTROL, UTN57_A_FINA]),
            Some(Utn57Position::Fina),
        );
    }

    #[test]
    fn unregistered_same_position_candidates_fail_closed() {
        static SOURCES: &[ZvvnmodCode] = &[B_INIT];
        static B_TARGETS: &[Utn57PositionedWrittenUnit] = &[UTN57_B_INIT];
        static C_TARGETS: &[Utn57PositionedWrittenUnit] = &[UTN57_C_INIT];
        static B: ZvvnmodToUtn57Mapping = ZvvnmodToUtn57Mapping {
            id: "test:unknown-b",
            sources: SOURCES,
            targets: B_TARGETS,
            intrinsic_position: Some(Utn57Position::Fina),
        };
        static C: ZvvnmodToUtn57Mapping = ZvvnmodToUtn57Mapping {
            id: "test:unknown-c",
            sources: SOURCES,
            targets: C_TARGETS,
            intrinsic_position: Some(Utn57Position::Fina),
        };
        let error = resolve_candidates(
            &[&B, &C],
            Utn57Position::Init,
            Utn57ConversionOptions::default(),
            7,
        )
        .unwrap_err();
        assert_eq!(
            error,
            Utn57AmbiguityError {
                index: 7,
                sources: SOURCES,
                candidate_ids: vec!["test:unknown-b", "test:unknown-c"],
            },
        );
    }
}
