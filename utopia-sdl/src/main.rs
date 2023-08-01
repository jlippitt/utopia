use audio::Audio;
use bios::BiosLoader;
use clap::Parser;
use joypad::Joypad;
use sdl2::event::{Event, WindowEvent};
use std::error::Error;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use utopia::{Options, Sync};
use video::{Video, VideoOptions};

mod audio;
mod bios;
mod joypad;
mod log;
mod video;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,

    #[arg(short, long)]
    full_screen: bool,

    #[arg(short, long)]
    disable_vsync: bool,

    #[arg(short, long)]
    skip_boot: bool,

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
            full_screen: args.full_screen,
            disable_vsync: args.disable_vsync || system.sync() != Sync::Video,
        },
    )?;

    let texture_creator = video.texture_creator();

    let mut texture = video.create_texture(&texture_creator)?;

    let sample_rate = system.sample_rate();

    let mut audio = Audio::new(&sdl_context, sample_rate.try_into()?)?;

    let mut joypad = Joypad::new(&sdl_context)?;

    let mut event_pump = sdl_context.event_pump()?;

    audio.resume();

    let start_time = Instant::now();

    'outer: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::ControllerButtonDown { which, button, .. } => {
                    joypad.button_event(which, button, true)
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    joypad.button_event(which, button, false)
                }
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => joypad.key_event(scancode, true),
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => joypad.key_event(scancode, false),
                Event::ControllerDeviceAdded { which, .. } => joypad.add_controller(which),
                Event::ControllerDeviceRemoved { which, .. } => joypad.remove_controller(which),
                Event::Window {
                    win_event: WindowEvent::SizeChanged(_, _),
                    ..
                } => video.on_size_changed()?,
                Event::Quit { .. } => break 'outer,
                _ => (),
            }
        }

        system.run_frame(joypad.state());

        video.update(&mut texture, system.pixels())?;

        if system.sync() == Sync::Audio {
            let expected_duration = (system.total_samples() * 1000) / sample_rate;
            let expected_time = start_time + Duration::from_millis(expected_duration);
            let actual_time = Instant::now();

            let duration = if actual_time < expected_time {
                expected_time - actual_time
            } else {
                Duration::ZERO
            };

            thread::sleep(duration);
        }
    }

    Ok(())
}
