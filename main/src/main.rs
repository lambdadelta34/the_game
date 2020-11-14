use platform::graphics::renderer::Renderer;
use platform::Platform;
use simple_logger::SimpleLogger;
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use libloading::Library;

struct Application(Library);
impl Application {
    fn build_platform(&self) -> Platform {
        unsafe {
            let f = self.0.get::<fn() -> Platform>(b"build_platform\0").unwrap();
            f()
        }
    }
}
const LIB_PATH: &'static str = "./target/debug/libplatform.dylib";

fn main() {
    SimpleLogger::from_env().init().unwrap();
    let app = Application(Library::new(LIB_PATH).unwrap_or_else(|error| panic!("{}", error)));

    // let mut last_modified = std::fs::metadata(LIB_PATH).unwrap()
    //     .modified().unwrap();
    // let platform = Platform::start().unwrap();
    // TODO: refactor code to support  hot reloading
    let platform = app.build_platform();
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
            event:
                WindowEvent::KeyboardInput {
                    input,
                    is_synthetic,
                    ..
                },
            ..
        } => {
            // Ignore synthetic tab presses so that we don't get tabs when alt-tabbing back
            // into the window
            if matches!(
                input.virtual_keycode,
                Some(winit::event::VirtualKeyCode::Tab)
            ) && is_synthetic
            {
                return;
            }
            if let Some(key) = input.virtual_keycode {
                // self.events.push(Event::InputUpdate(
                //     *game_input,
                //     input.state == winit::event::ElementState::Pressed,
                // ))
                println!("PRESSED ${:?}", key);
            }
            // emit kb event
        }
        // redraw continiously
        Event::MainEventsCleared => {
            Renderer::draw(&mut resources, &mut extent, start_time);
        }
        // redraw after event
        Event::RedrawRequested(_) => {}
        _ => (),
    });
}
