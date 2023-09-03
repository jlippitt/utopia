use bios::BiosLoader;
use clap::builder::PossibleValue;
use clap::{Parser, ValueEnum};
use mmap::MemoryMapper;
use std::error::Error;
use std::path::PathBuf;
use utopia_winit::{App, AppOptions, Sync};

mod bios;
mod log;
mod mmap;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct SyncArg(Sync);

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: PathBuf,

    #[arg(short, long)]
    full_screen: bool,

    #[arg(short, long)]
    bios_path: Option<PathBuf>,

    #[arg(short, long)]
    skip_boot: bool,

    #[arg(value_enum, long)]
    sync: Option<SyncArg>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let _log = log::init()?;

    let rom_data = std::fs::read(&args.rom_path)?;

    let app = App::new(AppOptions {
        rom_path: args.rom_path.clone(),
        rom_data,
        bios_loader: Box::new(BiosLoader::new(
            args.bios_path.unwrap_or(args.rom_path.clone()),
        )),
        memory_mapper: MemoryMapper::new(args.rom_path.clone()),
        skip_boot: args.skip_boot,
        full_screen: args.full_screen,
        sync: args.sync.map(|sync| sync.0),
    })?;

    app.run()?;

    Ok(())
}

impl ValueEnum for SyncArg {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self(Sync::None), Self(Sync::Video), Self(Sync::Audio)]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self.0 {
            Sync::None => PossibleValue::new("none"),
            Sync::Video => PossibleValue::new("video"),
            Sync::Audio => PossibleValue::new("audio"),
        })
    }
}
