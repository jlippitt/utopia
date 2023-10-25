use utopia::JoypadState;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

pub fn handle_input(joypad_state: &mut JoypadState, input: KeyEvent) {
    let JoypadState { buttons, .. } = joypad_state;
    let pressed = input.state == ElementState::Pressed;

    if let PhysicalKey::Code(key_code) = input.physical_key {
        match key_code {
            KeyCode::KeyZ => buttons[0] = pressed,
            KeyCode::KeyX => buttons[1] = pressed,
            KeyCode::KeyA => buttons[2] = pressed,
            KeyCode::KeyS => buttons[3] = pressed,
            KeyCode::KeyD => buttons[4] = pressed,
            KeyCode::KeyC => buttons[5] = pressed,
            KeyCode::Space => buttons[8] = pressed,
            KeyCode::Enter => buttons[9] = pressed,
            KeyCode::ArrowUp => buttons[12] = pressed,
            KeyCode::ArrowDown => buttons[13] = pressed,
            KeyCode::ArrowLeft => buttons[14] = pressed,
            KeyCode::ArrowRight => buttons[15] = pressed,
            _ => (),
        }
    }
}
