use std::path::PathBuf;

use axum::{
    extract::ws::{WebSocketUpgrade, WebSocket},
    routing::get,
    response::Response,
    Router,
};
use std::net::SocketAddr;

mod log_parser;
mod watch;

#[tokio::main]
async fn main() -> () {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::task::spawn(async move { watch::watch_logs(&PathBuf::from("./logs"), tx).await });
    tokio::task::spawn(async move { watch::process_logs(rx).await });

    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/", get(handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    ()
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            return;
        };

        if socket.send(msg).await.is_err() {
            return;
        }
    }
}
