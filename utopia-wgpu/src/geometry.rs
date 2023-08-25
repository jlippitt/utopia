use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::monitor::{MonitorHandle, VideoMode};

// We should be running at approx 60 FPS or more
const MIN_REFRESH_RATE: u32 = 59900;

pub fn upscale(source: PhysicalSize<u32>, target: PhysicalSize<u32>) -> PhysicalSize<u32> {
    let scale = scale_factor(source, target);
    (source.width * scale, source.height * scale).into()
}

pub fn center(source: PhysicalSize<u32>, target: PhysicalSize<u32>) -> PhysicalPosition<u32> {
    let pos_x = (target.width - source.width) / 2;
    let pos_y = (target.height - source.height) / 2;
    (pos_x, pos_y).into()
}

pub fn best_fit(source: PhysicalSize<u32>, monitor: MonitorHandle) -> Option<VideoMode> {
    let mut best_mode: Option<VideoMode> = None;
    let mut best_scale = 0;

    for video_mode in monitor.video_modes() {
        if video_mode.bit_depth() < 24 {
            continue;
        }

        if video_mode.refresh_rate_millihertz() < MIN_REFRESH_RATE {
            continue;
        }

        let scale = scale_factor(source, video_mode.size());

        if scale < best_scale {
            continue;
        }

        if scale == best_scale {
            if let Some(best_mode) = &best_mode {
                if video_mode.refresh_rate_millihertz() <= best_mode.refresh_rate_millihertz() {
                    continue;
                }
            }
        }

        best_mode = Some(video_mode);
        best_scale = scale;
    }

    best_mode
}

pub fn clip(source: PhysicalSize<u32>, target: PhysicalSize<u32>) -> [[f32; 2]; 4] {
    let PhysicalSize { width, height } = upscale(source, target);

    let offset_x = (target.width - width) / 2;
    let offset_y = (target.height - height) / 2;

    let left = (offset_x as f32 / target.width as f32) * 2.0 - 1.0;
    let right = ((offset_x + width) as f32 / target.width as f32) * 2.0 - 1.0;
    let top = 1.0 - (offset_y as f32 / target.height as f32) * 2.0;
    let bottom = 1.0 - ((offset_y + height) as f32 / target.height as f32) * 2.0;

    [[left, top], [left, bottom], [right, top], [right, bottom]]
}

fn scale_factor(source: PhysicalSize<u32>, target: PhysicalSize<u32>) -> u32 {
    let width_ratio = target.width / source.width;
    let height_ratio = target.height / source.height;
    width_ratio.min(height_ratio)
}
