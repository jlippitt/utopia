use audio::Audio;
use bios::BiosLoader;
use clap::Parser;
use joypad::Joypad;
use mmap::MemoryMapper;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Scancode;
use std::error::Error;
use std::fs;
use utopia::Options;
use video::{Video, VideoOptions};

mod audio;
mod bios;
mod joypad;
mod log;
mod mmap;
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
        bios_loader: BiosLoader::new(args.rom_path.clone().into()),
        memory_mapper: MemoryMapper::new(args.rom_path.clone().into()),
        skip_boot: args.skip_boot,
    };

    let mut system = utopia::create(rom_data, &args.rom_path, &options)?;

    let sdl_context = sdl2::init()?;

    let audio_enabled = system.audio_queue().is_some();

    let mut video = Video::new(
        &sdl_context,
        system.as_ref(),
        VideoOptions {
            upscale: args.upscale,
            full_screen: args.full_screen,
            disable_vsync: audio_enabled || args.disable_vsync,
        },
    )?;

    let texture_creator = video.texture_creator();

    let mut texture = video.create_texture(
        &texture_creator,
        system.screen_width(),
        system.screen_height(),
    )?;

    let mut prev_resolution = system.screen_resolution();

    let sample_rate = system.sample_rate();

    let mut joypad = Joypad::new(&sdl_context)?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut audio = Audio::new(&sdl_context, sample_rate)?;

    'outer: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::ControllerButtonDown { which, button, .. } => {
                    joypad.button_event(which, button, true)
                }
                Event::ControllerButtonUp { which, button, .. } => {
                    joypad.button_event(which, button, false)
                }
                Event::ControllerAxisMotion {
                    which, axis, value, ..
                } => {
                    joypad.axis_event(which, axis, value);
                }
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => match scancode {
                    Scancode::F11 => {
                        audio.pause();
                        video.toggle_full_screen()?;
                        audio.resume();
                    }
                    Scancode::Escape => break 'outer,
                    _ => joypad.key_event(scancode, true),
                },
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => joypad.key_event(scancode, false),
                Event::ControllerDeviceAdded { which, .. } => joypad.add_controller(which),
                Event::ControllerDeviceRemoved { which, .. } => joypad.remove_controller(which),
                Event::Window {
                    win_event: WindowEvent::SizeChanged(_, _),
                    ..
                } => video.on_window_size_changed()?,
                Event::Quit { .. } => break 'outer,
                _ => (),
            }
        }

        system.run_frame(joypad.state());

        if system.screen_resolution() != prev_resolution {
            video.set_screen_size(system.screen_width(), system.screen_height())?;

            texture = video.create_texture(
                &texture_creator,
                system.screen_width(),
                system.screen_height(),
            )?;

            prev_resolution = system.screen_resolution();
        }

        video.update(&mut texture, system.pixels(), system.pitch())?;

        if let Some(audio_queue) = system.audio_queue() {
            audio.sync(audio_queue);
        }
    }

    Ok(())
}
