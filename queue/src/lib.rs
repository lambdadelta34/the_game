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
        let (px, sx) = create_queue(2);
        let msg = Event::new(2, 1.);
        let sx1 = sx.clone();
        px.push(msg).unwrap();
        let res = sx.map(|x| x).collect::<Vec<_>>();
        let res1 = sx1.map(|x| x).collect::<Vec<_>>();
        assert_eq!(Event::new(2, 1.), res[0]);
        assert_eq!(Event::new(2, 1.), res1[0]);
    }

    #[test]
    fn receive_double_message() {
        let (px, sx) = create_queue(2);
        let msg = Event::new(2, 1.);
        px.push(msg).unwrap();
        let msg = Event::new(2, 1.);
        px.push(msg).unwrap();
        let res = sx.try_iter().map(|x| x).collect::<Vec<_>>();
        let res1 = sx.try_iter().map(|x| x).collect::<Vec<_>>();
        assert_eq!(Event::new(2, 1.), res[0]);
        assert_eq!(Event::new(2, 1.), res1[0]);
    }

    #[test]
    fn send_message() {
        let (px, sx) = bounded(1);
        let msg = Event::new(2, 1.);
        px.send(msg).unwrap();
        let res = sx.try_recv().unwrap();
        let _res2 = sx.try_recv().unwrap();
        assert_eq!(Event::new(2, 1.), res);
    }
}
