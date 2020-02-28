/*--------------------------------------------------------------------------------------------------------------
 * Copyright (c) Sheyne Anderson. All rights reserved.
 * Licensed under the MIT License.
 *-------------------------------------------------------------------------------------------------------------*/

use pinochle_lib::{Action, Board, Command, Response, ResponseError};
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock, Weak};
use std::thread::spawn;
use tungstenite::{protocol::WebSocket, server::accept, Message};
use weak_table::PtrWeakKeyHashMap;

struct Game {
    board: Board,
    positions: HashMap<String, usize>,
}

impl Game {
    fn new() -> Game {
        Game {
            board: Board::shuffle(),
            positions: HashMap::new(),
        }
    }

    fn play(&mut self, player: usize, action: Action) -> Result<(), ResponseError> {
        if self.board.turn != player {
            return Result::Err(ResponseError::NotYourTurn);
        }

        match action {
            Action::PlayCard(card) => match self.board.play(card) {
                Ok(_) => (),
                Err(e) => return Result::Err(ResponseError::GameError(e.to_string())),
            },
        }

        Ok(())
    }

    fn run(&mut self, player: &str, action: Action) -> Result<(), ResponseError> {
        match self.positions.get(player) {
            Some(p) => {
                let p = *p;
                self.play(p, action)
            }
            None => Result::Err(ResponseError::NotPlaying),
        }
    }
}

struct GameServer {
    sockets: RwLock<PtrWeakKeyHashMap<Weak<Mutex<WebSocket<TcpStream>>>, String>>,
    game: RwLock<Game>,
}

impl GameServer {
    fn send(
        &self,
        socket: Arc<Mutex<WebSocket<TcpStream>>>,
        command: Command,
    ) -> Result<(), tungstenite::error::Error> {
        match command {
            Command::Connect(id) => {
                let game = &mut self.game.write().unwrap();
                let positions = &mut game.positions;

                dbg!(&positions);

                let position = match positions.get(&id) {
                    Some(p) => Some(*p),
                    None => {
                        let candidate = positions.len();
                        if candidate < 4 {
                            positions.insert(id.clone(), candidate);
                            Some(candidate)
                        } else {
                            None
                        }
                    }
                };

                dbg!(&position);

                if let Some(position) = position {
                    let player = game.board.get(position);

                    let result = socket.lock().unwrap().write_message(Message::Text(
                        to_string(&Response::Update(player)).unwrap(),
                    ));

                    self.sockets.write().unwrap().insert(socket, id);

                    result
                } else {
                    socket.lock().unwrap().write_message(Message::Text(
                        to_string(&Response::Error(ResponseError::NotPlaying)).unwrap(),
                    ))
                }
            }
            Command::Action(a) => {
                let sockets = self.sockets.read().unwrap();
                let id = sockets.get(&socket);

                let response = match id {
                    Some(id) => self.game.write().unwrap().run(id, a),
                    None => Result::Err(ResponseError::NotConnected),
                };

                match response {
                    Result::Err(err) => socket
                        .lock()
                        .unwrap()
                        .write_message(Message::Text(to_string(&Response::Error(err)).unwrap())),
                    Result::Ok(_) => {
                        let game = &self.game.read().unwrap();

                        for (socket, id) in self.sockets.read().unwrap().iter() {
                            if let Some(position) = game.positions.get(id) {
                                println!("Updating {} (position = {})", id, position);
                                let mut socket = socket.lock().unwrap();
                                socket.write_message(Message::Text(
                                    to_string(&Response::Update(game.board.get(*position)))
                                        .unwrap(),
                                ))?;
                            }
                        }

                        Ok(())
                    }
                }
            }
        }
    }

    fn new() -> GameServer {
        GameServer {
            sockets: RwLock::new(PtrWeakKeyHashMap::new()),
            game: RwLock::new(Game::new()),
        }
    }
}

fn main() {
    let game_server = Arc::new(GameServer::new());

    // A WebSocket echo server
    let server = TcpListener::bind("0.0.0.0:3012").unwrap();
    for stream in server.incoming() {
        let game_server = game_server.clone();
        spawn(move || {
            let websocket = Arc::new(Mutex::new(accept(stream.unwrap()).unwrap()));
            loop {
                let msg = websocket.lock().unwrap().read_message();

                match msg {
                    Ok(msg) => {
                        if let Message::Text(s) = msg {
                            if let Err(err) =
                                game_server.send(websocket.clone(), from_str(&s).unwrap())
                            {
                                println!("Error: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                        break;
                    }
                }
            }
        });
    }
}
