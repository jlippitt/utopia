use gilrs::{Axis, Button, Error, EventType, Gilrs};
use utopia::JoypadState;

pub struct Gamepad {
    gilrs: Gilrs,
}

impl Gamepad {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            gilrs: Gilrs::new()?,
        })
    }

    pub fn handle_events(&mut self, joypad_state: &mut JoypadState) {
        while let Some(event) = self.gilrs.next_event() {
            match event.event {
                EventType::ButtonPressed(button, ..) => update_button(joypad_state, button, true),
                EventType::ButtonReleased(button, ..) => update_button(joypad_state, button, false),
                EventType::AxisChanged(axis, value, ..) => update_axis(joypad_state, axis, value),
                _ => (),
            }
        }
    }
}

fn update_button(joypad_state: &mut JoypadState, button: Button, pressed: bool) {
    let JoypadState { buttons, .. } = joypad_state;

    match button {
        Button::South => buttons[0] = pressed,
        Button::East => buttons[1] = pressed,
        // North and west appear swapped for me, though this may just be an Xbox controller issue?
        Button::North => buttons[2] = pressed,
        Button::West => buttons[3] = pressed,
        Button::LeftTrigger => buttons[4] = pressed,
        Button::RightTrigger => buttons[5] = pressed,
        Button::Select => buttons[8] = pressed,
        Button::Start => buttons[9] = pressed,
        Button::DPadUp => buttons[12] = pressed,
        Button::DPadDown => buttons[13] = pressed,
        Button::DPadLeft => buttons[14] = pressed,
        Button::DPadRight => buttons[15] = pressed,
        _ => (),
    }
}

fn update_axis(joypad_state: &mut JoypadState, axis: Axis, value: f32) {
    let JoypadState { axes, buttons } = joypad_state;

    match axis {
        Axis::LeftStickX => axes[0] = (value * i32::MAX as f32) as i32,
        Axis::LeftStickY => axes[1] = (value * i32::MAX as f32) as i32,
        Axis::RightStickX => axes[2] = (value * i32::MAX as f32) as i32,
        Axis::RightStickY => axes[3] = (value * i32::MAX as f32) as i32,
        Axis::LeftZ => buttons[6] = value >= -0.75,
        Axis::RightZ => buttons[7] = value >= -0.75,
        _ => (),
    }
}
