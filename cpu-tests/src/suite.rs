use std::error::Error;
use std::fs;
use tracing::{info, warn};

mod wdc65c816;

pub fn run(path: &str) -> Result<bool, Box<dyn Error>> {
    let failed = run_path(path)?;

    if failed == 0 {
        info!("All test suites passed");
        Ok(true)
    } else {
        warn!("{} FAILED TEST SUITES!", failed);
        Ok(false)
    }
}

pub fn run_path(path: &str) -> Result<u32, Box<dyn Error>> {
    let metadata = fs::metadata(path)?;

    if metadata.is_dir() {
        let mut failed: u32 = 0;

        for entry in fs::read_dir(path)? {
            if let Some(path) = entry?.path().to_str() {
                failed += run_file(path)?;
            }
        }

        Ok(failed)
    } else if metadata.is_file() {
        Ok(run_file(path)?)
    } else {
        Ok(0)
    }
}

pub fn run_file(path: &str) -> Result<u32, Box<dyn Error>> {
    info!("Running suite: {}", path);

    let data = fs::read_to_string(path)?;
    let tests = wdc65c816::parse(&data)?;

    let mut failed = 0;

    // Only do first test for now
    for test in &tests {
        if !wdc65c816::run(test) {
            failed += 1;
        }
    }

    if failed == 0 {
        info!("All test cases passed");
        Ok(0)
    } else {
        warn!("{} FAILED TEST CASES!", failed);
        Ok(1)
    }
}
