use futures::channel::mpsc::{unbounded, TrySendError, UnboundedSender};
use futures_util::{future, pin_mut, sink::SinkExt, stream::TryStreamExt, StreamExt};
use pinochle_lib::{Action, Response, ResponseError};
use serde_json::{from_str, to_string};
use std::{collections::HashMap, net::SocketAddr, sync::RwLock};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::protocol::Message;

mod game;
use game::Game;

type Tx = UnboundedSender<Message>;
type PeerMap = RwLock<HashMap<SocketAddr, Tx>>;

#[derive(Debug)]
pub enum TableError {
    SocketError(tungstenite::error::Error),
    TrySendError(TrySendError<Message>),
}

impl From<tungstenite::error::Error> for TableError {
    fn from(item: tungstenite::error::Error) -> Self {
        TableError::SocketError(item)
    }
}

impl From<TrySendError<Message>> for TableError {
    fn from(item: TrySendError<Message>) -> Self {
        TableError::TrySendError(item)
    }
}

pub struct Table {
    peer_map: PeerMap,
    players: RwLock<HashMap<SocketAddr, usize>>,
    game: RwLock<Game>,
}

impl Table {
    pub fn new() -> Self {
        Table {
            game: RwLock::new(Game::new()),
            players: RwLock::new(HashMap::new()),
            peer_map: RwLock::new(HashMap::new()),
        }
    }

    fn do_action(&self, index: usize, action: &Action) -> Result<(), ResponseError> {
        match action {
            Action::PlayCard(c) => self.game.write().unwrap().play_card(index, c),
            Action::Reset => {
                *self.game.write().unwrap() = Game::new();
                Ok(())
            }
        }
    }

    fn receive_action(&self, addr: SocketAddr, action: &Action) -> Result<(), TableError> {
        println!("Msg: {:?}", action);

        let index = self.players.read().unwrap().get(&addr).map(|x| *x);

        if let Some(index) = index {
            match self.do_action(index, action) {
                Ok(_) => {
                    let peers = self.peer_map.read().unwrap();

                    for (key, recp) in peers.iter() {
                        let index = self.players.read().unwrap().get(key).map(|x| *x);
                        if let Some(index) = index {
                            let state = self.game.read().unwrap().get(index);
                            recp.unbounded_send(Self::message(&Response::Update(state)))?;
                        }
                    }
                }
                Err(e) => {
                    if let Some(recp) = self.peer_map.read().unwrap().get(&addr) {
                        recp.unbounded_send(Self::message(&Response::Error(e)))?;
                    }
                }
            }
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
    ) -> Result<(), TableError> {
        let player_index = {
            let mut players = self.players.write().unwrap();
            let player_index = players.len();
            players.insert(addr, player_index);
            player_index
        };

        let initial_message = Response::Update(self.game.read().unwrap().get(player_index));
        ws_stream.send(Self::message(&initial_message)).await?;

        // Insert the write part of this peer to the peer map.
        let (tx, rx) = unbounded();
        let (outgoing, incoming) = ws_stream.split();

        println!("{} joined", addr);

        self.peer_map.write().unwrap().insert(addr, tx);

        let broadcast_incoming = incoming.try_for_each(|msg| {
            if let Message::Text(msg) = msg {
                if let Ok(action) = from_str(&msg) {
                    if let Err(e) = self.receive_action(addr, &action) {
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
        self.players.write().unwrap().remove(&addr);

        Ok(())
    }
}
