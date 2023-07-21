use clap::Parser;
use std::error::Error;

mod suite;

#[derive(Debug, Parser)]
#[command(author, version)]
struct Args {
    suite: String,
    path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.suite.as_str() {
        "wdc65c816" => suite::run(&args.path)?,
        _ => panic!("Test suite '{}' not found", args.suite),
    }

    Ok(())
}
