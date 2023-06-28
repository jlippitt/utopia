use bios::BiosLoader;
use clap::Parser;
use sdl2::event::Event;
use std::error::Error;
use std::fs;
use utopia::Options;
use video::{Video, VideoOptions};

mod bios;
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

    #[cfg(debug_assertions)]
    let subscriber = log::create_debug_writer("main")?;

    #[cfg(not(debug_assertions))]
    let subscriber = std::io::stdout;

    let _guard = log::set_subscriber(subscriber);

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

    let mut event_pump = sdl_context.event_pump()?;

    'outer: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'outer,
                _ => (),
            }
        }

        system.run_frame();

        video.update(&mut texture, system.pixels())?;
    }

    Ok(())
}
