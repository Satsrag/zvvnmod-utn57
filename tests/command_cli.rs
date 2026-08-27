use std::process::Command;
use zvvnmod_utn57::O_INIT;

#[test]
fn cli_rejects_zero_arguments() {
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-to-utn57"))
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
}

#[test]
fn cli_rejects_extra_arguments() {
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-to-utn57"))
        .args(["one", "two"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
}

#[test]
#[ignore = "requires mongol-norm 0.0.4 installed by zvvnmod-install-mongol-norm"]
fn cli_converts_zvvnmod_argument_through_mongol_norm_command() {
    let input = char::from_u32(O_INIT.codepoint()).unwrap().to_string();
    let output = Command::new(env!("CARGO_BIN_EXE_zvvnmod-to-utn57"))
        .arg(input)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, "\u{1824}\u{180b}\u{200d}\n".as_bytes());
}
