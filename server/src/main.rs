/*--------------------------------------------------------------------------------------------------------------
 * Copyright (c) Sheyne Anderson. All rights reserved.
 * Licensed under the MIT License.
 *-------------------------------------------------------------------------------------------------------------*/

use pinochle_lib::{Action, Card, Command, Rank, Response, Suit};
use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::spawn;
use tungstenite::{protocol::WebSocket, server::accept, Message};

struct Game {
    cards: Vec<Card>,
}

impl Game {
    fn run(&mut self, player: &str, action: Action) -> Response {
        println!("{} doing {:?}", player, action);
        Response::Update(self.cards.clone())
    }
}

struct GameServer {
    sockets: RwLock<HashMap<String, Arc<Mutex<WebSocket<TcpStream>>>>>,
    game: RwLock<Game>,
}

impl GameServer {
    fn send(&self, socket: Arc<Mutex<WebSocket<TcpStream>>>, command: Command) {
        match command {
            Command::Connect(id) => {
                self.sockets.write().unwrap().insert(id, socket);
            }
            Command::Action(a) => {
                let sockets = self.sockets.read().unwrap();
                let (id, _) = sockets
                    .iter()
                    .filter(|(_, s)| Arc::ptr_eq(s, &socket))
                    .next()
                    .unwrap();
                let response = self.game.write().unwrap().run(id, a);
                socket
                    .lock()
                    .unwrap()
                    .write_message(Message::Text(to_string(&response).unwrap()));
            }
            _ => {
                panic!("Boom");
            }
        }
    }

    fn new() -> GameServer {
        GameServer {
            sockets: RwLock::new(HashMap::new()),
            game: RwLock::new(Game { cards: vec![] }),
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
                let msg = websocket.lock().unwrap().read_message().unwrap();

                match msg {
                    Message::Text(s) => {
                        game_server.send(websocket.clone(), from_str(&s).unwrap());
                    }
                    _ => (),
                }
            }
        });
    }
}

// use pinochle_lib::{Card, Command, Rank, Response, Suit};
// use std::collections::HashMap;
// use std::rc::Rc;
// use strum::IntoEnumIterator;
// use ws::{listen, CloseCode, Handler, Message, Result, Sender};

// struct Connection {
//     out: Sender,
//     server: Rc<Server>,
// }

// impl<'a> Handler for Connection {
//     fn on_message(&mut self, msg: Message) -> Result<()> {
//         // Echo the message back
//         self.out.send(msg)
//     }

//     fn on_close(&mut self, code: CloseCode, reason: &str) {
//         // The WebSocket protocol allows for a utf8 reason for the closing state after the
//         // close code. WS-RS will attempt to interpret this data as a utf8 description of the
//         // reason for closing the connection. I many cases, `reason` will be an empty string.
//         // So, you may not normally want to display `reason` to the user,
//         // but let's assume that we know that `reason` is human-readable.
//         match code {
//             CloseCode::Normal => println!("The client is done with the connection."),
//             CloseCode::Away => println!("The client is leaving the site."),
//             _ => println!("The client encountered an error: {}", reason),
//         }
//     }
// }

// impl Connection {
//     fn new(server: Rc<Server>, out: Sender) -> Connection {
//         Connection {
//             out: out,
//             server: server.clone(),
//         }
//     }

//     fn add()
// }

// struct Server {
//     connections: HashMap<Sender, Connection>,
// }

// impl Server {
//     fn new() -> Server {
//         Server {
//             connections: HashMap::new(),
//         }
//     }
// }

// fn main() {
//     let server = Rc::new(Server::new());

//     listen("0.0.0.0:3012", move |out| {
//         let conn = Connection::new(server.clone(), out);
//         server.add(conn);
//         conn
//     })
//     .unwrap()
// }

//  fn main() {
//     let cards : Vec<Card> = iproduct!(Suit::iter(), Rank::iter()).map(|(s, r)| Card {suit: s, rank: r}).collect();

//     print!("Cards: ");
//     for card in cards {
//         print!("{:?}, ", card);
//     }
//     println!();

//     let mut board = Board {
//         hands: [vec!(Card{
//             suit: Suit::Heart,
//             rank: Rank::Ten}), vec!(Card{suit: Suit::Diamond, rank: Rank::Ace}, Card{suit: Suit::Spade, rank: Rank::Ace}), vec!(), vec!()],
//         play_area: vec!(),
//         taken: [vec!(), vec!()],
//         trump: Suit::Spade,
//         turn: 0
//     };

//     board.play(Card{suit: Suit::Heart, rank: Rank::Ten}).unwrap();

//     println!("Play area: {:?}", board.play_area);

//     board.play(Card{suit: Suit::Spade, rank: Rank::Ace}).unwrap();

//     println!("Play area: {:?}", board.play_area);
// }
