// find a way to create mutable shared ownership
use std::sync::{ Arc, Mutex };

#[derive(Debug, Copy, Clone)]
pub struct Message {
}
impl PartialEq for Message {
    fn eq(&self, _other: &Message) -> bool {
        true
    }
}
impl Eq for Message {}

#[derive(Debug, Clone)]
pub struct Channel {
    pub messages: Vec<Message>
}
impl Channel {
    pub fn new() -> Self {
        Self {
          messages: vec!()
        }
    }
}

#[derive(Debug)]
pub struct Publisher {
    pub channel: Arc<Mutex<Channel>>,
}
impl Publisher {
    pub fn new(channel: Arc<Mutex<Channel>>) -> Self {
        Self {
            channel
        }
    }
    pub fn broadcast(&self, message: Message) {
        self.channel.lock().unwrap().messages.push(message);
    }
}

#[derive(Debug)]
pub struct Subscriber {
    pub channel: Arc<Mutex<Channel>>,
}
impl Subscriber {
    pub fn new(channel: Arc<Mutex<Channel>>) -> Self {
        Self {
            channel
        }
    }
    pub fn try_recv(&self) -> Option<Message> {
        self.channel.lock().unwrap().messages.pop()
    }
}

impl Iterator for Subscriber {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        self.try_recv()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let ch = Arc::new(Mutex::new(Channel::new()));
        let px = Publisher::new(ch.clone());
        let sx = Subscriber::new(ch);
        let msg = Message{};
        px.broadcast(msg);
        let res = sx.map(|x| x).collect::<Vec<Message>>();
        println!("{:?} {:?}", res, msg);
        assert_eq!(msg, res[0]);
    }
}
