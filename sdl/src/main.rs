use clap::Parser;
use std::error::Error;
use std::{fs, io};

mod log;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = fs::read(&args.rom_path)?;

    let _guard = log::set_subscriber(io::stdout);

    utopia::create(&args.rom_path, rom_data);

    Ok(())
}
