/*--------------------------------------------------------------------------------------------------------------
 * Copyright (c) Sheyne Anderson. All rights reserved.
 * Licensed under the MIT License.
 *-------------------------------------------------------------------------------------------------------------*/

use pinochle_lib::{Action, Board, Command, Response, ResponseError};
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::spawn;
use tungstenite::{protocol::WebSocket, server::accept, Message};

struct Game {
    board: Board,
}

impl Game {
    fn new() -> Game {
        Game {
            board: Board::shuffle(),
        }
    }

    fn run(&mut self, player: &str, action: Action) -> Response {
        println!("{} doing {:?}", player, action);
        Response::Update(self.board.get(0))
    }
}

struct GameServer {
    sockets: RwLock<HashMap<String, Arc<Mutex<WebSocket<TcpStream>>>>>,
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
                let player = self.game.read().unwrap().board.get(0);

                let result = socket
                    .lock()
                    .unwrap()
                    .write_message(Message::Text(to_string(&Response::Update(player)).unwrap()));

                self.sockets.write().unwrap().insert(id, socket);

                result
            }
            Command::Action(a) => {
                let sockets = self.sockets.read().unwrap();
                let id = sockets
                    .iter()
                    .filter(|(_, s)| Arc::ptr_eq(s, &socket))
                    .map(|(id, _)| id)
                    .next();

                let response = match id {
                    Some(id) => self.game.write().unwrap().run(id, a),
                    None => Response::Error(ResponseError::NotConnected),
                };

                socket
                    .lock()
                    .unwrap()
                    .write_message(Message::Text(to_string(&response).unwrap()))
            }
        }
    }

    fn new() -> GameServer {
        GameServer {
            sockets: RwLock::new(HashMap::new()),
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
