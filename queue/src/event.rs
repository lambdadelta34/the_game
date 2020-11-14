use common::Time;

#[derive(Debug)]
pub struct Event<T> {
    time: Time,
    payload: T,
    name: String,
}

impl<T> Event<T> {
    pub fn new(payload: T, name: String, time: Time) -> Self {
        Self {
            time,
            payload,
            name,
        }
    }
}
impl<T> PartialEq for Event<T> {
    fn eq(&self, _other: &Event<T>) -> bool {
        true
    }
}
impl<T> Eq for Event<T> {}
