use std::path::Path;
use std::process::ExitCode;

use serendip::SerendipThermogram;

fn main() -> ExitCode {
    let mut args = std::env::args();
    let _ = args.next();

    let Some(file_path) = args.next() else {
        eprintln!("Usage: serendip <filepath>");
        return ExitCode::FAILURE;
    };

    match SerendipThermogram::new_from_path(Path::new(&file_path)) {
        Ok(_thermogram) => {
            println!("Successfully decoded thermogram from {file_path}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Failed to decode {file_path}: {e}");
            ExitCode::FAILURE
        }
    }
}
