use super::FRAME_IRQ;
use crate::core::mos6502::Interrupt;
use tracing::debug;

const STEPS: [u64; 5] = [7458, 14914, 22372, 29830, 37282];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FrameEvent {
    Quarter,
    Half,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Mode {
    Short,
    Long,
}

pub struct FrameCounter {
    cycles: u64,
    target_cycles: u64,
    step: u8,
    mode: Mode,
    irq_inhibit: bool,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            cycles: 0,
            target_cycles: STEPS[0],
            step: 0,
            mode: Mode::Short,
            irq_inhibit: false,
        }
    }

    pub fn set_control(&mut self, interrupt: &mut Interrupt, value: u8) {
        self.mode = if (value & 0x80) != 0 {
            Mode::Long
        } else {
            Mode::Short
        };
        debug!("Frame Counter Mode: {:?}", self.mode);

        self.irq_inhibit = (value & 0x40) != 0;
        debug!("Frame Counter IRQ Inhibit: {}", self.irq_inhibit);

        if self.irq_inhibit {
            *interrupt &= !FRAME_IRQ;
            debug!("Frame Counter IRQ Cleared");
        }

        // Prepare for timer reset
        // TODO: Slightly more accurate timing, dependent on odd/even cycles?
        self.step = 4;
        self.target_cycles = self.cycles + 3;
    }

    pub fn step(&mut self, interrupt: &mut Interrupt) -> Option<FrameEvent> {
        self.cycles += 1;

        if self.cycles != self.target_cycles {
            return None;
        }

        let frame = match self.step {
            0 => {
                self.step += 1;
                Some(FrameEvent::Quarter)
            }
            1 => {
                self.step += 1;
                Some(FrameEvent::Half)
            }
            2 => {
                self.step += 1;
                Some(FrameEvent::Quarter)
            }
            3 => match self.mode {
                Mode::Short => {
                    self.cycles = 0;
                    self.step = 0;

                    if !self.irq_inhibit {
                        *interrupt |= FRAME_IRQ;
                        debug!("Frame Counter IRQ Raised");
                    }

                    Some(FrameEvent::Half)
                }
                Mode::Long => {
                    self.step += 1;
                    None
                }
            },
            4 => {
                self.cycles = 0;
                self.step = 0;

                match self.mode {
                    Mode::Short => None,
                    Mode::Long => Some(FrameEvent::Half),
                }
            }
            _ => unreachable!(),
        };

        self.target_cycles = STEPS[self.step as usize];

        debug!(
            "Frame Counter Step: {} (Target Cycles: {})",
            self.step, self.target_cycles
        );

        frame
    }
}
