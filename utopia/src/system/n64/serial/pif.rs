use crate::util::facade::{DataReader, DataWriter, ReadFacade, WriteFacade};
use arrayvec::ArrayVec;
use tracing::{debug, warn};

pub struct Pif {
    rom: [u8; 2048],
    ram: [u8; 64],
    input: [u8; 64],
}

impl Pif {
    pub fn new() -> Self {
        Self {
            rom: [0; 2048],
            ram: [0; 64],
            input: [0; 64],
        }
    }

    pub fn upload(&mut self) {
        self.input = self.ram;
    }

    pub fn process(&mut self) {
        if (self.input[0x3f] & 0x01) == 0 {
            return;
        }

        debug!("PIF JoyBus Input: {:X?}", self.input);

        let mut channel = 0;
        let mut index = 0;

        while index < self.input.len() {
            let send_bytes = self.input[index] as usize;
            index += 1;

            if send_bytes == 0xfe {
                break;
            }

            if (send_bytes & 0xc0) != 0 {
                continue;
            }

            if send_bytes == 0 {
                channel += 1;
                continue;
            }

            let recv_bytes = self.input[index] as usize;
            index += 1;

            if recv_bytes == 0xfe {
                break;
            }

            if (index + send_bytes) > self.input.len() {
                warn!("JoyBus read overflow");
                break;
            }

            let send_data =
                ArrayVec::<u8, 64>::try_from(&self.input[index..(index + send_bytes)]).unwrap();

            index += send_bytes;

            if (index + recv_bytes) > self.input.len() {
                warn!("JoyBus write overflow");
                break;
            }

            if let Some(recv_data) = self.query_joybus(channel, &send_data) {
                assert!(recv_data.len() == recv_bytes);
                self.ram[index..(index + recv_bytes)].copy_from_slice(&recv_data);
                index += recv_bytes;
            } else {
                self.ram[index - 2] |= 0x80;
            }

            channel += 1;
        }

        self.ram[0x3f] = 0;

        debug!("PIF JoyBus Output: {:X?}", self.ram);
    }

    fn query_joybus(&mut self, channel: usize, input: &[u8]) -> Option<ArrayVec<u8, 64>> {
        let mut output = ArrayVec::new();

        match input[0] {
            0x00 | 0x0f => {
                match channel {
                    0 => {
                        output.push(0x05);
                        output.push(0x00);
                        output.push(0x02); // TODO: Controller Pak
                    }
                    1 | 2 | 3 => return None,
                    4 => todo!("EEPROM"),
                    _ => panic!("Invalid JoyBus channel: {}", channel),
                }
            }
            0x01 => {
                if channel > 3 {
                    panic!("Invalid JoyBus channel: {}", channel);
                }

                // TODO: Controller button reads
                output.push(0x00);
                output.push(0x00);
                output.push(0x00);
                output.push(0x00);
            }
            0x02 => {
                if channel > 3 {
                    panic!("Invalid JoyBus channel: {}", channel);
                }

                warn!("Controller Pak reads not yet implemented");

                for _ in 0..32 {
                    output.push(0);
                }

                output.push(crc(&output[0..32]));
            }
            0x03 => {
                if channel > 3 {
                    panic!("Invalid JoyBus channel: {}", channel);
                }

                warn!("Controller Pak writes not yet implemented");
                output.push(crc(&input[3..35]));
            }
            _ => panic!("Unknown JoyBus command: {:02X}", input[0]),
        }

        Some(output)
    }
}

impl DataReader for Pif {
    type Address = u32;
    type Value = u8;

    fn read(&self, address: u32) -> u8 {
        match address {
            0x0000_00000..=0x0000_007bf => self.rom.as_slice().read_be(address as usize),
            0x0000_007c0..=0x0000_007ff => self.ram.as_slice().read_be(address as usize & 63),
            _ => unimplemented!("Serial Bus Read: {:08X}", address),
        }
    }
}

impl DataWriter for Pif {
    fn write(&mut self, address: u32, value: u8) {
        match address {
            0x0000_007c0..=0x0000_007ff => self
                .ram
                .as_mut_slice()
                .write_be(address as usize & 63, value),
            _ => unimplemented!("Serial Bus Write: {:08X} <= {:08X}", address, value),
        }
    }
}

pub fn crc(data: &[u8]) -> u8 {
    assert!(data.len() == 32);

    let mut result: u8 = 0;

    for index in 0..=data.len() {
        for bit in (0..=7).rev() {
            let xor_tap = if (result & 0x80) != 0 { 0x85 } else { 0 };
            result <<= 1;

            if index < data.len() && (data[index] & (1 << bit)) != 0 {
                result |= 1;
            }

            result ^= xor_tap;
        }
    }

    result
}
