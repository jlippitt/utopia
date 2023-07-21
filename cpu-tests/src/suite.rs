use std::error::Error;
use std::fs;
mod wdc65c816;

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string(path)?;
    wdc65c816::parse(&data)?;
    Ok(())
}
