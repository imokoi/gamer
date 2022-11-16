use serde::Serialize;
use std::{collections::HashMap, fmt::Debug};

// Store is used to store the states of the game
// it is a singleton and can be accessed from anywhere
pub struct Store<S> {
    states: HashMap<String, S>,
}

pub trait MessageData: Debug {}

// websocket messages
#[derive(Debug)]
pub struct WebsocketMessage {
    pub code: usize,
    pub message: Box<dyn MessageData>,
}

type MessageCode = usize;

pub struct Event {
    pub code: MessageCode,
    pub handler: Box<dyn Fn(WebsocketMessage)>,
}

pub trait EventObserver {
    fn on_event(&mut self, code: MessageCode, handler: Box<dyn Fn(WebsocketMessage)>);
}

pub trait EventRunner {
    fn run(&mut self, code: MessageCode, message: WebsocketMessage);
}

pub struct Gamer {
    pub events: HashMap<MessageCode, Event>,
}

impl Gamer {
    pub fn new() -> Self {
        Gamer {
            events: HashMap::new(),
        }
    }
}

impl EventObserver for Gamer {
    fn on_event(&mut self, code: MessageCode, handler: Box<dyn Fn(WebsocketMessage)>) {
        if let Some(event) = self.events.get(&code) {
            panic!("Event with code {} already exists", event.code);
        } else {
            self.events.insert(code, Event { code, handler });
        }
    }
}

impl EventRunner for Gamer {
    fn run(&mut self, code: MessageCode, message: WebsocketMessage) {
        if let Some(event) = self.events.get(&code) {
            (event.handler)(message);
        }
    }
}
