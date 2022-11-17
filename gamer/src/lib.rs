use std::{collections::HashMap, fmt::Debug};

// Store is used to store the states of the game
// it is a singleton and can be accessed from anywhere
pub struct Store<S> {
    states: HashMap<String, S>,
}

pub trait MessageData: Debug {}

// websocket messages
#[derive(Debug)]
pub struct WebsocketMessage<T: MessageData> {
    pub code: usize,
    pub data: T,
}

type MessageCode = usize;

pub struct Event {
    pub code: MessageCode,
    pub handler: Box<dyn Fn(Box<dyn MessageData>)>,
}

pub trait EventObserver {
    fn on_event(&mut self, code: MessageCode, handler: Box<dyn Fn(Box<dyn MessageData>)>);
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
}

impl EventObserver for Gamer {
    fn on_event(&mut self, code: MessageCode, handler: Box<dyn Fn(Box<dyn MessageData>)>) {
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
