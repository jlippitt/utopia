use clap::Parser;
use std::error::Error;
use std::{fs, io};
use tracing::debug;

mod log;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = fs::read(args.rom_path)?;

    let subscriber = log::create_subscriber(io::stdout);

    let _guard = tracing::subscriber::set_default(subscriber);

    debug!("{:?}", rom_data);

    Ok(())
}
