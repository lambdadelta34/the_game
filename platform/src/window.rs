use super::APP_NAME;
use gfx_hal::window::Extent2D;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::EventLoop,
    window,
    window::WindowBuilder,
};

const WINDOW_SIZE: [u16; 2] = [512, 512];

#[derive(Debug)]
pub struct Window {
    pub event_loop: EventLoop<()>,
    pub window: window::Window,
    pub surface_extent: Extent2D,
}

impl Window {
    pub fn new() -> Result<Self, ()> {
        let event_loop = EventLoop::new();
        let (logical_window_size, physical_window_size) = {
            let dpi = event_loop.primary_monitor().unwrap().scale_factor();
            let logical: LogicalSize<u32> = WINDOW_SIZE.into();
            let physical: PhysicalSize<u32> = logical.to_physical(dpi);
            (logical, physical)
        };
        let surface_extent = Extent2D {
            width: physical_window_size.width,
            height: physical_window_size.height,
        };
        let window = WindowBuilder::new()
            .with_title(APP_NAME)
            .with_inner_size(logical_window_size)
            .build(&event_loop)
            .expect("Failed to create window");

        Ok(Self {
            window,
            event_loop,
            surface_extent,
        })
    }
}
