use std::error::Error;
use std::fs;
use tracing::{info, warn};

mod wdc65c816;

pub fn run(path: &str) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string(path)?;
    let tests = wdc65c816::parse(&data)?;

    let mut passed = 0;

    // Only do first test for now
    for test in &tests {
        if wdc65c816::run(test) {
            passed += 1;
        }
    }

    if passed == tests.len() {
        info!("{}: All tests passed", path);
    } else {
        warn!("{}: {} FAILED TESTS!", path, tests.len() - passed);
    }

    Ok(())
}
