#[cfg(any(feature = "bus-queue", feature = "default"))]
use bus_queue::{channel::TryRecvError, flavors::arc_swap::Receiver as R};
#[cfg(feature = "crossbeam")]
use crossbeam_channel::{Receiver as R, TryIter, TryRecvError};

#[cfg(any(feature = "bus-queue", feature = "default"))]
use std::sync::Arc;

#[derive(Debug)]
pub struct Receiver<T> {
    pub receiver: R<T>,
}
impl<T> Receiver<T> {
    pub fn new(receiver: R<T>) -> Self {
        Self { receiver }
    }

    #[cfg(any(feature = "bus-queue", feature = "default"))]
    pub fn try_recv(&self) -> Result<Arc<T>, TryRecvError> {
        self.receiver.try_recv()
    }

    #[cfg(feature = "crossbeam")]
    pub fn try_recv(&self) -> Result<Arc<T>, TryRecvError> {
        self.receiver.try_recv()
    }

    #[cfg(feature = "crossbeam")]
    pub fn try_iter(&self) -> TryIter<'_, T> {
        self.receiver.try_iter()
    }
}

impl<T> Iterator for Receiver<T> {
    #[cfg(any(feature = "bus-queue", feature = "default"))]
    type Item = Arc<T>;
    #[cfg(feature = "crossbeam")]
    type Item = T;

    #[cfg(any(feature = "bus-queue", feature = "default"))]
    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.try_recv().ok()
        // self.receiver.try_iter().next()
    }

    #[cfg(feature = "crossbeam")]
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
