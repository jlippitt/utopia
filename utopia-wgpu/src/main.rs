use clap::Parser;
use std::error::Error;
use video::VideoController;
use winit::dpi::{PhysicalPosition, PhysicalSize, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Fullscreen, WindowBuilder};

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

    let inner_size: PhysicalSize<u32> = system.screen_resolution().into();

    let window_builder = WindowBuilder::new()
        .with_title("Utopia")
        .with_inner_size(Size::Physical(inner_size));

    let window_builder = if args.full_screen {
        let video_mode = monitor.video_modes().next().unwrap();
        window_builder.with_fullscreen(Some(Fullscreen::Exclusive(video_mode)))
    } else {
        let monitor_size = monitor.size();
        let pos_x = (monitor_size.width - inner_size.width) / 2;
        let pos_y = (monitor_size.height - inner_size.height) / 2;
        window_builder.with_position(PhysicalPosition::new(pos_x, pos_y))
    };

    let window = window_builder.build(&event_loop)?;

    let mut video = VideoController::new(window, inner_size)?;

    let mut joypad_state = utopia::JoypadState::default();

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent { event, window_id } if window_id == video.window().id() => {
                match event {
                    WindowEvent::CloseRequested => control_flow.set_exit(),
                    WindowEvent::KeyboardInput { input, .. } => {
                        keyboard::handle_input(&mut joypad_state, input)
                    }
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
                system.run_frame(&joypad_state);
                video.window().request_redraw();
            }
            _ => (),
        }
    });
}
