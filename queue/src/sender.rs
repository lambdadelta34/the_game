#[cfg(any(feature = "bus-queue", feature = "default"))]
use bus_queue::channel::SendError;
#[cfg(any(feature = "bus-queue", feature = "default"))]
use bus_queue::flavors::arc_swap::Sender as S;
#[cfg(feature = "crossbeam")]
use crossbeam_channel::{SendError, Sender as S};

#[derive(Debug)]
pub struct Sender<T> {
    sender: S<T>,
}

impl<T> Sender<T> {
    pub fn new(sender: S<T>) -> Self {
        Self { sender }
    }

    #[cfg(any(feature = "bus-queue", feature = "default"))]
    pub fn push(&self, event: T) -> Result<(), SendError<T>> {
        self.sender.broadcast(event)
    }
    #[cfg(feature = "crossbeam")]
    pub fn push(&self, event: T) -> Result<(), SendError<T>> {
        self.sender.send(event)
    }
}

#[cfg(feature = "crossbeam")]
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Sender {
            sender: self.sender.clone(),
        }
    }
}
