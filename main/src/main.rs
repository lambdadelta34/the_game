// use platform::graphics::renderer::Renderer;
use platform::{window::Window, Platform};
use queue::{create_queue, event::Event};
use simple_logger::SimpleLogger;
use winit::event::{Event as E, WindowEvent};
use winit::event_loop::ControlFlow;

// use libloading::Library;

// struct Application(Library);
// impl Application {
//     fn build_platform(&self) -> Platform {
//         unsafe {
//             let f = self.0.get::<fn() -> Platform>(b"build_platform\0").unwrap();
//             f()
//         }
//     }
// }
// const LIB_PATH: &'static str = "./target/debug/libplatform.dylib";

fn main() {
    SimpleLogger::from_env().init().unwrap();
    // let app = Application(Library::new(LIB_PATH).unwrap_or_else(|error| panic!("{}", error)));
    // let mut last_modified = std::fs::metadata(LIB_PATH).unwrap()
    //     .modified().unwrap();
    // TODO: refactor code to support  hot reloading
    // let platform = app.build_platform();
    let (emitter, p) = create_queue(1000);
    let window = Window::new().unwrap();
    let mut platform = Platform::start(&window, p).unwrap();
    let event_loop = window.event_loop;
    let start_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        match event {
            E::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            E::WindowEvent {
                event: WindowEvent::ScaleFactorChanged { .. },
                ..
            } => {}
            E::WindowEvent {
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
                // if matches!(
                //     input.virtual_keycode,
                //     Some(winit::event::VirtualKeyCode::Tab)
                // ) && is_synthetic
                // {
                //     return;
                // }
                // if let Some(key) = input.virtual_keycode {
                //     println!("PRESSED ${:?}", key);
                // }
                // emit kb event
                // let event = Event::new(
                //     event.to_static().unwrap(),
                //     start_time.elapsed().as_secs_f32(),
                // );
                // emitter.push(event).unwrap();
                // world.proccess_events();
                // platform.proccess_events();
            }
            _ => {
                let event = Event::new(
                    event.to_static().unwrap(),
                    start_time.elapsed().as_secs_f32(),
                );
                emitter.push(event).unwrap();
                platform.proccess_events();
            }
        };
    });
}
