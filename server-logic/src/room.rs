use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    future::Either,
    select,
    sink::Sink,
    stream::Stream,
    StreamExt,
};
use std::{collections::HashMap, sync::RwLock};

pub struct Room<Key, PeerMessage> {
    senders: RwLock<HashMap<Key, UnboundedSender<PeerMessage>>>,
}

pub enum Completion {
    Continue,
    Finished,
}

pub use Completion::*;

impl<Key, PeerMessage> Room<Key, PeerMessage>
where
    Key: Eq + std::hash::Hash + Clone,
    PeerMessage: Clone,
{
    pub fn new() -> Room<Key, PeerMessage> {
        Room {
            senders: RwLock::new(HashMap::new()),
        }
    }

    pub fn broadcast(&self, msg: PeerMessage) {
        self.broadcast_to(|_| true, msg);
    }

    pub fn broadcast_to<F>(&self, mut filter: F, msg: PeerMessage)
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

    pub fn send_to(&self, key: &Key, msg: PeerMessage) {
        if let Some(stream) = self.senders.read().unwrap().get(key) {
            stream.unbounded_send(msg.clone()).unwrap();
        }
    }

    pub fn send<F>(&self, mut message: F)
    where
        F: FnMut(&Key) -> Option<PeerMessage>,
    {
        let peers = self.senders.read().unwrap();

        for (key, recp) in peers.iter() {
            if let Some(msg) = message(key) {
                recp.unbounded_send(msg).unwrap();
            }
        }
    }

    pub async fn enter<S, I, C, SocketTx, SocketRx>(
        &self,
        key: Key,
        stream: S,
        initial: I,
        mut callback: C,
    ) -> S
    where
        S: Stream<Item = SocketRx> + Sink<SocketTx> + Unpin,
        I: FnOnce(),
        C: FnMut(&mut UnboundedSender<SocketTx>, Either<SocketRx, PeerMessage>) -> Completion,
    {
        let (mut outgoing, incoming) = stream.split();
        let (tx_for_others, mut rx_from_others) = unbounded();
        self.senders
            .write()
            .unwrap()
            .insert(key.clone(), tx_for_others);
        let (mut to_sink, rx_to_sink) = unbounded();
        let rx_to_sink = rx_to_sink.map(Ok);

        let mut sending_task = rx_to_sink.forward(&mut outgoing);
        let mut incoming = incoming.fuse();

        initial();

        loop {
            select! {
                _ = sending_task => (),
                x = rx_from_others.next() => {
                    if let Some(message) = x {
                        match callback(&mut to_sink, Either::Right(message)) {
                            Continue => (),
                            Finished => break,
                        }
                    }
                }
                x = incoming.next() => {
                    if let Some(message) = x {
                        match callback(&mut to_sink, Either::Left(message)) {
                            Continue => (),
                            Finished => break,
                        }
                    }
                }
            }
        }

        self.senders.write().unwrap().remove(&key);

        incoming.into_inner().reunite(outgoing).unwrap()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
