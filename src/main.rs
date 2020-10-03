mod platform;

use platform::core::Platform;
use simple_logger::SimpleLogger;
use winit::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};
use platform::graphics::window::init_window::WinitState;

fn main() {
    // init vulkan
    // run main loop
    SimpleLogger::from_env().init().unwrap();

    let plt = Platform::start().unwrap();
    let su = plt.surface;
    let e = su.event_loop;
    let mut v = su.vulkan;

    e.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                // recreate_swapchain = true;
                // emit window resized event
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                println!("PRESSED ${:?}", input);
                // emit kb event
            }
            Event::MainEventsCleared => {
                // run main loop
                WinitState::draw(&mut v);
            }
            _ => (),
        });
}
