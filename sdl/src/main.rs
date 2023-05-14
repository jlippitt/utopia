use clap::Parser;
use std::error::Error;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = fs::read(args.rom_path)?;

    println!("{:?}", rom_data);

    Ok(())
}
