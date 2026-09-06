//! Regression contract for the duplicate-free written-unit stream introduced by
//! `mongol-norm` 0.2.0.

use zvvnmod_utn57::{
    mongol_norm::{Locale, Shaper, WrittenUnit},
    shape_utn57_positioned_written_units, Utn57PositionedWrittenUnit, Utn57WrittenUnit,
};

fn units(records: &[Utn57PositionedWrittenUnit]) -> Vec<Utn57WrittenUnit> {
    records.iter().map(|record| record.written_unit).collect()
}

fn shape_normalized(raw: &[WrittenUnit]) -> Vec<Utn57WrittenUnit> {
    let shaper = Shaper::new(Locale::Mng);
    let text = shaper.normalize_written_units(raw).unwrap();
    units(&shape_utn57_positioned_written_units(&text).unwrap())
}

#[test]
fn conformant_composite_collisions_are_consumed_as_the_decomposed_stream() {
    use Utn57WrittenUnit::{A as Ua, B as Ub, O as Uo, S as Us};
    use WrittenUnit::{Dd, A, B, H, O, S};

    let cases: &[(&[WrittenUnit], &[Utn57WrittenUnit])] = &[
        (&[A, O, Dd, B, O], &[Ua, Uo, Uo, Ua, Ub, Uo]),
        (&[A, O, Dd], &[Ua, Uo, Uo, Ua]),
        (&[B, A, H, S, A], &[Ub, Ua, Ua, Ua, Us, Ua]),
    ];
    for (raw, expected) in cases {
        assert_eq!(shape_normalized(raw), *expected, "raw stream: {raw:?}");
    }
}

#[test]
fn final_a_aa_contracts_only_immediately_after_a_bowed_written_unit() {
    use Utn57WrittenUnit::{
        Aa as Uaa, Gx as Ugx, A as Ua, B as Ub, F as Uf, G as Ug, K as Uk, K2 as Uk2, N as Un,
        P as Up,
    };
    use WrittenUnit::{Aa, Gx, A, B, F, G, K, K2, N, P};

    for (bowed, expected) in [
        (B, Ub),
        (P, Up),
        (F, Uf),
        (G, Ug),
        (Gx, Ugx),
        (K, Uk),
        (K2, Uk2),
    ] {
        assert_eq!(shape_normalized(&[bowed, A, Aa]), [expected, Uaa]);
    }
    assert_eq!(shape_normalized(&[N, A, Aa]), [Un, Ua, Uaa]);
    assert_eq!(shape_normalized(&[B, A, A, Aa]), [Ub, Ua, Ua, Uaa]);
}
