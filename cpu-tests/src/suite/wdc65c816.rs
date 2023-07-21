use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct State {
    pc: u16,
    s: u16,
    p: u8,
    a: u16,
    x: u16,
    y: u16,
    dbr: u8,
    d: u16,
    pbr: u8,
    e: u8,
    ram: Vec<(u32, u8)>,
}

#[derive(Debug, Deserialize)]
struct Test {
    name: String,
    initial: State,
    r#final: State,
    cycles: Vec<(u32, u8, String)>,
}

pub fn parse(input: &str) -> Result<(), Box<dyn Error>> {
    let tests: Vec<Test> = serde_json::from_str(input)?;
    println!("{:?}", tests);
    Ok(())
}
