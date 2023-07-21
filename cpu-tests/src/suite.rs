use std::error::Error;
use std::fs;
mod wdc65c816;

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string(path)?;
    let tests = wdc65c816::parse(&data)?;

    // Only do first test for now
    for test in &tests[0..1] {
        wdc65c816::run(test);
    }

    Ok(())
}
