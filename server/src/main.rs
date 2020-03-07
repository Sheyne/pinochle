use server_logic::{get_connection, State};
use std::{
    collections::HashMap,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(state: Arc<State>, raw_stream: TcpStream, addr: SocketAddr) {
    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    get_connection(&state, addr, ws_stream).await;
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = match std::env::var("PORT") {
        Ok(val) => format!("0.0.0.0:{}", val),
        Err(_) => format!("0.0.0.0:{}", 3011),
    };

    let state = Arc::new(RwLock::new(HashMap::new()));

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
