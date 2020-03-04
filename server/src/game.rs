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
    players: RwLock<HashMap<SocketAddr, usize>>,
    board: RwLock<Board>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            board: RwLock::new(Board::shuffle()),
            players: RwLock::new(HashMap::new()),
            peer_map: RwLock::new(HashMap::new()),
        }
    }

    fn do_action(&self, index: usize, action: &Action) -> Result<(), ResponseError> {
        if let Err(e) = match action {
            Action::PlayCard(c) => {
                if self.board.read().unwrap().turn != index {
                    return Err(ResponseError::NotYourTurn);
                }
                self.board.write().unwrap().play(*c)
            }
            Action::Reset => {
                *self.board.write().unwrap() = Board::shuffle();
                Ok(())
            }
        } {
            return Err(ResponseError::GameError(e.to_string()));
        }

        Ok(())
    }

    fn receive_action(&self, addr: SocketAddr, action: &Action) -> Result<(), GameError> {
        println!("Msg: {:?}", action);

        let index = self.players.read().unwrap().get(&addr).map(|x| *x);

        if let Some(index) = index {
            match self.do_action(index, action) {
                Ok(_) => {
                    let peers = self.peer_map.read().unwrap();

                    for (key, recp) in peers.iter() {
                        let index = self.players.read().unwrap().get(key).map(|x| *x);
                        if let Some(index) = index {
                            let state = self.board.read().unwrap().get(index);
                            dbg!(&state);
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
    ) -> Result<(), GameError> {
        let player_index = {
            let mut players = self.players.write().unwrap();
            let player_index = players.len();
            players.insert(addr, player_index);
            player_index
        };

        let initial_message = Response::Update(self.board.read().unwrap().get(player_index));
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
