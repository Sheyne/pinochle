use futures::{
    sink::Sink,
    stream::{Stream, TryStreamExt},
};
use pinochle_lib::{
    command::{Command, PlayingInput, PlayingResponse, TableCommand, TableState},
    game::{states::Project, Game},
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
    E: std::fmt::Debug,
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

                    match table.join(addr, stream).await {
                        (result_stream, Ok(())) => {
                            stream = result_stream;
                        }
                        (_, Err(e)) => {
                            println!("Error: {:?}", e);
                            break;
                        }
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
}

#[derive(Debug)]
struct TableStateInternal {
    players: PlayerMap<Option<SocketAddr>>,
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
            players: PlayerMap::new(None, None, None, None),
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

    fn play(
        &self,
        addr: &SocketAddr,
        message: &str,
        player_map: &PlayerMap<SocketAddr>,
        game: &RwLock<Game>,
    ) -> Result<(Option<TableStates>, Completion), String> {
        let connected_player = player_map
            .get_player(addr)
            .ok_or("Not playing".to_owned())?;
        let input = from_str(message).map_err(|_| "Invalid message".to_owned())?;

        match input {
            PlayingInput::Resign => {
                let response = PlayingResponse::Resigned(connected_player);
                let message = to_string(&response).unwrap();
                let message = Message::Text(message);
                self.room.broadcast(message);

                Ok((Some(Lobby(Mutex::new(TableStateInternal::new()))), Finished))
            }
            PlayingInput::Play(game_input) => {
                {
                    let mut game = game.write().unwrap();
                    if game.turn().map_or(false, |p| p == connected_player) {
                        game.play(game_input.clone())?
                    } else {
                        Err("Not your turn".to_owned())?
                    }
                }

                let response = PlayingResponse::Played(connected_player, game_input);
                let message = to_string(&response).unwrap();
                let message = Message::Text(message);
                self.room.broadcast(message);

                Ok((None, Continue))
            }
        }
    }

    fn table_info(&self, player: Option<Player>, s: &TableStateInternal) -> Message {
        let mut response = TableState::new(player);
        for (player, ready) in s.ready.iter() {
            if let Some(player) = s.players.get_player(&Some(*player)) {
                *response.ready.get_value_mut(player) = *ready;
            }
        }
        Message::Text(to_string(&response).unwrap())
    }

    fn text(&self, addr: &SocketAddr, message: String) -> Completion {
        let (new_state, completion) = match &*self.state.read().unwrap() {
            Lobby(s) => {
                let mut s = s.lock().unwrap();

                match from_str(&message) {
                    Ok(TableCommand::SetReady(b)) => {
                        if s.players.get_player(&Some(*addr)).is_some() {
                            s.ready.insert(*addr, b);
                        }
                    }
                    Ok(TableCommand::SetPlayer(p)) => {
                        if let Some(player) = s.players.get_player(&Some(*addr)) {
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

                self.room
                    .send(|addr| Some(self.table_info(s.players.get_player(&Some(*addr)), &s)));

                if s.players
                    .iter_all()
                    .map(|(_, a)| *a.and_then(|a| s.ready.get(&a)).unwrap_or(&false))
                    .all(|b| b)
                {
                    println!("Starting playing");
                    let game: Game = Game::new(Player::A, shuffle()).into();
                    self.room.send(|dest| {
                        if let Some(player) = s.players.get_player(&Some(*dest)) {
                            let projected = game.project(player);
                            Some(Message::Text(to_string(&projected).unwrap()))
                        } else {
                            None
                        }
                    });
                    let s = Playing(s.players.clone().unwrap(), RwLock::new(game));

                    (Some(s), Continue)
                } else {
                    (None, Continue)
                }
            }
            Playing(player_map, game) => match self.play(addr, &message, player_map, game) {
                Ok(c) => c,
                Err(e) => {
                    let response = PlayingResponse::Error(e);
                    let message = to_string(&response).unwrap();
                    let message = Message::Text(message);
                    self.room.send_to(addr, message);

                    (None, Continue)
                }
            },
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

    pub async fn join<S, E>(&self, a: SocketAddr, stream: S) -> (S, Result<(), E>)
    where
        S: Stream<Item = Result<Message, E>> + Sink<Message> + Unpin,
    {
        println!("Joining table {}", a);

        let stream = self
            .room
            .enter(
                a,
                stream,
                || match &*self.state.read().unwrap() {
                    Lobby(table_state) => {
                        let mut table_state = table_state.lock().unwrap();

                        let player = table_state.players.get_player(&None);

                        if let Some(player) = player {
                            *table_state.players.get_value_mut(player) = Some(a);
                        }

                        self.room.send_to(&a, self.table_info(player, &table_state));
                    }
                    _ => {}
                },
                |m| self.main_loop(&a, m),
            )
            .await;

        match &*self.state.read().unwrap() {
            Lobby(table_state) => {
                let mut table_state = table_state.lock().unwrap();

                let player = table_state.players.get_player(&Some(a));
                table_state.ready.insert(a, false);

                if let Some(player) = player {
                    *table_state.players.get_value_mut(player) = None;
                }

                self.room.send(|addr| {
                    Some(
                        self.table_info(table_state.players.get_player(&Some(*addr)), &table_state),
                    )
                });
            }
            Playing(_player_map, _game) => {}
        }

        println!("Exiting {}", a);

        stream
    }
}
