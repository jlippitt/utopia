pub use utopia::{BiosLoader, DefaultBiosLoader, DefaultMemoryMapper, Error, MemoryMapper};

use audio::AudioController;
use gamepad::Gamepad;
use std::error;
use std::path::PathBuf;
use std::rc::Rc;
use utopia::{Instance, InstanceOptions, JoypadState, SystemOptions};
use video::VideoController;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
use winit::keyboard::Key;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(target_arch = "wasm32")]
use winit::platform::web::EventLoopExtWebSys;

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

#[derive(Clone, Debug)]
pub struct ResetOptions<T: MemoryMapper> {
    pub bios_loader: Rc<dyn BiosLoader>,
    pub memory_mapper: T,
    pub rom_path: PathBuf,
    pub rom_data: Vec<u8>,
    pub skip_boot: bool,
    pub full_screen: bool,
    pub sync: Option<Sync>,
    #[cfg(target_arch = "wasm32")]
    pub canvas: HtmlCanvasElement,
}

struct ResetState {
    video: VideoController,
    audio: AudioController,
    gamepad: Gamepad,
    joypad_state: JoypadState,
    instance: Box<dyn Instance>,
    sync: Sync,
}

impl ResetState {
    pub fn new<T: MemoryMapper>(
        window_target: &EventLoopWindowTarget<AppEvent<T>>,
        options: ResetOptions<T>,
    ) -> Result<Self, Box<dyn error::Error>> {
        let system = utopia::create(SystemOptions {
            system_type: options.rom_path.as_path().try_into()?,
            bios_loader: options.bios_loader.as_ref(),
            memory_mapper: &options.memory_mapper,
            skip_boot: options.skip_boot,
        })?;

        let source_size: PhysicalSize<u32> = system.default_resolution().into();

        let sync = options.sync.unwrap_or_else(|| {
            if system.default_sample_rate().is_some() {
                Sync::Audio
            } else {
                Sync::Video
            }
        });

        let video = VideoController::create_with_context(
            window_target,
            source_size,
            options.full_screen,
            sync == Sync::Video,
            #[cfg(target_arch = "wasm32")]
            options.canvas,
        )?;

        let instance = system.create_instance(InstanceOptions {
            rom_data: options.rom_data,
            wgpu_context: Some(video.ctx().clone()),
        })?;

        let mut audio = AudioController::new(instance.sample_rate())?;

        let gamepad = Gamepad::new()?;

        let joypad_state = JoypadState::default();

        audio.resume()?;

        Ok(Self {
            video,
            audio,
            gamepad,
            joypad_state,
            instance,
            sync,
        })
    }
}

#[derive(Clone, Debug)]
pub enum AppEvent<T: MemoryMapper> {
    Reset(ResetOptions<T>),
    UpdateViewport,
}

#[derive(Default)]
pub struct App<T: MemoryMapper + 'static> {
    proxy: Option<EventLoopProxy<AppEvent<T>>>,
}

impl<T: MemoryMapper> App<T> {
    pub fn new() -> Self {
        Self { proxy: None }
    }

    pub fn reset(&mut self, options: ResetOptions<T>) -> Result<(), Box<dyn error::Error>> {
        if let Some(proxy) = &self.proxy {
            proxy.send_event(AppEvent::Reset(options))?;
        } else {
            start_event_loop(&mut self.proxy, options)?;
        }

        Ok(())
    }

    pub fn update_viewport(&mut self) -> Result<(), Box<dyn error::Error>> {
        if let Some(proxy) = &self.proxy {
            proxy.send_event(AppEvent::UpdateViewport)?;
        }

        Ok(())
    }
}

fn start_event_loop<T: MemoryMapper>(
    proxy: &mut Option<EventLoopProxy<AppEvent<T>>>,
    options: ResetOptions<T>,
) -> Result<(), Box<dyn error::Error>> {
    let event_loop = EventLoopBuilder::with_user_event().build()?;

    *proxy = Some(event_loop.create_proxy());

    let mut state = ResetState::new(&event_loop, options)?;

    let event_loop_body = move |event, window_target: &_, control_flow: &mut ControlFlow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => control_flow.set_exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        match event.logical_key {
                            Key::Escape => control_flow.set_exit(),
                            Key::F11 => {
                                state.video.toggle_full_screen(window_target).unwrap();
                            }
                            _ => (),
                        }
                    }

                    keyboard::handle_input(&mut state.joypad_state, event);
                }
                WindowEvent::Moved(..) => {
                    state.audio.resync();
                }
                WindowEvent::Resized(..) => {
                    state.video.on_window_size_changed().unwrap();
                    state.audio.resync();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    state.video.on_window_size_changed().unwrap();
                    state.audio.resync();
                }
                _ => (),
            },
            Event::UserEvent(AppEvent::Reset(options)) => {
                state = ResetState::new(window_target, options).unwrap();
            }
            Event::UserEvent(AppEvent::UpdateViewport) => {
                state.video.update_viewport(window_target)
            }
            Event::RedrawRequested(..) => {
                state.video.render(window_target).unwrap();
            }
            Event::AboutToWait => {
                state.gamepad.handle_events(&mut state.joypad_state);

                let run_frame = if state.sync == Sync::Audio {
                    Instant::now() >= state.audio.sync_time()
                } else {
                    true
                };

                if run_frame {
                    state.instance.run_frame(&state.joypad_state);

                    if let Some(queue) = state.instance.audio_queue() {
                        state.audio.queue_samples(queue);
                    }

                    let source_size: PhysicalSize<u32> = state.instance.resolution().into();

                    if source_size != state.video.source_size() {
                        state.video.set_source_size(window_target, source_size);
                    }
                }

                state.video.window().request_redraw();

                if state.sync == Sync::Audio {
                    control_flow.set_wait_until(state.audio.sync_time())
                }
            }
            _ => (),
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    event_loop.run(event_loop_body)?;

    #[cfg(target_arch = "wasm32")]
    event_loop.spawn(event_loop_body);

    Ok(())
}
