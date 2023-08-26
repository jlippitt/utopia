use audio::AudioController;
use bios::BiosLoader;
use clap::{Parser, ValueEnum};
use gamepad::Gamepad;
use mmap::MemoryMapper;
use std::error::Error;
use std::path::Path;
use std::time::Instant;
use utopia::{InstanceOptions, JoypadState, SystemOptions};
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

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum Sync {
    None,
    Video,
    Audio,
}

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

    #[arg(value_enum, long)]
    sync: Option<Sync>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let _log = log::init()?;

    let rom_data = std::fs::read(&args.rom_path)?;

    let system = utopia::create(SystemOptions {
        system_type: Path::new(&args.rom_path).try_into()?,
        bios_loader: BiosLoader::new(args.bios_path.unwrap_or(args.rom_path.clone()).into()),
        memory_mapper: MemoryMapper::new(args.rom_path.clone().into()),
        skip_boot: args.skip_boot,
    })?;

    let event_loop = EventLoop::new();

    let source_size: PhysicalSize<u32> = system.default_resolution().into();

    let sync = args.sync.unwrap_or_else(|| {
        if system.default_sample_rate().is_some() {
            Sync::Audio
        } else {
            Sync::Video
        }
    });

    let mut video = VideoController::new(
        &event_loop,
        source_size,
        args.full_screen,
        sync == Sync::Video,
    )?;

    let mut instance = system.create_instance(InstanceOptions { rom_data })?;

    let mut audio = AudioController::new(instance.sample_rate())?;

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
                WindowEvent::Moved(..) => {
                    audio.resync();
                }
                WindowEvent::Resized(..) => {
                    video.on_window_size_changed().unwrap();
                    audio.resync();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    video.on_window_size_changed().unwrap();
                    audio.resync();
                }
                WindowEvent::Destroyed => {
                    video.on_target_changed(window_target);
                    audio.resync();
                }
                _ => (),
            },
            Event::RedrawRequested(window_id) if window_id == video.window().id() => {
                video.render().unwrap();
            }
            Event::RedrawEventsCleared => {
                if sync == Sync::Audio {
                    control_flow.set_wait_until(audio.sync_time())
                }
            }
            Event::MainEventsCleared => {
                gamepad.handle_events(&mut joypad_state);

                let run_frame = if sync == Sync::Audio {
                    Instant::now() >= audio.sync_time()
                } else {
                    true
                };

                if run_frame {
                    instance.run_frame(&joypad_state);

                    if let Some(queue) = instance.audio_queue() {
                        audio.queue_samples(queue);
                    }

                    let source_size: PhysicalSize<u32> = instance.resolution().into();

                    if source_size != video.source_size() {
                        video.set_source_size(window_target, source_size);
                    }

                    video
                        .update_texture(instance.pixels(), instance.pitch())
                        .unwrap();

                    video.window().request_redraw();
                }

                if sync == Sync::Audio {
                    control_flow.set_wait_until(audio.sync_time())
                }
            }
            _ => (),
        }
    });
}
