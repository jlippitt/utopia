use clap::Parser;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use std::error::Error;
use std::fs;

mod log;

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

    //system.run();

    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;
    let width = system.width();
    let height = system.height();
    let pitch = width as usize * 4;

    let pixels = vec![0u8; pitch * height as usize];

    let window = video.window("Utopia", width, height)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas()
        .present_vsync()
        .build()?;

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB888, width, height)?;

    let mut event_pump = sdl_context.event_pump()?;

    'outer: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'outer,
                _ => (),
            }
        }

        texture.update(None, &pixels, pitch)?;

        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();
    }

    Ok(())
}
