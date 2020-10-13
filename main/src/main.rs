use platform::core::Platform;
use platform::graphics::renderer::Renderer;
use simple_logger::SimpleLogger;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

// const LIB_PATH: &'static str = "../../target/debug/libplatform.dylib";

fn main() {
    SimpleLogger::from_env().init().unwrap();
    // let mut last_modified = std::fs::metadata(LIB_PATH).unwrap().modified().unwrap();
    // println!("${:?}", &last_modified);
    let platform = Platform::start().unwrap();
    let graphics = platform.graphics;
    let Renderer {
        window,
        mut resources,
    } = graphics;
    let event_loop = window.event_loop;
    let mut extent = window.surface_extent;

    let start_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(dimension),
            ..
        } => {
            println!("RESIZED TO ${:?}", dimension);
            // surface_extent = Extent2D {
            //     width: dimension.width,
            //     height: dimension.height,
            // };
            // emit window resized event
        }
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            println!("PRESSED ${:?}", input);
            // emit kb event
        }
        // redraw continiously
        Event::MainEventsCleared => {
            Renderer::draw(&mut resources, &mut extent, start_time);
        }
        Event::RedrawRequested(_) => {} // redraw after event
        _ => (),
    });
}
