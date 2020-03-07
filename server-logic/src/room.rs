use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    future::{self, Either},
    sink::{Sink, SinkExt},
    stream::{Stream, TryStreamExt},
    StreamExt,
};
use std::{collections::HashMap, sync::RwLock};

pub struct Room<Key, Message> {
    senders: RwLock<HashMap<Key, UnboundedSender<Message>>>,
}

fn left<L, R>(e: Either<L, R>) -> Option<L> {
    match e {
        Either::Left(l) => Some(l),
        _ => None,
    }
}

pub enum Completion {
    Continue,
    Finished,
}

pub use Completion::*;

impl<Key, Message> Room<Key, Message>
where
    Key: Eq + std::hash::Hash + Clone,
    Message: Clone,
{
    pub fn new() -> Room<Key, Message> {
        Room {
            senders: RwLock::new(HashMap::new()),
        }
    }

    pub fn broadcast(&self, msg: Message) {
        self.broadcast_to(|_| true, msg);
    }

    pub fn broadcast_to<F>(&self, mut filter: F, msg: Message)
    where
        F: FnMut(&Key) -> bool,
    {
        let peers = self.senders.read().unwrap();

        let broadcast_recipients = peers
            .iter()
            .filter(|(peer_key, _)| filter(*peer_key))
            .map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            recp.unbounded_send(msg.clone()).unwrap();
        }
    }

    pub fn send_to(&self, key: &Key, msg: Message) {
        if let Some(stream) = self.senders.read().unwrap().get(key) {
            stream.unbounded_send(msg.clone()).unwrap();
        }
    }

    pub fn send<F>(&self, mut message: F)
    where
        F: FnMut(&Key) -> Option<Message>,
    {
        let peers = self.senders.read().unwrap();

        for (key, recp) in peers.iter() {
            if let Some(msg) = message(key) {
                recp.unbounded_send(msg).unwrap();
            }
        }
    }

    pub async fn enter<E, S, C>(&self, key: Key, stream: S, mut callback: C) -> S
    where
        S: Stream<Item = Result<Message, E>> + Sink<Message> + Unpin,
        C: FnMut(Message) -> Completion,
    {
        let (mut outgoing, mut incoming) = stream.split();
        let (tx, rx) = unbounded();
        self.senders.write().unwrap().insert(key.clone(), tx);

        let mut forward = rx.map(Ok);

        let mut send_all = outgoing.send_all(&mut forward);
        loop {
            let selected = future::select(incoming.try_next(), send_all);
            let (message, send_all_) = left(selected.await).unwrap();
            send_all = send_all_;

            match message {
                Ok(Some(message)) => match callback(message) {
                    Finished => break,
                    _ => (),
                },
                _ => (),
            }
        }

        self.senders.write().unwrap().remove(&key);

        incoming.reunite(outgoing).unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
