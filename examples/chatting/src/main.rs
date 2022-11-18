use gamer::{EventObserver, EventRunner, Gamer};
use std::fmt::Debug;

#[derive(Debug)]
pub struct ChattingMessage {
    pub message: String,
}

impl gamer::MessageData for ChattingMessage {}

fn main() {
    let mut gamer = Gamer::new();
    gamer.on_event(
        1,
        Box::new(|message: Box<dyn gamer::MessageData>| {
            println!("Message: {:?}", message);
        }),
    );

    gamer.run_event(
        1,
        Box::new(ChattingMessage {
            message: "Hello".to_string(),
        }),
    );
}
