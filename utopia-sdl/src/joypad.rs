use sdl2::controller::{Button, GameController};
use sdl2::keyboard::Scancode;
use sdl2::{GameControllerSubsystem, Sdl};
use std::error::Error;
use tracing::{info, warn};
use utopia::JoypadState;

pub struct Joypad {
    subsystem: GameControllerSubsystem,
    state: JoypadState,
    controller: Option<GameController>,
}

impl Joypad {
    pub fn new(sdl_context: &Sdl) -> Result<Self, Box<dyn Error>> {
        let subsystem = sdl_context.game_controller()?;

        let controller = 'controller: {
            for id in 0..subsystem.num_joysticks()? {
                if !subsystem.is_game_controller(id) {
                    continue;
                }

                let controller = open_controller(&subsystem, id);

                if controller.is_some() {
                    break 'controller controller;
                }
            }

            None
        };

        Ok(Self {
            subsystem,
            state: Default::default(),
            controller,
        })
    }

    pub fn state(&self) -> &JoypadState {
        &self.state
    }

    pub fn key_event(&mut self, scancode: Scancode, pressed: bool) {
        let JoypadState { buttons, .. } = &mut self.state;

        match scancode {
            Scancode::Z => buttons[0] = pressed,
            Scancode::X => buttons[1] = pressed,
            Scancode::A => buttons[2] = pressed,
            Scancode::S => buttons[3] = pressed,
            Scancode::D => buttons[4] = pressed,
            Scancode::C => buttons[5] = pressed,
            Scancode::Space => buttons[8] = pressed,
            Scancode::Return => buttons[9] = pressed,
            Scancode::Up => buttons[12] = pressed,
            Scancode::Down => buttons[13] = pressed,
            Scancode::Left => buttons[14] = pressed,
            Scancode::Right => buttons[15] = pressed,
            _ => (),
        }
    }

    pub fn button_event(&mut self, id: u32, button: Button, pressed: bool) {
        if !self.is_controller_connected(id) {
            return;
        }

        let JoypadState { buttons, .. } = &mut self.state;

        match button {
            Button::A => buttons[0] = pressed,
            Button::B => buttons[1] = pressed,
            Button::X => buttons[2] = pressed,
            Button::Y => buttons[3] = pressed,
            Button::LeftShoulder => buttons[4] = pressed,
            Button::RightShoulder => buttons[5] = pressed,
            Button::Back => buttons[8] = pressed,
            Button::Start => buttons[9] = pressed,
            Button::DPadUp => buttons[12] = pressed,
            Button::DPadDown => buttons[13] = pressed,
            Button::DPadLeft => buttons[14] = pressed,
            Button::DPadRight => buttons[15] = pressed,
            _ => (),
        }
    }

    pub fn add_controller(&mut self, id: u32) {
        if self.controller.is_none() {
            self.controller = open_controller(&self.subsystem, id);
        }
    }

    pub fn remove_controller(&mut self, id: u32) {
        if self.is_controller_connected(id) {
            info!("Game controller disconnected");
            self.controller = None;
        }
    }

    fn is_controller_connected(&self, id: u32) -> bool {
        self.controller
            .as_ref()
            .is_some_and(|c| c.instance_id() == id)
    }
}

fn open_controller(subsystem: &GameControllerSubsystem, id: u32) -> Option<GameController> {
    match subsystem.open(id) {
        Ok(controller) => {
            info!("Game controller connected");
            Some(controller)
        }
        Err(error) => {
            warn!("Failed to open game controller {}: {}", id, error);
            None
        }
    }
}
