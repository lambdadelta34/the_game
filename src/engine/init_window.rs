use winit::{
    error::OsError,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub const WINDOW_NAME: &str = "Hello";

#[derive(Debug)]
pub struct WinitState {
    pub event_loop: EventLoop<()>,
    pub window: Window,
}

impl WinitState {
    /// Constructs a new `EventLoop` and `Window` pair.
    ///
    /// The specified title is used, other elements are default.
    /// ## Failure
    /// It's possible for the window creation to fail. This is unlikely.
    pub fn new(title: String) -> Result<Self, OsError> {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title(title).build(&event_loop)?;
        Ok(Self { event_loop, window })
    }
}

impl Default for WinitState {
    /// ## Panics
    /// If a `OsError` occurs.
    fn default() -> Self {
        Self::new(WINDOW_NAME.to_string()).expect("Could not create a window")
    }
}
