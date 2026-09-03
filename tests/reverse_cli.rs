use std::process::Command;

fn run(arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_utn57-to-zvvnmod"))
        .args(arguments)
        .output()
        .unwrap()
}

#[test]
fn cli_rejects_zero_arguments() {
    let output = run(&[]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
}

#[test]
fn cli_rejects_extra_arguments() {
    let output = run(&["one", "two"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
}

#[test]
fn cli_converts_a_mongolian_argument() {
    // ᠰᠢᠨ᠎ᠡ
    let output = run(&["\u{1830}\u{1822}\u{1828}\u{180E}\u{1821}"]);

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, "\u{E03C}\u{E006}\u{E077}\n".as_bytes());
}

#[test]
fn cli_reports_a_unit_without_a_zvvnmod_glyph() {
    // ᠪᠠᠳᠠᠭ + FVS3
    let output = run(&["\u{182A}\u{1820}\u{1833}\u{1820}\u{182D}\u{180D}"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no ZVVNMOD glyph"));
}
