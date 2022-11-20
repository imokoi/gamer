use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::{self};
use std::sync::Arc;
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

// Store is used to store the states of the game
// it is a singleton and can be accessed from anywhere
pub struct Store<S> {
    states: HashMap<String, S>,
}

// websocket messages
#[derive(Debug)]
pub struct WebsocketMessage {
    pub code: usize,
    pub data: String,
}

type MessageCode = usize;
type MessageHandler = Box<dyn Fn(String) + Send + Sync + 'static>;

pub struct Event {
    pub code: MessageCode,
    pub handler: MessageHandler,
}

pub trait EventObserver {
    fn on_event(&mut self, code: MessageCode, handler: MessageHandler);
}

pub trait EventRunner {
    fn run_event(&mut self, code: MessageCode, data: String);
}

// gamer manager
pub struct Gamer {
    pub events: HashMap<MessageCode, Event>,
}

impl Gamer {
    pub fn new() -> Self {
        Gamer {
            events: HashMap::new(),
        }
    }

    pub async fn handle_websocket_message(gamer: Arc<Mutex<Gamer>>, ws: WebSocket) {
        let (tx, mut rx) = ws.split();

        // after establishing the connection, send ping messages to the client
        let arc_tx = Arc::new(Mutex::new(tx));
        let ping_tx = arc_tx.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(10)).await;
                let message = Message::Ping("ping".into());
                let mut sender = ping_tx.lock().await;
                match sender.send(message).await {
                    Ok(_) => {
                        println!("sent ping");
                    }
                    Err(e) => {
                        println!("send error: {}", e);
                        break;
                    }
                };
            }
        });

        // handle the messages sent from clients
        let message_tx = arc_tx.clone();
        while let Some(Ok(message)) = rx.next().await {
            let mut sender = message_tx.lock().await;
            // handle messages from the client
            match message {
                Message::Text(text) => {
                    let maybe_websocket_message: Result<serde_json::Value, serde_json::Error> =
                        serde_json::from_str(&text);

                    let websocket_message = match maybe_websocket_message {
                        Ok(message) => message,
                        Err(e) => {
                            let _ = sender
                                .send(Message::Text(format!("invalid message: {}", e)))
                                .await;
                            return;
                        }
                    };

                    println!("websocket message: {:?}", websocket_message);

                    match websocket_message.as_object() {
                        Some(message) => {
                            let code = (*message)
                                .get("code")
                                .and_then(|v| v.as_u64())
                                .map(|v| v as usize)
                                .unwrap_or(0);

                            let data = (*message)
                                .get("data")
                                .and_then(|v| v.as_str())
                                .map(|v| v.to_string())
                                .unwrap_or("".to_string());

                            gamer.lock().await.run_event(code, data);
                            sender.send(Message::Text(text)).await.ok();
                        }
                        None => {
                            sender
                                .send(Message::Text("invalid message".into()))
                                .await
                                .ok();
                        }
                    }
                }
                Message::Pong(_) => {
                    println!("received pong");
                }
                Message::Close(_) => {
                    println!("Websocket connection closed");
                    break;
                }
                _ => {}
            }
        }
    }
}

impl EventObserver for Gamer {
    fn on_event(&mut self, code: MessageCode, handler: MessageHandler) {
        if let Some(event) = self.events.get(&code) {
            panic!("Event with code {} already exists", event.code);
        } else {
            self.events.insert(code, Event { code, handler });
        }
    }
}

impl EventRunner for Gamer {
    fn run_event(&mut self, code: MessageCode, data: String) {
        if let Some(event) = self.events.get(&code) {
            (event.handler)(data);
        } else {
            panic!("Event with code {} does not exist", code);
        }
    }
}
