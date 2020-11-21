pub mod graphics;
use graphics::renderer::Renderer;
pub mod window;
use queue::{event::Event, receiver::Receiver};
use window::Window;
use winit::event::Event as WEvent;

pub const APP_NAME: &'static str = "Gamey";

#[derive(Debug)]
pub struct Platform<'a> {
    pub graphics: Renderer<'a>,
}

impl<'a> Platform<'a> {
    pub fn start(window: &Window, events: Receiver<Event<WEvent<'a, ()>>>) -> Result<Self, ()> {
        let graphics = Renderer::new(&window, events)?;

        Ok(Self { graphics })
    }

    pub fn proccess_events(&mut self) {
        &self.graphics.update();
    }
}

#[no_mangle]
pub fn build_platform<'a>(
    window: &Window,
    events: Receiver<Event<WEvent<'a, ()>>>,
) -> Platform<'a> {
    Platform::start(window, events).unwrap()
}
