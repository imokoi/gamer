# gamer

Gamer is a Websocket messages handler and sessions manager.

you can use on_event style to handle messages like with SocketIO.

```rust
gamer.on_event(message_code, Box<dyn Fn(message)>)
```
