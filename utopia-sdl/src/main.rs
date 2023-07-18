use bios::BiosLoader;
use clap::Parser;
use joypad::Joypad;
use sdl2::event::Event;
use std::error::Error;
use std::fs;
use utopia::Options;
use video::{Video, VideoOptions};

mod bios;
mod joypad;
mod log;
mod video;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,

    #[arg(short, long)]
    skip_boot: bool,

    #[arg(short, long)]
    disable_vsync: bool,

    #[arg(short, long)]
    upscale: Option<u32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = fs::read(&args.rom_path)?;

    let _guard = log::init()?;

    let options = Options {
        skip_boot: args.skip_boot,
    };

    let mut system = utopia::create(&args.rom_path, rom_data, &BiosLoader::new(), &options)?;

    let sdl_context = sdl2::init()?;

    let mut video = Video::new(
        &sdl_context,
        VideoOptions {
            width: system.width().try_into()?,
            height: system.height().try_into()?,
            clip_top: system.clip_top().try_into()?,
            clip_bottom: system.clip_bottom().try_into()?,
            upscale: args.upscale,
            disable_vsync: args.disable_vsync,
        },
    )?;

    let texture_creator = video.texture_creator();

    let mut texture = video.create_texture(&texture_creator)?;

    let mut joypad = Joypad::new();

    let mut event_pump = sdl_context.event_pump()?;

    'outer: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => joypad.key_event(scancode, true),
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => joypad.key_event(scancode, false),
                Event::Quit { .. } => break 'outer,
                _ => (),
            }
        }

        system.run_frame(joypad.state());

        video.update(&mut texture, system.pixels())?;
    }

    Ok(())
}
