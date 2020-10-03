use super::graphics::window::init_window;

use init_window::WinitState;
use vulkano_win::CreationError;

#[derive(Debug)]
pub struct Platform {
    pub surface: WinitState,
}

impl Platform {
    pub fn start() -> Result<Self, CreationError> {
        let surface = WinitState::new()?;

        Ok(Self { surface })
    }
}
