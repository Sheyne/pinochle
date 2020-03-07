use futures::{
    sink::Sink,
    stream::{Stream, TryStreamExt},
};
use pinochle_lib::{shuffle, Action, Command, Game, Player};
pub use room::*;
use serde_json::from_str;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tungstenite::Message;

pub mod room;

pub type State = RwLock<HashMap<String, Arc<Table>>>;

pub async fn get_connection<S, E>(state: &State, addr: SocketAddr, mut stream: S)
where
    S: Stream<Item = Result<Message, E>> + Sink<Message> + Unpin,
{
    loop {
        let message = stream.try_next().await;
        match message {
            Ok(Some(Message::Text(message))) => match from_str(&message) {
                Ok(Command::JoinTable(name)) => {
                    let table = state.read().unwrap().get(&name).map(|t| t.clone());
                    let table = match table {
                        None => state
                            .write()
                            .unwrap()
                            .entry(name)
                            .or_insert_with(|| Arc::new(Table::new())).clone(),
                        Some(table) => table,
                    };

                    stream = table.play(addr, stream).await;
                }
                _ => (),
            },
            _ => (),
        }
    }
}

pub struct Table {
    room: Room<SocketAddr, Message>,
    game: RwLock<Option<Game>>,
}

impl Table {
    fn new() -> Table {
        Table {
            room: Room::new(),
            game: RwLock::new(Some(Game::new(Player::A, shuffle()).into())),
        }
    }

    fn error(&self, addr: &SocketAddr, error: String) {
        self.room.send_to::<SocketAddr>(addr, Message::Text(error))
    }

    fn action(&self, action: Action) -> Completion {
        match action {
            Action::Resign => {
                Finished
            }
        }
    }

    fn main_loop(&self, addr: &SocketAddr, message: Message) -> Completion {
        match message {
            Message::Text(message) => {
                let mut action: Option<Action> = None;
                let mut error: Option<String> = None;
                {
                    let mut game = self.game.write().unwrap();
                    *game = game.take().map(|game| {
                        let (game, a, e) = game.get_str(&message);

                        action = a;
                        error = e;

                        game
                    });
                }
                error.map(|e| self.error(addr, e));

                action.map(|a| self.action(a)).unwrap_or(Continue)
            }
            _ => Continue,
        }
    }

    pub async fn play<S, E>(&self, a: SocketAddr, stream: S) -> S
    where
        S: Stream<Item = Result<Message, E>> + Sink<Message> + Unpin,
    {
        self.room.enter(a, stream, |m| self.main_loop(&a, m)).await
    }
}
