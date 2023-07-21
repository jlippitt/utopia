use clap::Parser;
use std::error::Error;
use std::process::ExitCode;

mod log;
mod suite;

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    suite: String,
    path: String,
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let _guard = log::init()?;

    let args = Args::parse();

    let all_passed = match args.suite.as_str() {
        "wdc65c816" => suite::run(&args.path)?,
        _ => Err(format!("Test suite '{}' not found", args.suite))?,
    };

    Ok(if all_passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
