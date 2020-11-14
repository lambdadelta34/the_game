pub mod graphics;

use common::{time, Time};
use graphics::renderer::Renderer;

pub const APP_NAME: &'static str = "Gamey";

#[derive(Debug)]
pub struct Platform {
    pub graphics: Renderer,
    pub time: Time,
}

impl Platform {
    pub fn start() -> Result<Self, ()> {
        let graphics = Renderer::new()?;

        Ok(Self {
            graphics,
            time: time(),
        })
    }
}

#[no_mangle]
pub fn build_platform() -> Platform {
    Platform::start().unwrap()
}
