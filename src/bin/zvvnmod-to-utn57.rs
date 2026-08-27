use std::process::ExitCode;
use zvvnmod_utn57::convert_zvvnmod_to_utn57;

fn main() -> ExitCode {
    let mut arguments = std::env::args();
    let program = arguments
        .next()
        .unwrap_or_else(|| "zvvnmod-to-utn57".to_owned());
    let Some(input) = arguments.next() else {
        eprintln!("usage: {program} <zvvnmod-text>");
        return ExitCode::from(2);
    };
    if arguments.next().is_some() {
        eprintln!("usage: {program} <zvvnmod-text>");
        return ExitCode::from(2);
    }

    match convert_zvvnmod_to_utn57(&input) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
