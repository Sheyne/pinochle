use futures::{
    sink::Sink,
    stream::{Stream, TryStreamExt},
};
use pinochle_lib::{shuffle, Action, Command, Game, Player, PlayerMap, TableCommand, TableState};
pub use room::*;
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
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
                            .or_insert_with(|| Arc::new(Table::new()))
                            .clone(),
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

#[derive(Debug)]
struct TableStateInternal {
    players: PlayerMap<SocketAddr>,
    ready: HashMap<SocketAddr, bool>,
}

enum TableStates {
    Lobby(Mutex<TableStateInternal>),
    Playing(RwLock<Option<Game>>),
}

use TableStates::*;

pub struct Table {
    state: RwLock<TableStates>,
    room: Room<SocketAddr, Message>,
}

impl Table {
    fn new() -> Table {
        Table {
            state: RwLock::new(Lobby(Mutex::new(TableStateInternal {
                ready: HashMap::new(),
                players: PlayerMap::new(),
            }))),
            room: Room::new(),
        }
    }

    fn error(&self, addr: &SocketAddr, error: String) {
        self.room.send_to(addr, Message::Text(error))
    }

    fn action(&self, action: Action) -> Completion {
        match action {
            Action::Resign => Finished,
        }
    }

    fn text(&self, addr: &SocketAddr, message: String) -> Completion {
        let (new_state, completion) = match &*self.state.read().unwrap() {
            Lobby(s) => {
                let mut s = s.lock().unwrap();

                match from_str(&message) {
                    Ok(TableCommand::SetReady(b)) => {
                        if s.players.get_player(addr).is_some() {
                            s.ready.insert(*addr, b);
                        }
                    }
                    Ok(TableCommand::SetPlayer(p)) => {
                        if let Some(player) = s.players.get_player(addr) {
                            *s.players.get_value_mut(player) = None;
                        }
                        let player_addr = s.players.get_value_mut(p);
                        let prev_val = *player_addr;
                        *player_addr = Some(*addr);
                        s.ready.insert(*addr, true);
                        if let Some(prev_val) = prev_val {
                            s.ready.insert(prev_val, false);
                        }
                    }
                    _ => (),
                }

                self.room.send(|addr| {
                    if let Some(player) = s.players.get_player(addr) {
                        let mut response = TableState::new(player);
                        for (player, ready) in s.ready.iter() {
                            if let Some(player) = s.players.get_player(&player) {
                                *response.ready.get_value_mut(player) = Some(*ready);
                            }
                        }
                        Some(Message::Text(to_string(&response).unwrap()))
                    } else {
                        None
                    }
                });

                if s.players
                    .iter_all()
                    .map(|(_, a)| *a.and_then(|a| s.ready.get(&a)).unwrap_or(&false))
                    .all(|b| b)
                {
                    println!("Starting playing");
                    let game = Game::new(Player::A, shuffle()).into();
                    self.room
                        .broadcast(Message::Text(to_string(&game).unwrap()));
                    let s = Playing(RwLock::new(Some(game)));

                    (Some(s), Continue)
                } else {
                    (None, Continue)
                }
            }
            Playing(game) => {
                let mut action: Option<Action> = None;
                let mut error: Option<String> = None;
                {
                    let mut game = game.write().unwrap();
                    *game = game.take().map(|game| {
                        let (game, a, e) = game.get_str(&message);

                        action = a;
                        error = e;

                        game
                    });
                }
                error.map(|e| self.error(addr, e));

                let completion = action.map(|a| self.action(a)).unwrap_or(Continue);
                (None, completion)
            }
        };
        if let Some(new_state) = new_state {
            *self.state.write().unwrap() = new_state;
        }
        completion
    }

    fn main_loop(&self, addr: &SocketAddr, message: Message) -> Completion {
        match message {
            Message::Text(message) => self.text(addr, message),
            _ => Continue,
        }
    }

    pub async fn play<S, E>(&self, a: SocketAddr, stream: S) -> S
    where
        S: Stream<Item = Result<Message, E>> + Sink<Message> + Unpin,
    {
        println!("Joining table {}", a);
        self.room.enter(a, stream, |m| self.main_loop(&a, m)).await
    }
}
