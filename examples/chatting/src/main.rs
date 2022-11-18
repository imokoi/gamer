use axum::{extract::WebSocketUpgrade, response::IntoResponse, routing::get, Extension, Router};
use gamer::{EventObserver, EventRunner, Gamer};
use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

#[derive(Debug)]
pub struct ChattingMessage {
    pub message: String,
}

impl gamer::MessageData for ChattingMessage {}

#[tokio::main]
async fn main() {
    let gamer = Arc::new(Mutex::new(Gamer::new()));
    gamer.lock().unwrap().on_event(
        1,
        Box::new(|message: Box<dyn gamer::MessageData>| {
            println!("Message: {:?}", message);
        }),
    );

    // gamer.run_event(
    //     1,
    //     Box::new(ChattingMessage {
    //         message: "Hello".to_string(),
    //     }),
    // );

    let app = Router::new()
        .route("/ws", get(handle_websocket))
        .layer(Extension(gamer));

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_websocket(
    ws: WebSocketUpgrade,
    Extension(gamer): Extension<Arc<Mutex<Gamer>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|stream| async move { gamer.lock().unwrap().handle_websocket_message(stream) })
}
