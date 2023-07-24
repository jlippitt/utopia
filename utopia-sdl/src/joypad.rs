use sdl2::controller::{Button, GameController};
use sdl2::keyboard::Scancode;
use sdl2::Sdl;
use std::error::Error;
use tracing::{info, warn};
use utopia::JoypadState;

pub struct Joypad {
    state: JoypadState,
    controller: Option<GameController>,
}

impl Joypad {
    pub fn new(sdl_context: &Sdl) -> Result<Self, Box<dyn Error>> {
        let game_controller = sdl_context.game_controller()?;

        let controller = 'controller: {
            for id in 0..game_controller.num_joysticks()? {
                if !game_controller.is_game_controller(id) {
                    continue;
                }

                match game_controller.open(id) {
                    Ok(controller) => {
                        info!("Game controller connected");
                        break 'controller Some(controller);
                    }
                    Err(error) => warn!("Failed to open game controller {}: {}", id, error),
                }
            }

            None
        };

        Ok(Self {
            state: Default::default(),
            controller,
        })
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

    pub fn button_event(&mut self, id: u32, button: Button, pressed: bool) {
        if !self
            .controller
            .as_ref()
            .is_some_and(|c| c.instance_id() == id)
        {
            return;
        }

        match button {
            Button::DPadUp => self.state.up = pressed,
            Button::DPadDown => self.state.down = pressed,
            Button::DPadLeft => self.state.left = pressed,
            Button::DPadRight => self.state.right = pressed,
            Button::B => self.state.a = pressed,
            Button::A => self.state.b = pressed,
            Button::Y => self.state.x = pressed,
            Button::X => self.state.y = pressed,
            Button::LeftShoulder => self.state.l = pressed,
            Button::RightShoulder => self.state.r = pressed,
            Button::Back => self.state.select = pressed,
            Button::Start => self.state.start = pressed,
            _ => (),
        }
    }
}
