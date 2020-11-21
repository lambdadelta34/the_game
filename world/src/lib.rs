use queue::{event::Event, receiver::Receiver};

// TODO: remove winit dependency
use winit::event::{Event as WEvent, VirtualKeyCode, WindowEvent};

#[derive(Debug)]
pub struct WorldState {
    pub player: (f32, f32),
}
impl WorldState {
    pub fn new() -> Self {
        Self {
            player: (-0.5, -0.5),
        }
    }
}

#[derive(Debug)]
pub struct World<'a> {
    pub events: Receiver<Event<WEvent<'a, ()>>>,
}
impl<'a> World<'a> {
    pub fn start(events: Receiver<Event<WEvent<'a, ()>>>) -> Self {
        Self { events }
    }

    pub fn proccess_events(&self, world: &mut WorldState) {
        let event = self.events.try_recv().unwrap();
        match event.payload {
            WEvent::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input,
                        is_synthetic,
                        ..
                    },
                ..
            } => {
                // ignore synthetic tab presses so that we don't get tabs when alt-tabbing back into the window
                if matches!(input.virtual_keycode, Some(VirtualKeyCode::Tab)) && is_synthetic {
                    return;
                }
                if let Some(key) = input.virtual_keycode {
                    println!("PRESSED ${:?}", input);
                    match key {
                        VirtualKeyCode::W => world.player = (world.player.0, world.player.1 - 0.01),
                        VirtualKeyCode::A => world.player = (world.player.0 - 0.01, world.player.1),
                        VirtualKeyCode::S => world.player = (world.player.0, world.player.1 + 0.01),
                        VirtualKeyCode::D => world.player = (world.player.0 + 0.01, world.player.1),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
