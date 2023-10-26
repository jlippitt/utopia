use super::interrupt::{RcpIntType, RcpInterrupt};
use super::WgpuContext;
use crate::util::memory::{Masked, Reader, Writer};
use crate::util::size::Size;
use crate::util::upscaler::Upscaler;
use registers::{AntiAliasMode, ColorDepth, HSync, HalfLine, Registers, VSync};
use tracing::trace;

mod registers;

pub fn decode_rgba16(chunk: &[u8]) -> [u8; 4] {
    let value = u16::from_be_bytes([chunk[0], chunk[1]]);
    let red = ((value >> 11) as u8 & 31) << 3;
    let green = ((value >> 6) as u8 & 31) << 3;
    let blue = ((value >> 1) as u8 & 31) << 3;
    let alpha = (value as u8 & 1) * 255;
    [red, green, blue, alpha]
}

pub struct VideoInterface {
    h_counter: u64,
    v_counter: u32,
    field: bool,
    frame_complete: bool,
    regs: Registers,
    rcp_int: RcpInterrupt,
    pixels: Vec<u8>,
    upscaler: Upscaler,
}

impl VideoInterface {
    pub const DEFAULT_TARGET_SIZE: Size = Size::new(800, 600);

    const MIN_SOURCE_SIZE: Size = Size::new(256, 192);

    pub fn new(ctx: WgpuContext, rcp_int: RcpInterrupt) -> Self {
        Self {
            h_counter: 0,
            v_counter: 0,
            field: false,
            frame_complete: false,
            regs: Registers {
                v_intr: HalfLine::new().with_half_line(0x3ff),
                v_sync: VSync::new().with_v_sync(0x3ff),
                h_sync: HSync::new().with_h_sync(0x7ff),
                ..Default::default()
            },
            rcp_int,
            pixels: Vec::new(),
            upscaler: Upscaler::new(ctx, Self::MIN_SOURCE_SIZE, Self::DEFAULT_TARGET_SIZE, true),
        }
    }

    pub fn frame_complete(&self) -> bool {
        self.frame_complete
    }

    pub fn reset_frame_complete(&mut self) {
        self.frame_complete = false;
    }

    pub fn step(&mut self, cycles: u64) {
        self.h_counter += cycles;

        let cycles_per_line = self.regs.h_sync.h_sync() as u64;

        while self.h_counter >= cycles_per_line {
            self.h_counter -= cycles_per_line;
            self.v_counter += 1;

            // VCurrent & VSync are given in half lines, with the low bit
            // representing the field in interlace mode
            self.regs
                .v_current
                .set_half_line((self.v_counter & !1) | self.field as u32);

            if self.v_counter >= self.regs.v_sync.v_sync() {
                self.v_counter = 0;
                self.field ^= self.regs.ctrl.serrate();
                self.regs.v_current.set_half_line(self.field as u32);
                self.frame_complete = true;
            }

            trace!("Line: {}", self.v_counter);

            if self.v_counter == (self.regs.v_intr.half_line() >> 1) {
                self.rcp_int.raise(RcpIntType::VI);
            }
        }
    }

