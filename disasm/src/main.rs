use capstone::arch::mips::ArchMode;
use capstone::arch::{BuildsCapstone, BuildsCapstoneEndian};
use capstone::{Capstone, Endian};
use clap::Parser;
use std::error::Error;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,
    entry_point: String,
}

// Why does this library's Error struct not implement std::error::Error? :(
fn cs_error(err: capstone::Error) -> String {
    format!("{}", err).to_owned()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = fs::read(args.rom_path)?;

    let entry_point = u64::from_str_radix(&args.entry_point, 16)?;

    // TODO: Support multiple architectures
    let mut cs = Capstone::new()
        .mips()
        .mode(ArchMode::Mips64)
        .endian(Endian::Big)
        .detail(true)
        .build()
        .map_err(cs_error)?;

    cs.set_skipdata(true).map_err(cs_error)?;

    let output = cs
        .disasm_all(&rom_data[0x1000..], entry_point)
        .map_err(cs_error)?;

    println!("{}", output);

    Ok(())
}
