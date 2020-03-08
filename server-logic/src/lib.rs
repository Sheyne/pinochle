use futures::{
    sink::Sink,
    stream::{Stream, TryStreamExt},
};
use pinochle_lib::{
    command::{Command, PlayingInput, PlayingResponse, TableCommand, TableState},
    game::Game,
    shuffle, Player, PlayerMap,
};
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
    Playing(PlayerMap<SocketAddr>, RwLock<Game>),
}

use TableStates::*;

pub struct Table {
    state: RwLock<TableStates>,
    room: Room<SocketAddr, Message>,
}

impl TableStateInternal {
    fn new() -> TableStateInternal {
        TableStateInternal {
            ready: HashMap::new(),
            players: PlayerMap::new(),
        }
    }
}

impl Table {
    fn new() -> Table {
        Table {
            state: RwLock::new(Lobby(Mutex::new(TableStateInternal::new()))),
            room: Room::new(),
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
                    let s = Playing(s.players.clone(), RwLock::new(game));

                    (Some(s), Continue)
                } else {
                    (None, Continue)
                }
            }
            Playing(player_map, game) => {
                if let Ok(input) = from_str(&message) {
                    match input {
                        PlayingInput::Resign => {
                            (Some(Lobby(Mutex::new(TableStateInternal::new()))), Finished)
                        }
                        PlayingInput::Play(game_input) => {
                            let (player, result) = {
                                let mut game = game.write().unwrap();
                                let player = game.turn();

                                if let Some(player) = player {
                                    if let Some(connected_player) = player_map.get_player(addr) {
                                        if player == connected_player {
                                            let result = game.play(game_input.clone());
                                            (
                                                Some(player),
                                                result.clone().map_err(|e| e.to_string()),
                                            )
                                        } else {
                                            (None, Err("Not active player".to_owned()))
                                        }
                                    } else {
                                        (None, Err("Not playing".to_owned()))
                                    }
                                } else {
                                    (None, Err("Game is over".to_owned()))
                                }
                            };

                            match result {
                                Ok(_) => {
                                    let response = PlayingResponse::Played(
                                        player.expect("Player must be non none or we'd be in Err"),
                                        game_input,
                                    );
                                    let message = to_string(&response).unwrap();
                                    let message = Message::Text(message);
                                    self.room.broadcast(message)
                                }
                                Err(e) => {
                                    let response = PlayingResponse::Error(e);
                                    let message = to_string(&response).unwrap();
                                    let message = Message::Text(message);
                                    self.room.send_to(addr, message)
                                }
                            }

                            (None, Continue)
                        }
                    }
                } else {
                    (None, Continue)
                }
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
