use async_trait::async_trait;
use axum::extract::ws::{Message, WebSocket};
use futures::stream::SplitSink;
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

#[async_trait]
pub trait EventObserver {
    async fn on_event(&self, gamer: Arc<Mutex<Gamer>>, code: MessageCode, handler: MessageHandler) {
        let mut my_gamer = gamer.lock().await;
        if let Some(event) = my_gamer.events.get(&code) {
            panic!("Event with code {} already exists", event.code);
        } else {
            my_gamer.events.insert(code, Event { code, handler });
        }
    }
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
        let arc_tx = Arc::new(Mutex::new(tx));

        // after establishing the connection, send ping messages to the client
        let ping_tx = arc_tx.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(10)).await;
                let message = Message::Ping("ping".into());
                let mut sender = ping_tx.lock().await;
                sender.send(message).await.ok();
            }
        });

        // handle the messages sent from clients
        let message_tx = arc_tx.clone();
        while let Some(Ok(message)) = rx.next().await {
            // handle messages from the client
            match message {
                Message::Text(text) => {
                    Gamer::handle_text_message(gamer.clone(), message_tx.clone(), text).await;
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

    async fn handle_text_message(
        gamer: Arc<Mutex<Gamer>>,
        session: Arc<Mutex<SplitSink<WebSocket, Message>>>,
        text: String,
    ) {
        let mut sender = session.lock().await;
        let websocket_message: serde_json::Value = match serde_json::from_str(&text) {
            Ok(message) => message,
            Err(e) => {
                let _ = sender
                    .send(Message::Text(format!("invalid message: {}", e)))
                    .await;
                return;
            }
        };

        match websocket_message.as_object() {
            Some(message) => {
                let code = match (*message)
                    .get("code")
                    .and_then(|v| v.as_str().and_then(|s| u64::from_str_radix(s, 10).ok()))
                    .map(|v| v as usize)
                {
                    Some(code) => code,
                    None => {
                        sender
                            .send(Message::Text(format!("invalid message code")))
                            .await
                            .ok();
                        return;
                    }
                };

                let data = match (*message).get("data").map(|v| v.to_string()) {
                    Some(data) => data,
                    None => {
                        sender
                            .send(Message::Text(format!("invalid message data")))
                            .await
                            .ok();
                        return;
                    }
                };

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
