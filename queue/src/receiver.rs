use crossbeam_channel::{Receiver as R, TryRecvError};

pub struct Receiver<T> {
    pub receiver: R<T>,
}
impl<T> Receiver<T> {
    pub fn new(receiver: R<T>) -> Self {
        Self { receiver }
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }
}

impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.receiver.try_recv() {
            Ok(e) => Some(e),
            Err(_) => None,
        }
    }
}
