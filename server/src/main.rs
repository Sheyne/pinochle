use futures_util::StreamExt;
use pinochle_lib::Command;
use serde_json::from_str;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tungstenite::protocol::Message;

mod game;
use game::Game;

struct State {
    games: HashMap<String, Arc<Game>>,
}

async fn handle_connection(state: Arc<Mutex<State>>, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let mut ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let message = ws_stream.next().await;
    if let Some(Ok(Message::Text(text))) = message {
        match from_str(&text) {
            Ok(Command::Connect(room)) => {
                let game = state
                    .lock()
                    .unwrap()
                    .games
                    .entry(room)
                    .or_insert(Arc::new(Game::new()))
                    .clone();

                if let Err(e) = game.connect(addr, ws_stream).await {
                    dbg!(e);
                }
            }
            _ => (),
        }
    }

    println!("Handle connection closing {}", addr);
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "0.0.0.0:3012".to_string());

    let state = Arc::new(Mutex::new(State {
        games: HashMap::new(),
    }));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let mut listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }

    Ok(())
}
