use clap::Parser;
use std::error::Error;
use std::process::ExitCode;

mod log;
mod runner;

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    runner: String,
    path: String,
}

fn main() -> Result<ExitCode, Box<dyn Error>> {
    let _guard = log::init()?;

    let args = Args::parse();

    let all_tests_passed = match args.runner.as_str() {
        "spc700" => runner::run::<runner::Spc700>(&args.path)?,
        "wdc65c816" => runner::run::<runner::Wdc65c816>(&args.path)?,
        _ => Err(format!("Test suite '{}' not found", args.runner))?,
    };

    Ok(if all_tests_passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
