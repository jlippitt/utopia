use clap::Parser;
use std::error::Error;
use std::fs;
use std::time::{Duration, Instant};
use video::VideoController;
use winit::dpi::{PhysicalPosition, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

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
    skip_boot: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let rom_data = fs::read(&args.rom_path)?;

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

    let inner_size = system.screen_resolution().into();

    let window = WindowBuilder::new()
        .with_title("Utopia")
        .with_inner_size(Size::Physical(inner_size))
        .with_resizable(false)
        .build(&event_loop)?;

    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let pos_x = (monitor_size.width - inner_size.width) / 2;
        let pos_y = (monitor_size.height - inner_size.height) / 2;
        window.set_outer_position(PhysicalPosition::new(pos_x, pos_y));
    }

    let mut video = VideoController::new(window, inner_size)?;

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == video.window().id() => control_flow.set_exit(),
            Event::RedrawRequested(window_id) if window_id == video.window().id() => {
                video.render(system.pixels(), system.pitch()).unwrap();
            }
            Event::MainEventsCleared => {
                system.run_frame(&Default::default());
                video.window().request_redraw();
            }
            _ => (),
        }
    });
}
