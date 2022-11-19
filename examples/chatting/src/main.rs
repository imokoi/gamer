use axum::{extract::WebSocketUpgrade, response::IntoResponse, routing::get, Extension, Router};
use gamer::{EventObserver, EventRunner, Gamer};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChattingMessage {
    pub message: String,
}

// change to a macro #[derive(MessageData)]
impl gamer::MessageData for ChattingMessage {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[tokio::main]
async fn main() {
    let my_gamer = Arc::new(Mutex::new(Gamer::new()));
    my_gamer.lock().await.on_event(
        1,
        Box::new(|message: Box<dyn gamer::MessageData>| {
            let my_message = message.as_any().downcast_ref::<ChattingMessage>().unwrap();
            println!("Message: {:?}", my_message.message);
        }),
    );

    my_gamer.lock().await.run_event(
        1,
        Box::new(ChattingMessage {
            message: "Hello".to_string(),
        }),
    );

    let app = Router::new()
        .route("/ws", get(handle_websocket))
        .layer(Extension(my_gamer));

    axum::Server::bind(&"0.0.0.0:8888".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_websocket(
    ws: WebSocketUpgrade,
    Extension(my_gamer): Extension<Arc<Mutex<Gamer>>>,
) -> impl IntoResponse {
    ws.on_upgrade(|stream| async move { Gamer::handle_websocket_message(my_gamer, stream).await })
}
