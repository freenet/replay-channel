# ReplayChannel

`ReplayChannel` is a Rust library that lets you create a channel where messages are broadcast to all 
receivers. Importantly, if a new receiver is added later, they'll get all previously sent messages 
until they are caught up with the sender.

Developed by [Ian Clarke](https://twitter.com/sanity) for the [Freenet Project](https://freenet.org/).

## Features

- **Message Replay:** New receivers are sent all previously sent messages until they are caught up with the sender.
- **Multi-Receiver:** Supports multiple receivers, each with its own view of the message history and real-time stream.
- **Asynchronous:** Designed to be used with Tokio, async-std, or any other async runtime.
- **Efficient:** Uses an [AppendOnlyVec](https://crates.io/crates/append-only-vec) to store sent
                   messages, avoiding locks.

## Memory Usage

`ReplayChannel` stores all sent messages, so the memory usage is proportional to the number of 
messages sent. Because of this the number of messages sent should be bounded.

## Getting Started

To use `ReplayChannel`, add it to your project's `Cargo.toml`:

```bash
$ cargo add replay-channel
```

### Usage Example

```rust
let replay_channel = ReplayChannel::new();
let sender = replay_channel.sender();
sender.send("message 1");
sender.send("message 2");

let mut receiver = replay_channel.receiver();
assert_eq!(receiver.receive().await, "message 1");
assert_eq!(receiver.receive().await, "message 2");

let mut new_receiver = replay_channel.receiver();
assert_eq!(new_receiver.receive().await, "message 1");
assert_eq!(new_receiver.receive().await, "message 2");

sender.send("message 3");
assert_eq!(new_receiver.receive().await, "message 3");
```

## License

Available under the [MIT license](LICENSE.md).