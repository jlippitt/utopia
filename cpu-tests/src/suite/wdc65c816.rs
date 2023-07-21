use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use utopia::core::wdc65c816::{Bus, Core, Interrupt};

#[derive(Debug, Deserialize)]
pub struct State {
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
pub struct Test {
    name: String,
    initial: State,
    r#final: State,
    cycles: Vec<(u32, u8, String)>,
}

#[derive(Debug)]
pub struct Memory {
    map: HashMap<u32, u8>,
}

impl Memory {
    pub fn new(ram: &[(u32, u8)]) -> Self {
        let mut map = HashMap::new();

        for (address, value) in ram {
            map.insert(*address, *value);
        }

        Self { map }
    }
}

impl Bus for Memory {
    fn idle(&mut self) {
        //
    }

    fn read(&mut self, address: u32) -> u8 {
        *self.map.get(&address).unwrap_or(&0)
    }

    fn write(&mut self, address: u32, value: u8) {
        self.map.insert(address, value);
    }

    fn poll(&self) -> Interrupt {
        0
    }

    fn acknowledge(&mut self, _interrupt: Interrupt) {
        //
    }
}

pub fn parse(input: &str) -> Result<Vec<Test>, Box<dyn Error>> {
    let tests: Vec<Test> = serde_json::from_str(input)?;
    Ok(tests)
}

pub fn run(test: &Test) {
    println!("{:?}", test);
    let memory = Memory::new(&test.initial.ram);
    println!("{:?}", memory);
}
