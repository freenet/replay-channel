use crate::shared_state::SharedState;
use parking_lot::RwLock;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct Receiver<T> {
    shared_state: Arc<RwLock<SharedState<T>>>,
    notify: Arc<Notify>,
    index: usize,
}

impl<T: Clone + Send + Sync + 'static> Receiver<T> {
    pub async fn receive(&mut self) -> T {
        let message;
        {
            while self.index >= self.shared_state.read().messages.len() {
                self.notify.notified().await;
            }
            message = self.shared_state.read().messages[self.index].clone();
        }
        self.index += 1;
        message
    }

    // Adjust the constructor accordingly
    pub(crate) fn new(shared_state: Arc<RwLock<SharedState<T>>>) -> Self {
        let notify = {
            let mut state = shared_state.write();
            state.add_receiver()
        };

        Receiver {
            shared_state,
            notify,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReplayChannel;

    #[test]
    fn receiver_resource_cleanup() {
        let channel: ReplayChannel<u8> = ReplayChannel::new();
        let _sender = channel.sender(); // Keep a sender to prevent premature cleanup
        let receiver = channel.receiver();

        // Initially, there should be an additional reference for the receiver
        // and one for each condvar inside the shared state.
        let initial_count = 2 + channel.shared_state.read().notifiers.len();
        assert_eq!(initial_count, Arc::strong_count(&channel.shared_state));

        // Drop the receiver
        drop(receiver);

        // After dropping, the count should decrease by 1 (for the receiver)
        // and by the number of condition variables, since each receiver has one.
        let final_count = 1 + channel.shared_state.read().notifiers.len(); // Only `channel` and possibly senders hold a reference
        assert_eq!(final_count, Arc::strong_count(&channel.shared_state));
    }
}
