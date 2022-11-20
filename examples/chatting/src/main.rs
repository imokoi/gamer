use axum::{extract::WebSocketUpgrade, response::IntoResponse, routing::get, Extension, Router};
use gamer::{EventObserver, Gamer};
use serde::{Deserialize, Serialize};
use std::{default, fmt::Debug, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChattingMessage {
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Chatting;

impl EventObserver for Chatting {}

impl Chatting {
    async fn on_events(&self, gamer: Arc<Mutex<Gamer>>) {
        self.on_event(
            gamer,
            100,
            Box::new(|data| {
                let message: ChattingMessage = serde_json::from_str(&data).unwrap();
                println!("message: {}", message.message);
            }),
        )
        .await;
    }
}

#[tokio::main]
async fn main() {
    let my_gamer = Arc::new(Mutex::new(Gamer::new()));
    let chatting = Chatting::default();
    chatting.on_events(my_gamer.clone()).await;

    let app = Router::new()
        .route("/ws", get(handle_websocket))
        .layer(Extension(my_gamer));

    println!("start http server on port: 8888");
    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .expect("failed to start http server");
}

async fn handle_websocket(
    ws: WebSocketUpgrade,
    Extension(my_gamer): Extension<Arc<Mutex<Gamer>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|stream| async move { Gamer::handle_websocket_message(my_gamer, stream).await })
}
