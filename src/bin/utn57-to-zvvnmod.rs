use std::process::ExitCode;
use zvvnmod_utn57::convert_utn57_to_zvvnmod;

fn main() -> ExitCode {
    let mut arguments = std::env::args();
    let program = arguments
        .next()
        .unwrap_or_else(|| "utn57-to-zvvnmod".to_owned());
    let Some(input) = arguments.next() else {
        eprintln!("usage: {program} <mongolian-text>");
        return ExitCode::from(2);
    };
    if arguments.next().is_some() {
        eprintln!("usage: {program} <mongolian-text>");
        return ExitCode::from(2);
    }

    match convert_utn57_to_zvvnmod(&input) {
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
