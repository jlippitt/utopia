use sdl2::keyboard::Scancode;
use utopia::JoypadState;

pub struct Joypad {
    state: JoypadState,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            state: Default::default(),
        }
    }

    pub fn state(&self) -> &JoypadState {
        &self.state
    }

    pub fn key_event(&mut self, scancode: Scancode, pressed: bool) {
        match scancode {
            Scancode::Up => self.state.up = pressed,
            Scancode::Down => self.state.down = pressed,
            Scancode::Left => self.state.left = pressed,
            Scancode::Right => self.state.right = pressed,
            Scancode::X => self.state.a = pressed,
            Scancode::Z => self.state.b = pressed,
            Scancode::S => self.state.x = pressed,
            Scancode::A => self.state.y = pressed,
            Scancode::D => self.state.l = pressed,
            Scancode::C => self.state.r = pressed,
            Scancode::Space => self.state.select = pressed,
            Scancode::Return => self.state.start = pressed,
            _ => (),
        }
    }
}
