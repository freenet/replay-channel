use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::{Arc, Condvar, Mutex};
use crate::receiver::Receiver;
use crate::sender::Sender;
use crate::shared_state::SharedState;

mod sender;
mod receiver;
mod shared_state;

/// A `ReplayChannel` provides a multi-receiver, message-passing communication channel
/// where receivers can "catch up" by receiving all previously sent messages before
/// continuing to receive new messages.
///
/// Each new `Receiver` created from a `ReplayChannel` will first receive all messages
/// that were sent prior to its creation, ensuring that it starts with the full context.
/// Once it has caught up, it will then receive messages as they are sent in real-time.
///
/// This is particularly useful in scenarios where the state history is important and
/// late-joining receivers need to process all past messages to be properly synchronized
/// with the current state.
///
/// # Examples
///
/// Creating a `ReplayChannel` and sending messages:
///
/// ```
/// use replay_channel::ReplayChannel;
/// let replay_channel = ReplayChannel::new();
/// let sender = replay_channel.sender();
/// sender.send("message 1");
/// sender.send("message 2");
///
/// let mut receiver = replay_channel.receiver();
/// assert_eq!(receiver.receive(), "message 1");
/// assert_eq!(receiver.receive(), "message 2");
///
/// let mut new_receiver = replay_channel.receiver();
/// assert_eq!(new_receiver.receive(), "message 1");
/// assert_eq!(new_receiver.receive(), "message 2");
///
/// sender.send("message 3");
/// assert_eq!(new_receiver.receive(), "message 3");
/// ```
pub struct ReplayChannel<T: Clone + Send + 'static> {
    shared_state: Arc<Mutex<SharedState<T>>>,
}

impl<T: Clone + Send + 'static> ReplayChannel<T> {
    pub fn new() -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            messages: VecDeque::new(),
            condvars: vec![],
        }));
        ReplayChannel { shared_state }
    }

    pub fn sender(&self) -> Sender<T> {
        Sender::new(Arc::clone(&self.shared_state))
    }

    pub fn receiver(&self) -> Receiver<T> {
        Receiver::new(Arc::clone(&self.shared_state))
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use super::*;

    #[test]
    fn message_sending_and_receiving() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver = channel.receiver();

        sender.send(1);
        sender.send(2);

        assert_eq!(receiver.receive(), 1);
        assert_eq!(receiver.receive(), 2);
    }

    #[test]
    fn receiver_replays_past_messages() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver1 = channel.receiver();

        // Send two messages
        sender.send(1);
        sender.send(2);

        // Receiver 1 receives the two messages
        assert_eq!(receiver1.receive(), 1);
        assert_eq!(receiver1.receive(), 2);

        // Receiver 2 is created and should receive the same two messages
        let mut receiver2 = channel.receiver();
        assert_eq!(receiver2.receive(), 1);
        assert_eq!(receiver2.receive(), 2);

        // Do not call receive() again to avoid blocking
    }

    #[test]
    fn multiple_receivers_real_time() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver1 = channel.receiver();
        let mut receiver2 = channel.receiver();

        sender.send(1);
        sender.send(2);

        assert_eq!(receiver1.receive(), 1);
        assert_eq!(receiver1.receive(), 2);
        assert_eq!(receiver2.receive(), 1);
        assert_eq!(receiver2.receive(), 2);

        sender.send(3);
        assert_eq!(receiver1.receive(), 3);
        assert_eq!(receiver2.receive(), 3);
    }

    #[test]
    fn no_lost_messages() {
        let channel = ReplayChannel::new();
        let sender1 = channel.sender();
        let sender2 = channel.sender();
        let mut receiver = channel.receiver();

        sender1.send(1);
        sender2.send(2);

        let received1 = receiver.receive();
        let received2 = receiver.receive();

        assert!(received1 == 1 && received2 == 2 || received1 == 2 && received2 == 1);
    }

    #[test]
    fn receiver_message_order() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver = channel.receiver();

        sender.send(1);
        sender.send(2);
        sender.send(3);

        assert_eq!(receiver.receive(), 1);
        assert_eq!(receiver.receive(), 2);
        assert_eq!(receiver.receive(), 3);
    }

    #[test]
    fn receiver_index_handling() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver = channel.receiver();

        sender.send(1);
        assert_eq!(receiver.receive(), 1);

        sender.send(2);
        sender.send(3);
        assert_eq!(receiver.receive(), 2);
        assert_eq!(receiver.receive(), 3);
    }

    #[test]
    fn error_handling_lock_poisoning() {
        use std::sync::Arc;
        use std::thread;

        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let shared_state = Arc::clone(&channel.shared_state);

        // Simulate a panic while holding the lock to poison it
        let _ = thread::spawn(move || {
            let _lock = shared_state.lock().unwrap();
            panic!("Simulating a panic while holding the lock");
        })
            .join();

        // Attempt to send a message after the lock has been poisoned
        let result = thread::spawn(move || {
            sender.send(42);  // This should handle the poisoned lock internally
        })
            .join();

        // Check if the send operation completed without panicking
        assert!(result.is_ok(), "Send operation should handle the poisoned lock and not panic");
    }

}
