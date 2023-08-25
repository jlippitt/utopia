use audio::AudioController;
use bios::BiosLoader;
use clap::Parser;
use gamepad::Gamepad;
use mmap::MemoryMapper;
use std::error::Error;
use utopia::JoypadState;
use video::VideoController;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;

mod audio;
mod bios;
mod gamepad;
mod keyboard;
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
    bios_path: Option<String>,

    #[arg(short, long)]
    skip_boot: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let _log = log::init()?;

    let rom_data = std::fs::read(&args.rom_path)?;

    let mut system = utopia::create(
        rom_data,
        &args.rom_path,
        &utopia::Options {
            bios_loader: BiosLoader::new(args.bios_path.unwrap_or(args.rom_path.clone()).into()),
            memory_mapper: MemoryMapper::new(args.rom_path.clone().into()),
            skip_boot: args.skip_boot,
        },
    )?;

    let event_loop = EventLoop::new();

    let source_size: PhysicalSize<u32> = system.screen_resolution().into();

    let mut video = VideoController::new(&event_loop, source_size, args.full_screen)?;

    let mut audio = AudioController::new(system.sample_rate())?;

    let mut gamepad = Gamepad::new()?;

    let mut joypad_state = JoypadState::default();

    audio.resume()?;

    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => control_flow.set_exit(),
                            Some(VirtualKeyCode::F11) => {
                                video.toggle_full_screen(window_target).unwrap()
                            }
                            _ => (),
                        }
                    }

                    keyboard::handle_input(&mut joypad_state, input);
                }
                WindowEvent::Resized(..) => {
                    video.on_window_size_changed().unwrap();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    video.on_window_size_changed().unwrap();
                }
                WindowEvent::Destroyed => {
                    video.on_target_changed(window_target);
                }
                _ => (),
            },
            Event::RedrawRequested(window_id) if window_id == video.window().id() => {
                video.render(system.pixels(), system.pitch()).unwrap();
            }
            Event::MainEventsCleared => {
                gamepad.handle_events(&mut joypad_state);
                system.run_frame(&joypad_state);

                if let Some(queue) = system.audio_queue() {
                    audio.drain(queue).unwrap();
                }

                let source_size: PhysicalSize<u32> = system.screen_resolution().into();

                if source_size != video.source_size() {
                    video.set_source_size(window_target, source_size);
                }

                video.window().request_redraw();
            }
            _ => (),
        }
    });
}
