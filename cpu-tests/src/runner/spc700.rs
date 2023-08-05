use super::Runner;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use tracing::{debug, info};
use utopia::core::spc700::{Bus, Core, State};

#[derive(Debug, Deserialize)]
pub struct TestState {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    psw: u8,
    ram: Vec<(u16, u8)>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Test {
    name: String,
    initial: TestState,
    r#final: TestState,
    cycles: Vec<(Option<u16>, Option<u8>, String)>,
}

pub struct Spc700;

impl Runner for Spc700 {
    type Test = Test;

    fn parse(input: &str) -> Result<Vec<Test>, Box<dyn Error>> {
        let tests: Vec<Test> = serde_json::from_str(input)?;
        Ok(tests)
    }

    fn run(test: &Test) -> bool {
        let memory = Memory::new(&test.initial.ram);
        let mut core = Core::new(memory);
        let core_initial = State::from(&test.initial);

        core.set_state(&core_initial);
        core.step();

        let core_actual = core.state();
        let core_expected = State::from(&test.r#final);

        let ram_actual = core.bus().values();
        let mut ram_expected = test.r#final.ram.clone();
        ram_expected.sort();

        let core_ok = core_actual == core_expected;
        let ram_ok = ram_actual == ram_expected;

        if core_ok && ram_ok {
            debug!("Passed: {}", test.name);
            return true;
        }

        info!("Failed: {}", test.name);

        if !core_ok {
            info!("Core Initial: {:?}", core_initial);
            info!("Core Expected: {:?}", core_expected);
            info!("Core Actual: {:?}", core_actual);
        }

        if !ram_ok {
            let mut ram_initial = test.initial.ram.clone();
            ram_initial.sort();

            info!("RAM Initial: {:?}", ram_initial);
            info!("RAM Expected: {:?}", ram_expected);
            info!("RAM Actual: {:?}", ram_actual);
        }

        false
    }
}

#[derive(Debug)]
struct Memory {
    map: HashMap<u16, u8>,
}

impl Memory {
    fn new(ram: &[(u16, u8)]) -> Self {
        let mut map = HashMap::new();

        for (address, value) in ram {
            map.insert(*address, *value);
        }

        Self { map }
    }

    fn values(&self) -> Vec<(u16, u8)> {
        let mut vec: Vec<(u16, u8)> = self.map.iter().map(|(k, v)| (*k, *v)).collect();
        vec.sort();
        vec
    }
}

impl Bus for Memory {
    fn idle(&mut self) {}

    fn read(&mut self, address: u16) -> u8 {
        *self.map.get(&address).unwrap_or(&0)
    }

    fn write(&mut self, address: u16, value: u8) {
        self.map.insert(address, value);
    }
}

impl From<&TestState> for State {
    fn from(state: &TestState) -> State {
        State {
            a: state.a,
            x: state.x,
            y: state.y,
            sp: state.sp,
            pc: state.pc,
            psw: state.psw,
        }
    }
}
