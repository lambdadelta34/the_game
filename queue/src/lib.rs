#[cfg(any(feature = "bus-queue", feature = "default"))]
use bus_queue::raw_bounded as queue;
#[cfg(feature = "crossbeam")]
use crossbeam_channel::bounded as queue;

pub mod event;
pub mod receiver;
pub mod sender;

use receiver::Receiver;
use sender::Sender;

pub fn create_queue<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let (s, r) = queue(size);
    (Sender::new(s), Receiver::new(r))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::*;
    #[test]
    fn send_simple_message() {
        let (px, sx) = create_queue(2);
        let msg = Event::new(2, 1);
        px.push(msg).unwrap();
        let res = sx.map(|x| x).collect::<Vec<_>>();
        assert_eq!(Event::new(2, 1).payload, res[0].payload);
    }
    // #[test]
    // fn send_simple_message1() {
    //     let (s, r) = bounded(3);
    //     let r1 = r.clone();
    //     let msg = Event::new(2, 1);
    //     s.send(msg).unwrap();
    //     let res = r.try_recv().unwrap();
    //     let res1 = r1.try_recv().unwrap();
    //     assert_eq!(Event::new(2, 1), res);
    //     assert_eq!(Event::new(2, 1), res1);
    // }
    #[test]
    fn receive_double_message() {
        let (px, sx) = create_queue(2);
        let sx1 = sx.clone();
        let msg = Event::new(2, 1);
        px.push(msg).unwrap();
        let res = sx.try_recv().unwrap();
        let res1 = sx1.try_recv().unwrap();
        assert_eq!(Event::new(2, 1).payload, res.payload);
        assert_eq!(Event::new(2, 1).payload, res1.payload);
    }

    // #[test]
    // fn send_message() {
    //     let (px, sx) = bounded(1);
    //     let msg = Event::new(2, 1);
    //     px.send(msg).unwrap();
    //     let res = sx.try_recv().unwrap();
    //     assert_eq!(Event::new(2, 1), res);
    // }
}
