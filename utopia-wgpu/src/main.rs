use clap::Parser;
use gamepad::Gamepad;
use std::error::Error;
use utopia::JoypadState;
use video::VideoController;
use winit::dpi::{PhysicalSize, Size};
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, WindowBuilder};

mod gamepad;
mod geometry;
mod keyboard;
mod video;

struct BiosLoader;

impl utopia::BiosLoader for BiosLoader {
    fn load(&self, _name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        todo!("BIOS Loader");
    }
}

struct MemoryMapper;

impl utopia::MemoryMapper for MemoryMapper {
    type Mapped = Vec<u8>;

    fn open(&self, len: usize, _battery_backed: bool) -> Result<Vec<u8>, Box<dyn Error>> {
        Ok(vec![0; len])
    }
}

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    rom_path: String,

    #[arg(short, long)]
    full_screen: bool,

    #[arg(short, long)]
    skip_boot: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = std::fs::read(&args.rom_path)?;

    let mut system = utopia::create(
        rom_data,
        &args.rom_path,
        &utopia::Options {
            bios_loader: BiosLoader,
            memory_mapper: MemoryMapper,
            skip_boot: args.skip_boot,
        },
    )?;

    let event_loop = EventLoop::new();

    let monitor = event_loop.available_monitors().next().unwrap();

    let source_size: PhysicalSize<u32> = system.screen_resolution().into();

    let window_builder = WindowBuilder::new().with_title("Utopia");

    let (target_size, clip_rect, window_builder) = if args.full_screen {
        let default_video_mode = monitor.video_modes().next().unwrap();
        let video_mode = geometry::best_fit(source_size, monitor).unwrap_or(default_video_mode);
        let clip_rect = geometry::clip(source_size, video_mode.size());

        let window_builder =
            window_builder.with_fullscreen(Some(Fullscreen::Exclusive(video_mode)));

        (source_size, Some(clip_rect), window_builder)
    } else {
        let monitor_size = monitor.size();
        let target_size = geometry::upscale(source_size, monitor_size);
        let position = geometry::center(target_size, monitor_size);

        let window_builder = window_builder
            .with_inner_size(Size::Physical(target_size))
            .with_position(position);

        (target_size, None, window_builder)
    };

    let window = window_builder.build(&event_loop)?;

    let mut video = VideoController::new(window, source_size, target_size, clip_rect)?;

    let mut gamepad = Gamepad::new()?;

    let mut joypad_state = JoypadState::default();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, window_id } if window_id == video.window().id() => {
                match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => control_flow.set_exit(),
                        _ => keyboard::handle_input(&mut joypad_state, input),
                    },
                    WindowEvent::Resized(..) => {
                        video.on_window_size_changed().unwrap();
                    }
                    WindowEvent::ScaleFactorChanged { .. } => {
                        video.on_window_size_changed().unwrap();
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(window_id) if window_id == video.window().id() => {
                video.render(system.pixels(), system.pitch()).unwrap();
            }
            Event::MainEventsCleared => {
                gamepad.handle_events(&mut joypad_state);
                system.run_frame(&joypad_state);
                video.window().request_redraw();
            }
            _ => (),
        }
    });
}
