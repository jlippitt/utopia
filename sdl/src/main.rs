use clap::Parser;
use std::error::Error;
use std::{fs, io, thread};
use tracing::debug;

mod log;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let _rom_data = fs::read(args.rom_path)?;

    let _guard = log::set_subscriber(io::stdout);

    debug!("UI thread");

    let inner_thread = thread::spawn(move || {
        let _guard = log::set_subscriber(log::create_debug_writer("main").unwrap());

        debug!("Inner thread");
    });

    inner_thread.join().unwrap();

    Ok(())
}
