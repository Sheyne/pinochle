use futures::channel::mpsc::{unbounded, TrySendError, UnboundedSender};
use futures_util::{future, pin_mut, sink::SinkExt, stream::TryStreamExt, StreamExt};
use pinochle_lib::{Action, Board, Response, ResponseError};
use serde_json::{from_str, to_string};
use std::{collections::HashMap, net::SocketAddr, sync::RwLock};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Message;

type Tx = UnboundedSender<Message>;
type PeerMap = RwLock<HashMap<SocketAddr, Tx>>;

#[derive(Debug)]
pub enum GameError {
    SocketError(tungstenite::error::Error),
    TrySendError(TrySendError<Message>),
}

impl From<tungstenite::error::Error> for GameError {
    fn from(item: tungstenite::error::Error) -> Self {
        GameError::SocketError(item)
    }
}

impl From<TrySendError<Message>> for GameError {
    fn from(item: TrySendError<Message>) -> Self {
        GameError::TrySendError(item)
    }
}

pub struct Game {
    peer_map: PeerMap,
    board: RwLock<Board>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: RwLock::new(Board::shuffle()),
            peer_map: RwLock::new(HashMap::new()),
        }
    }

    fn receive_action(&self, addr: SocketAddr, action: Action) -> Result<(), GameError> {
        println!("Msg: {:?}", action);

        let peers = self.peer_map.read().unwrap();

        // We want to broadcast the message to everyone except ourselves.
        let broadcast_recipients = peers
            .iter()
            // .filter(|(peer_addr, _)| peer_addr != &&addr)
            .map(|(_, ws_sink)| ws_sink);

        for recp in broadcast_recipients {
            let msg = "";
            recp.unbounded_send(Message::Text(msg.to_owned()))?;
        }

        Ok(())
    }

    fn message(response: &Response) -> Message {
        Message::text(to_string(response).unwrap())
    }

    pub async fn connect(
        &self,
        addr: SocketAddr,
        mut ws_stream: WebSocketStream<TcpStream>,
    ) -> Result<(), GameError> {
        ws_stream
            .send(Game::message(&Response::Error(ResponseError::NotConnected)))
            .await?;

        // Insert the write part of this peer to the peer map.
        let (tx, rx) = unbounded();
        let (outgoing, incoming) = ws_stream.split();

        println!("{} joined", addr);

        self.peer_map.write().unwrap().insert(addr, tx);

        let broadcast_incoming = incoming.try_for_each(|msg| {
            if let Message::Text(msg) = msg {
                if let Ok(action) = from_str(&msg) {
                    if let Err(e) = self.receive_action(addr, action) {
                        dbg!(e);
                    }
                }
            }

            future::ok(())
        });

        let receive_from_others = rx.map(Ok).forward(outgoing);

        pin_mut!(broadcast_incoming, receive_from_others);
        future::select(broadcast_incoming, receive_from_others).await;

        println!("{} disconnected", &addr);
        self.peer_map.write().unwrap().remove(&addr);

        Ok(())
    }
}
