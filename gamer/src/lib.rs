use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{self, json};
use std::any::Any;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

// Store is used to store the states of the game
// it is a singleton and can be accessed from anywhere
pub struct Store<S> {
    states: HashMap<String, S>,
}

pub trait MessageData: Debug {
    fn as_any(&self) -> &dyn std::any::Any;
}

// websocket messages
#[derive(Debug)]
pub struct WebsocketMessage {
    pub code: usize,
    pub data: String,
}

type MessageCode = usize;
type MessageHandler = Box<dyn Fn(Box<dyn MessageData>) + Send + Sync + 'static>;
pub struct Event {
    pub code: MessageCode,
    pub handler: MessageHandler,
}

pub trait EventObserver {
    fn on_event(&mut self, code: MessageCode, handler: MessageHandler);
}

pub trait EventRunner {
    fn run_event(&mut self, code: MessageCode, message: Box<dyn MessageData>);
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

        // send pint pong message
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

        let message_tx = arc_tx.clone();
        while let Some(Ok(message)) = rx.next().await {
            let mut sender = message_tx.lock().await;
            // handle messages from the client
            match message {
                Message::Text(text) => {
                    let websocket_message: serde_json::Value = serde_json::from_str(&text).unwrap();
                    println!("websocket message: {:?}", websocket_message);
                    let code_map = websocket_message.as_object().unwrap();
                    println!("code map: {:?}", code_map);
                    let code = (*code_map).get("code").unwrap();
                    println!("code: {:?}", code);
                    let data = websocket_message
                        .get("data")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string();
                    // let message = WebsocketMessage { code, data };
                    // println!("received message: {:?}", message);
                    // match websocketMessage {
                    //     Ok(message) => {
                    //         let code = message.code;
                    //         let data = message.data;
                    //         gamer.lock().await.run_event(code, data);
                    //     }
                    //     Err(e) => {
                    //         println!("error: {}", e);
                    //     }
                    // }
                    sender.send(Message::Text(text)).await.unwrap();
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
    fn run_event(&mut self, code: MessageCode, message: Box<dyn MessageData>) {
        if let Some(event) = self.events.get(&code) {
            (event.handler)(message);
        }
    }
}
