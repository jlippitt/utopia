pub use utopia::{BiosLoader, Error, MemoryMapper};

use audio::AudioController;
use gamepad::Gamepad;
use std::error;
use std::path::PathBuf;
use std::time::Instant;
use utopia::{InstanceOptions, JoypadState, SystemOptions};
use video::VideoController;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;

mod audio;
mod gamepad;
mod keyboard;
mod video;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Sync {
    None,
    Video,
    Audio,
}

pub struct UtopiaWinitOptions<T: MemoryMapper + 'static> {
    pub rom_path: PathBuf,
    pub rom_data: Vec<u8>,
    pub bios_loader: Box<dyn BiosLoader>,
    pub memory_mapper: T,
    pub skip_boot: bool,
    pub full_screen: bool,
    pub sync: Option<Sync>,
}

pub fn run<T: MemoryMapper>(options: UtopiaWinitOptions<T>) -> Result<(), Box<dyn error::Error>> {
    let system = utopia::create(SystemOptions {
        system_type: options.rom_path.as_path().try_into()?,
        bios_loader: options.bios_loader,
        memory_mapper: options.memory_mapper,
        skip_boot: options.skip_boot,
    })?;

    let event_loop = EventLoop::new();

    let source_size: PhysicalSize<u32> = system.default_resolution().into();

    let sync = options.sync.unwrap_or_else(|| {
        if system.default_sample_rate().is_some() {
            Sync::Audio
        } else {
            Sync::Video
        }
    });

    let (mut video, wgpu_context) = VideoController::create_with_context(
        &event_loop,
        source_size,
        options.full_screen,
        sync == Sync::Video,
    )?;

    let mut instance = system.create_instance(InstanceOptions {
        rom_data: options.rom_data,
        wgpu_context: Some(wgpu_context),
    })?;

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
                                video
                                    .toggle_full_screen(instance.wgpu_context(), window_target)
                                    .unwrap();
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
                    video
                        .on_window_size_changed(instance.wgpu_context())
                        .unwrap();
                    audio.resync();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    video
                        .on_window_size_changed(instance.wgpu_context())
                        .unwrap();
                    audio.resync();
                }
                _ => (),
            },
            Event::RedrawRequested(window_id) if window_id == video.window().id() => {
                video
                    .render(instance.wgpu_context(), window_target)
                    .unwrap();
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
                        video.set_source_size(
                            instance.wgpu_context_mut(),
                            window_target,
                            source_size,
                        );
                    }

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
