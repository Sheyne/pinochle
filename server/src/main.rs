use server_logic::{get_connection, State};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};
use warp::ws::WebSocket;
use warp::Filter;

static NEXT_USER_ID: AtomicUsize = AtomicUsize::new(1);

async fn handle_connection(state: Arc<State>, socket: WebSocket) {
    let addr = NEXT_USER_ID.fetch_add(1, Ordering::Relaxed);
    println!("WebSocket connection established: {}", addr);
    get_connection(&state, addr, socket).await;
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3011);

    let state: Arc<State> = Arc::new(RwLock::new(HashMap::new()));
    let state = warp::any().map(move || state.clone());

    let socket = warp::path("socket")
        .and(warp::ws())
        .and(state)
        .map(|ws: warp::ws::Ws, state| {
            ws.on_upgrade(move |socket| handle_connection(state, socket))
        });

    let style = warp::get()
        .and(warp::path::path("style.css"))
        .and(warp::fs::file("./client/style.css"));

    let pkg = warp::get()
        .and(warp::path::path("pkg"))
        .and(warp::fs::dir("./client/pkg"));

    let index = warp::path::end().and(warp::fs::file("./client/index.html"));

    let routes = index.or(style).or(pkg).or(socket);

    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