    pub fn update(&mut self, rdram: &[u8]) -> Result<(), wgpu::SurfaceError> {
        let resample = self.regs.ctrl.aa_mode() != AntiAliasMode::Disabled;

        self.upscaler.set_resample(resample);

        let buffer_width = self.regs.width.width();

        let screen_width = (self.regs.h_video.end() - self.regs.h_video.start())
            * self.regs.x_scale.scale()
            / 1024;

        let screen_height = ((self.regs.v_video.end() - self.regs.v_video.start()) >> 1)
            * self.regs.y_scale.scale()
            / 1024;

        // Don't try to copy pixels if the screen width is zero
        let color_depth = if buffer_width != 0 && screen_width != 0 && screen_height != 0 {
            self.regs.ctrl.color_depth()
        } else {
            ColorDepth::Blank
        };

        let source_size = Size::new(
            screen_width.max(Self::MIN_SOURCE_SIZE.width),
            screen_height.max(Self::MIN_SOURCE_SIZE.height),
        );

        self.upscaler.set_source_size(source_size);

        self.pixels.resize(
            source_size.width as usize * source_size.height as usize * 4,
            0,
        );

        match color_depth {
            ColorDepth::Blank => self.pixels.fill(0),
            ColorDepth::Reserved => panic!("Invalid use of 'Reserved' color depth"),
            ColorDepth::Color16 => {
                let src_pitch = buffer_width as usize * 2;
                let dst_pitch = screen_width as usize * 4;
                let dst_display = screen_width.min(buffer_width) as usize * 4;

                let mut src = self.regs.origin.origin() as usize;
                let mut dst = 0;

                for _ in 0..screen_height {
                    let iter = rdram[src..(src + src_pitch)]
                        .chunks_exact(2)
                        .flat_map(decode_rgba16);

                    self.pixels.splice(dst..(dst + dst_display), iter);
                    self.pixels[(dst + dst_display)..(dst + dst_pitch)].fill(0);

                    src += src_pitch;
                    dst += dst_pitch;
                }
            }
            ColorDepth::Color32 => {
                let src_pitch = buffer_width as usize * 4;
                let dst_pitch = screen_width as usize * 4;
                let dst_display = dst_pitch.min(src_pitch);

                let mut src = self.regs.origin.origin() as usize;
                let mut dst = 0;

                for _ in 0..screen_height {
                    let iter = rdram[src..(src + src_pitch)].iter().copied();

                    self.pixels.splice(dst..(dst + dst_display), iter);
                    self.pixels[(dst + dst_display)..(dst + dst_pitch)].fill(0);

                    src += src_pitch;
                    dst += dst_pitch;
                }
            }
        }

        self.upscaler.update(&self.pixels);

        Ok(())
    }

    pub fn render(&self, canvas: &wgpu::Texture) {
        self.upscaler.render(canvas);
    }
}

impl Reader for VideoInterface {
    fn read_u32(&self, address: u32) -> u32 {
        match address {
            0x00 => self.regs.ctrl.into(),
            0x04 => self.regs.origin.into(),
            0x08 => self.regs.width.into(),
            0x0c => self.regs.v_intr.into(),
            0x10 => self.regs.v_current.into(),
            0x14 => self.regs.burst.into(),
            0x18 => self.regs.v_sync.into(),
            0x1c => self.regs.h_sync.into(),
            0x20 => self.regs.h_sync_leap.into(),
            0x24 => self.regs.h_video.into(),
            0x28 => self.regs.v_video.into(),
            0x2c => self.regs.v_burst.into(),
            0x30 => self.regs.x_scale.into(),
            0x34 => self.regs.y_scale.into(),
            0x38 => self.regs.test_addr.into(),
            0x3c => self.regs.staged_data,
            _ => unimplemented!("Video Interface Read: {:08X}", address),
        }
    }
}

impl Writer for VideoInterface {
    type SideEffect = ();

    fn write_u32(&mut self, address: u32, value: Masked<u32>) {
        match address {
            0x00 => value.write_reg("VI_CTRL", &mut self.regs.ctrl),
            0x04 => value.write_reg_hex("VI_ORIGIN", &mut self.regs.origin),
            0x08 => value.write_reg("VI_WIDTH", &mut self.regs.width),
            0x0c => value.write_reg("VI_V_INTR", &mut self.regs.v_intr),
            0x10 => {
                // VI_V_CURRENT
                self.rcp_int.clear(RcpIntType::VI);
            }
            0x14 => value.write_reg("VI_BURST", &mut self.regs.burst),
            0x18 => value.write_reg("VI_V_SYNC", &mut self.regs.v_sync),
            0x1c => value.write_reg("VI_H_SYNC", &mut self.regs.h_sync),
            0x20 => value.write_reg("VI_H_SYNC_LEAP", &mut self.regs.h_sync_leap),
            0x24 => value.write_reg("VI_H_VIDEO", &mut self.regs.h_video),
            0x28 => value.write_reg("VI_V_VIDEO", &mut self.regs.v_video),
            0x2c => value.write_reg("VI_V_BURST", &mut self.regs.v_burst),
            0x30 => value.write_reg("VI_X_SCALE", &mut self.regs.x_scale),
            0x34 => value.write_reg("VI_Y_SCALE", &mut self.regs.y_scale),
            0x38 => value.write_reg("VI_TEST_ADDR", &mut self.regs.test_addr),
            0x3c => value.write_reg("VI_STAGED_DATA", &mut self.regs.staged_data),
            _ => unimplemented!("Video Interface Write: {:08X}", address),
        }
    }
}
