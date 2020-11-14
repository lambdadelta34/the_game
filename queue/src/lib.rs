use crossbeam_channel::bounded;
pub mod event;
pub mod receiver;
pub mod sender;

use receiver::Receiver;
use sender::Sender;

pub fn create_queue<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let (s, r) = bounded(size);
    (Sender::new(s), Receiver::new(r))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::*;
    #[test]
    fn send_simple_message() {
        println!("0");
        let (px, sx) = create_queue(2);
        let msg = Event::new(2, String::from("sad"), 1);
        px.push(msg).unwrap();
        let res = sx.map(|x| x).collect::<Vec<Event<_>>>();
        assert_eq!(Event::new(2, String::from("sad"), 1), res[0]);
    }
}
