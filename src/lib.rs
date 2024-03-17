use crate::receiver::Receiver;
use crate::sender::Sender;
use crate::shared_state::SharedState;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;

pub mod receiver;
pub mod sender;
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
/// ```rust
/// # use tokio::runtime::Runtime; // Hidden line
/// # use replay_channel::ReplayChannel; // This line is visible in the documentation
/// # let rt = Runtime::new().unwrap(); // Hidden line
/// # rt.block_on(async { // Hidden line
/// let replay_channel = ReplayChannel::new();
/// let sender = replay_channel.sender();
/// sender.send("message 1");
/// sender.send("message 2");
///
/// let mut receiver = replay_channel.receiver();
/// assert_eq!(receiver.receive().await, "message 1");
/// assert_eq!(receiver.receive().await, "message 2");
///
/// let mut new_receiver = replay_channel.receiver();
/// assert_eq!(new_receiver.receive().await, "message 1");
/// assert_eq!(new_receiver.receive().await, "message 2");
///
/// sender.send("message 3");
/// assert_eq!(new_receiver.receive().await, "message 3");
/// # }); // Hidden line
/// ```
pub struct ReplayChannel<T: Clone + Send + 'static> {
    shared_state: Arc<RwLock<SharedState<T>>>,
}

impl<T: Clone + Send + Sync + 'static> ReplayChannel<T> {
    pub fn new() -> Self {
        let shared_state = Arc::new(RwLock::new(SharedState {
            messages: VecDeque::new(),
            notifiers: vec![],
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
    use super::*;

    #[tokio::test]
    async fn message_sending_and_receiving() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver = channel.receiver();

        sender.send(1);
        sender.send(2);

        assert_eq!(receiver.receive().await, 1);
        assert_eq!(receiver.receive().await, 2);
    }

    #[tokio::test]
    async fn receiver_replays_past_messages() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver1 = channel.receiver();

        // Send two messages
        sender.send(1);
        sender.send(2);

        // Receiver 1 receives the two messages
        assert_eq!(receiver1.receive().await, 1);
        assert_eq!(receiver1.receive().await, 2);

        // Receiver 2 is created and should receive the same two messages
        let mut receiver2 = channel.receiver();
        assert_eq!(receiver2.receive().await, 1);
        assert_eq!(receiver2.receive().await, 2);

        // Do not call receive() again to avoid blocking
    }

    #[tokio::test]
    async fn multiple_receivers_real_time() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver1 = channel.receiver();
        let mut receiver2 = channel.receiver();

        sender.send(1);
        sender.send(2);

        assert_eq!(receiver1.receive().await, 1);
        assert_eq!(receiver1.receive().await, 2);
        assert_eq!(receiver2.receive().await, 1);
        assert_eq!(receiver2.receive().await, 2);

        sender.send(3);
        assert_eq!(receiver1.receive().await, 3);
        assert_eq!(receiver2.receive().await, 3);
    }

    #[tokio::test]
    async fn no_lost_messages() {
        let channel = ReplayChannel::new();
        let sender1 = channel.sender();
        let sender2 = channel.sender();
        let mut receiver = channel.receiver();

        sender1.send(1);
        sender2.send(2);

        let received1 = receiver.receive().await;
        let received2 = receiver.receive().await;

        assert!(received1 == 1 && received2 == 2 || received1 == 2 && received2 == 1);
    }

    #[tokio::test]
    async fn receiver_message_order() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver = channel.receiver();

        sender.send(1);
        sender.send(2);
        sender.send(3);

        assert_eq!(receiver.receive().await, 1);
        assert_eq!(receiver.receive().await, 2);
        assert_eq!(receiver.receive().await, 3);
    }

    #[tokio::test]
    async fn receiver_index_handling() {
        let channel = ReplayChannel::new();
        let sender = channel.sender();
        let mut receiver = channel.receiver();

        sender.send(1);
        assert_eq!(receiver.receive().await, 1);

        sender.send(2);
        sender.send(3);
        assert_eq!(receiver.receive().await, 2);
        assert_eq!(receiver.receive().await, 3);
    }
}
