use clap::Parser;
use std::error::Error;
use std::fs;
use winit::dpi::{PhysicalPosition, Size};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

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

fn main() {
    let args = Args::parse();

    let rom_data = fs::read(&args.rom_path).unwrap();

    let system = utopia::create(
        rom_data,
        &args.rom_path,
        &utopia::Options {
            bios_loader: BiosLoader,
            memory_mapper: MemoryMapper,
            skip_boot: args.skip_boot,
        },
    )
    .unwrap();

    let event_loop = EventLoop::new();

    let inner_size = system.screen_resolution().into();

    let window = WindowBuilder::new()
        .with_title("Utopia")
        .with_inner_size(Size::Physical(inner_size))
        .with_resizable(false)
        .build(&event_loop)
        .unwrap();

    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let pos_x = (monitor_size.width - inner_size.width) / 2;
        let pos_y = (monitor_size.height - inner_size.height) / 2;
        window.set_outer_position(PhysicalPosition::new(pos_x, pos_y));
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    });
}
