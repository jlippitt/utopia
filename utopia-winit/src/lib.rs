pub use utopia::{BiosLoader, DefaultBiosLoader, DefaultMemoryMapper, Error, MemoryMapper};

use audio::AudioController;
use gamepad::Gamepad;
use std::error;
use std::path::PathBuf;
use utopia::{Instance, InstanceOptions, JoypadState, SystemOptions};
use video::VideoController;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget};
use winit::keyboard::Key;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

mod audio;
mod gamepad;
mod keyboard;
mod video;

pub struct AppOptions<T: MemoryMapper + 'static> {
    pub bios_loader: Box<dyn BiosLoader>,
    pub memory_mapper: T,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Sync {
    None,
    Video,
    Audio,
}

pub struct ResetOptions {
    pub rom_path: PathBuf,
    pub rom_data: Vec<u8>,
    pub skip_boot: bool,
    pub full_screen: bool,
    pub sync: Option<Sync>,
    #[cfg(target_arch = "wasm32")]
    pub canvas: HtmlCanvasElement,
}

pub enum AppEvent {
    Reset(ResetOptions),
}

struct AppState {
    video: VideoController,
    audio: AudioController,
    gamepad: Gamepad,
    joypad_state: JoypadState,
    instance: Box<dyn Instance>,
    sync: Sync,
}

impl AppState {
    pub fn new<T: MemoryMapper>(
        window_target: &EventLoopWindowTarget<AppEvent>,
        app_options: &AppOptions<T>,
        reset_options: ResetOptions,
    ) -> Result<Self, Box<dyn error::Error>> {
        let system = utopia::create(SystemOptions {
            system_type: reset_options.rom_path.as_path().try_into()?,
            bios_loader: app_options.bios_loader.as_ref(),
            memory_mapper: &app_options.memory_mapper,
            skip_boot: reset_options.skip_boot,
        })?;

        let source_size: PhysicalSize<u32> = system.default_resolution().into();

        let sync = reset_options.sync.unwrap_or_else(|| {
            if system.default_sample_rate().is_some() {
                Sync::Audio
            } else {
                Sync::Video
            }
        });

        let (video, wgpu_context) = VideoController::create_with_context(
            window_target,
            source_size,
            reset_options.full_screen,
            sync == Sync::Video,
            #[cfg(target_arch = "wasm32")]
            options.canvas,
        )?;

        let instance = system.create_instance(InstanceOptions {
            rom_data: reset_options.rom_data,
            wgpu_context: Some(wgpu_context),
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

pub struct App<T: MemoryMapper + 'static> {
    options: AppOptions<T>,
    event_loop: EventLoop<AppEvent>,
    state: AppState,
}

impl<T: MemoryMapper> App<T> {
    pub fn new(
        options: AppOptions<T>,
        reset_options: ResetOptions,
    ) -> Result<Self, Box<dyn error::Error>> {
        let event_loop = EventLoopBuilder::with_user_event().build()?;
        let state = AppState::new(&event_loop, &options, reset_options)?;

        Ok(Self {
            options,
            event_loop,
            state,
        })
    }

    pub fn proxy(&self) -> EventLoopProxy<AppEvent> {
        self.event_loop.create_proxy()
    }

    pub fn run(mut self) -> Result<(), Box<dyn error::Error>> {
        let event_loop = self.event_loop;

        event_loop.run(move |event, window_target, control_flow| {
            control_flow.set_poll();

            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == ElementState::Pressed {
                            match event.logical_key {
                                Key::Escape => control_flow.set_exit(),
                                Key::F11 => {
                                    self.state
                                        .video
                                        .toggle_full_screen(
                                            self.state.instance.wgpu_context(),
                                            window_target,
                                        )
                                        .unwrap();
                                }
                                _ => (),
                            }
                        }

                        keyboard::handle_input(&mut self.state.joypad_state, event);
                    }
                    WindowEvent::Moved(..) => {
                        self.state.audio.resync();
                    }
                    WindowEvent::Resized(..) => {
                        self.state
                            .video
                            .on_window_size_changed(self.state.instance.wgpu_context())
                            .unwrap();
                        self.state.audio.resync();
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        self.state
                            .video
                            .on_window_size_changed(self.state.instance.wgpu_context())
                            .unwrap();
                        self.state.audio.resync();
                    }
                    _ => (),
                },
                Event::UserEvent(AppEvent::Reset(options)) => {
                    self.state = AppState::new(window_target, &self.options, options).unwrap();
                }
                Event::RedrawRequested(..) => {
                    self.state
                        .video
                        .render(self.state.instance.wgpu_context(), window_target)
                        .unwrap();
                }
                Event::AboutToWait => {
                    self.state
                        .gamepad
                        .handle_events(&mut self.state.joypad_state);

                    let run_frame = if self.state.sync == Sync::Audio {
                        Instant::now() >= self.state.audio.sync_time()
                    } else {
                        true
                    };

                    if run_frame {
                        self.state.instance.run_frame(&self.state.joypad_state);

                        if let Some(queue) = self.state.instance.audio_queue() {
                            self.state.audio.queue_samples(queue);
                        }

                        let source_size: PhysicalSize<u32> =
                            self.state.instance.resolution().into();

                        if source_size != self.state.video.source_size() {
                            self.state.video.set_source_size(
                                self.state.instance.wgpu_context_mut(),
                                window_target,
                                source_size,
                            );
                        }
                    }

                    self.state.video.window().request_redraw();

                    if self.state.sync == Sync::Audio {
                        control_flow.set_wait_until(self.state.audio.sync_time())
                    }
                }
                _ => (),
            }
        })?;

        Ok(())
    }
}
