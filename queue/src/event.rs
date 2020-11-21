use common::Time;

#[derive(Debug, Clone)]
pub struct Event<T> {
    pub time: Time,
    pub payload: T,
}

impl<T> Event<T> {
    pub fn new(payload: T, time: Time) -> Self {
        Self { time, payload }
    }
}
impl<T> PartialEq for Event<T> {
    fn eq(&self, _other: &Event<T>) -> bool {
        true
    }
}
impl<T> Eq for Event<T> {}
