use super::{Ppu, WIDTH};
use fifo::Fifo;

mod fifo;

pub struct RenderState {
    pos_x: usize,
    //bg_ready: bool,
    bg_step: u32,
    //bg_fine_x: u8,
    bg_fifo: Fifo<u8>,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            pos_x: 0,
            //bg_ready: false,
            bg_step: 0,
            //bg_fine_x: 0,
            bg_fifo: Fifo::new(),
        }
    }
}

impl Ppu {
    pub(super) fn reset_renderer(&mut self) {
        self.render = RenderState {
            pos_x: 0,
            //bg_ready: false,
            bg_step: 0,
            //bg_fine_x: self.scroll_x & 7,
            bg_fifo: Fifo::new(),
        };
    }

    pub(super) fn step_renderer(&mut self) -> bool {
        self.fetch_bg();

        if let Some(color) = self.render.bg_fifo.pop() {
            self.screen.draw_pixel(color);
            self.render.pos_x += 1;
        }

        self.render.pos_x == WIDTH
    }

    fn fetch_bg(&mut self) {
        match self.render.bg_step {
            0 | 2 | 4 => self.render.bg_step += 1,
            1 => {
                // Name
                self.render.bg_step += 1;
            }
            3 => {
                // CHR Low
                self.render.bg_step += 1;
            }
            5 => {
                // CHR High
                self.render.bg_step += 1;
            }
            6 => {
                // Push
                if self.render.bg_fifo.try_push([0; 8]) {
                    self.render.bg_step = 0;
                }
            }
            _ => unreachable!(),
        }
    }
}
