use clap::Parser;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use std::error::Error;
use std::fs;
use video::Video;

mod log;
mod video;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = fs::read(&args.rom_path)?;

    #[cfg(debug_assertions)]
    let subscriber = log::create_debug_writer("main")?;

    #[cfg(not(debug_assertions))]
    let subscriber = std::io::stdout;

    let _guard = log::set_subscriber(subscriber);

    let mut system = utopia::create(&args.rom_path, rom_data)?;

    let pixels = vec![0u8; system.width() as usize * system.height() as usize * 4];

    //system.run();

    let sdl_context = sdl2::init()?;
    
    let mut video = Video::new(&sdl_context, system.width(), system.height())?;

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

        video.update(&mut texture, &pixels)?;
    }

    Ok(())
}
