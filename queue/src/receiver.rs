use crossbeam_channel::{Receiver as R, TryIter, TryRecvError};

#[derive(Debug)]
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

    pub fn try_iter(&self) -> TryIter<'_, T> {
        self.receiver.try_iter()
    }
}
impl<T> Iterator for Receiver<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.try_iter().next()
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            receiver: self.receiver.clone(),
        }
    }
}

unsafe impl<T: Send> Send for Receiver<T> {}
unsafe impl<T: Send> Sync for Receiver<T> {}
