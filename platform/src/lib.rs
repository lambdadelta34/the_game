pub mod graphics;
use graphics::renderer::Renderer;

pub const APP_NAME: &'static str = "Gamey";

#[derive(Debug)]
pub struct Platform {
    pub graphics: Renderer,
}

impl Platform {
    pub fn start() -> Result<Self, ()> {
        let graphics = Renderer::new()?;

        Ok(Self { graphics })
    }
}

#[no_mangle]
pub fn build_platform() -> Platform {
   Platform::start().unwrap()
}
