pub use wdc65c816::Wdc65c816;

use std::error::Error;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

mod wdc65c816;

pub trait Runner {
    type Test;
    fn parse(input: &str) -> Result<Vec<Self::Test>, Box<dyn Error>>;
    fn run(test: &Self::Test) -> bool;
}

pub fn run<T: Runner>(path: &str) -> Result<bool, Box<dyn Error>> {
    let failed = run_path::<T>(path)?;

    if failed == 0 {
        info!("All test suites passed");
        Ok(true)
    } else {
        warn!("{} FAILED TEST SUITES!", failed);
        Ok(false)
    }
}

fn run_path<T: Runner>(path: &str) -> Result<u32, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        let mut failed: u32 = 0;

        for entry in fs::read_dir(path)? {
            if let Some(path) = entry?.path().to_str() {
                failed += run_file::<T>(path)?;
            }
        }

        Ok(failed)
    } else if metadata.is_file() {
        Ok(run_file::<T>(path)?)
    } else {
        Ok(0)
    }
}

fn run_file<T: Runner>(path: &str) -> Result<u32, Box<dyn Error>> {
    let name = Path::new(path).file_name().unwrap().to_str().unwrap();

    let data = fs::read_to_string(path)?;
    let tests = T::parse(&data)?;

    let mut failed = 0;

    // Only do first test for now
    for test in &tests {
        if !T::run(test) {
            failed += 1;
        }
    }

    if failed == 0 {
        info!("PASSED: {}", name);
        Ok(0)
    } else {
        warn!("FAILED: {} ({} failing test cases!)", name, failed);
        Ok(1)
    }
}
