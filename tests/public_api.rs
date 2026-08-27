use zvvnmod_utn57::{convert_zvvnmod_to_utn57, O_INIT};

#[test]
#[ignore = "requires the currently configured UTN #57 normalization backend"]
fn backend_neutral_api_converts_singleton_o_init() {
    let input = char::from_u32(O_INIT.codepoint()).unwrap().to_string();

    let output = convert_zvvnmod_to_utn57(&input).unwrap();

    assert_eq!(output, "\u{1824}\u{180b}\u{200d}");
}
