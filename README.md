# ReplayChannel

A Rust library that provides a `ReplayChannel`, a multi-receiver communication channel where each 
new receiver can catch up by receiving all previously sent messages before continuing with real-time 
message reception.

This library was created by [Ian Clarke](https://twitter.com/sanity) for the 
[Freenet Project](https://freenet.org/).

## Features

- **Message Replay:** Allows new receivers to receive all previously sent messages, ensuring full context is maintained.
- **Multi-Receiver:** Supports multiple receivers, each with its own view of the message history and real-time stream.
- **Thread-Safe:** Designed to be safe for use across multiple threads, allowing concurrent sends and receives.

## Usage

Add `ReplayChannel` to your `Cargo.toml`:

```bash
$ cargo add replay-channel
```

### Basic Example

```rust
use replay_channel::ReplayChannel;

// Create a new ReplayChannel
let replay_channel = ReplayChannel::new();
let sender = replay_channel.sender();

// Send some messages
sender.send("message 1");
sender.send("message 2");

// Create a new receiver and catch up on past messages
let mut receiver = replay_channel.receiver();
assert_eq!(receiver.receive(), "message 1");
assert_eq!(receiver.receive(), "message 2");

// Continue to receive new messages in real-time
sender.send("message 3");
assert_eq!(receiver.receive(), "message 3");
```

## Testing

The crate comes with a suite of tests to ensure functionality. Run tests with:

```bash
cargo test
```

## Contributions

Contributions are welcome! Please feel free to submit a pull request or create an issue for bugs, feature requests, or documentation improvements.

## License

This crate is distributed under the [MIT license](LICENSE.md).
