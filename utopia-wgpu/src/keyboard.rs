use utopia::JoypadState;
use winit::event::{ElementState, KeyboardInput, ScanCode};

struct PhysicalKey;

impl PhysicalKey {
    const Z: ScanCode = 0x2c;
    const X: ScanCode = 0x2d;
    const A: ScanCode = 0x1e;
    const S: ScanCode = 0x1f;
    const D: ScanCode = 0x20;
    const C: ScanCode = 0x2e;
    const SPACE: ScanCode = 0x39;
    const RETURN: ScanCode = 0x1c;
    const UP: ScanCode = 0x67;
    const DOWN: ScanCode = 0x6c;
    const LEFT: ScanCode = 0x69;
    const RIGHT: ScanCode = 0x6a;
}

pub fn handle_input(joypad_state: &mut JoypadState, input: KeyboardInput) {
    let JoypadState { buttons, .. } = joypad_state;
    let pressed = input.state == ElementState::Pressed;

    match input.scancode {
        PhysicalKey::Z => buttons[0] = pressed,
        PhysicalKey::X => buttons[1] = pressed,
        PhysicalKey::A => buttons[2] = pressed,
        PhysicalKey::S => buttons[3] = pressed,
        PhysicalKey::D => buttons[4] = pressed,
        PhysicalKey::C => buttons[5] = pressed,
        PhysicalKey::SPACE => buttons[8] = pressed,
        PhysicalKey::RETURN => buttons[9] = pressed,
        PhysicalKey::UP => buttons[12] = pressed,
        PhysicalKey::DOWN => buttons[13] = pressed,
        PhysicalKey::LEFT => buttons[14] = pressed,
        PhysicalKey::RIGHT => buttons[15] = pressed,
        _ => (),
    }
}
