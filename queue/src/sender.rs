use crossbeam_channel::{SendError, Sender as S};

pub struct Sender<T> {
    sender: S<T>,
}

impl<T> Sender<T> {
    pub fn new(sender: S<T>) -> Self {
        Self { sender }
    }

    pub fn push(&self, event: T) -> Result<(), SendError<T>> {
        self.sender.send(event)
    }
}
