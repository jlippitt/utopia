use crate::util::memory::{Memory, Value};
use crate::JoypadState;
use arrayvec::ArrayVec;
use tracing::{debug, warn};

const PIF_SIZE: usize = 0x800;
const PIF_RAM_START: usize = 0x7c0;

pub struct Pif {
    data: Memory,
    input: [u8; 64],
    joypads: [[u8; 4]; 4],
}

impl Pif {
    pub fn new() -> Self {
        Self {
            data: Memory::new(PIF_SIZE),
            input: [0; 64],
            joypads: [[0; 4]; 4],
        }
    }

    pub fn read<T: Value>(&self, address: u32) -> T {
        self.data.read_be(address as usize)
    }

    pub fn write(&mut self, address: u32, value: impl Value) {
        let address = address as usize;

        if address >= PIF_RAM_START {
            self.data.write_be(address, value);
        } else {
            warn!("PIF ROM cannot be written");
        }
    }

    pub fn upload(&mut self) {
        self.input.copy_from_slice(&self.data[PIF_RAM_START..]);
    }

    pub fn update_joypad(&mut self, state: &JoypadState) {
        let JoypadState { buttons, axes } = &state;
        let joypad = &mut self.joypads[0];

        joypad[0] = 0;

        // A, B, Z, Start, D-Up, D-Down, D-Left, D-Right
        joypad[0] |= if buttons[0] { 0x80 } else { 0 };
        joypad[0] |= if buttons[2] { 0x40 } else { 0 };
        joypad[0] |= if buttons[4] { 0x20 } else { 0 };
        joypad[0] |= if buttons[9] { 0x10 } else { 0 };
        joypad[0] |= if buttons[12] { 0x08 } else { 0 };
        joypad[0] |= if buttons[13] { 0x04 } else { 0 };
        joypad[0] |= if buttons[14] { 0x02 } else { 0 };
        joypad[0] |= if buttons[15] { 0x01 } else { 0 };

        // RST 'button' possibly doesn't need to be implemented?
        joypad[1] = 0;

        // L
        joypad[1] |= if buttons[6] { 0x20 } else { 0 };

        // R
        joypad[1] |= if buttons[5] | buttons[7] { 0x10 } else { 0 };

        // C-Up
        joypad[1] |= if axes[3] > (i32::MAX / 4 * 3) {
            0x08
        } else {
            0
        };

        // C-Down
        joypad[1] |= if buttons[1] || axes[3] < (i32::MIN / 4 * 3) {
            0x04
        } else {
            0
        };

        // C-Left
        joypad[1] |= if buttons[3] || axes[2] < (i32::MIN / 4 * 3) {
            0x02
        } else {
            0
        };

        // C-Right
        joypad[1] |= if axes[2] > (i32::MAX / 4 * 3) {
            0x01
        } else {
            0
        };

        // Joystick
        joypad[2] = ((axes[0] / 20 * 13) >> 24) as u8;
        joypad[3] = ((axes[1] / 20 * 13) >> 24) as u8;
    }

    pub fn process(&mut self) {
        if (self.input[0x3f] & 0x01) == 0 {
            return;
        }

        debug!("PIF JoyBus Input: {:X?}", self.input);

        let ram = &mut self.data[PIF_RAM_START..];
        let mut channel = 0;
        let mut index = 0;

        while index < (self.input.len() - 1) {
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

            if let Some(recv_data) = query_joybus(&self.joypads, channel, &send_data) {
                let len = recv_data.len();

                if len != recv_bytes {
                    warn!("Received data does not match expected length. Expected {} bytes but got {} bytes.", recv_bytes, len);
                }

                ram[index..(index + len)].copy_from_slice(&recv_data);
                index += len;
            } else {
                ram[index - 2] |= 0x80;
            }

            channel += 1;
        }

        ram[0x3f] = 0;

        debug!("PIF JoyBus Output: {:X?}", ram);
    }
}

fn query_joybus(joypads: &[[u8; 4]; 4], channel: usize, input: &[u8]) -> Option<ArrayVec<u8, 64>> {
    let mut output = ArrayVec::new();

    match input[0] {
        0x00 | 0xff => {
            match channel {
                0 => {
                    output.push(0x05);
                    output.push(0x00);
                    output.push(0x02); // TODO: Controller Pak
                }
                1..=3 => return None,
                4 => {
                    // Provide 4 Kbit EEPROM by default
                    // TODO: Support other EEPROM sizes
                    output.push(0x00);
                    output.push(0x80);
                    output.push(0x00); // TODO: 'Write in progress' flag
                }
                _ => panic!("Invalid JoyBus channel: {}", channel),
            }
        }
        0x01 => {
            if channel > 3 {
                panic!("Invalid JoyBus channel: {}", channel);
            }

            output.try_extend_from_slice(&joypads[channel]).unwrap();
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
        0x04 => {
            // TODO: EEPROM reads
            for _ in 0..8 {
                output.push(0x00);
            }
        }
        0x05 => {
            // TODO: EEPROM writes
            // TODO: 'Write in progress' flag
            output.push(0x00);
        }
        _ => panic!("Unknown JoyBus command: {:02X}", input[0]),
    }

    Some(output)
}

pub fn crc(data: &[u8]) -> u8 {
    debug_assert!(data.len() == 32);

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
