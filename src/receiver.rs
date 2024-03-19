use Ordering::Relaxed;
use crate::shared_state::SharedState;
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::atomic::Ordering::{AcqRel, Acquire, SeqCst};
use tokio::sync::Notify;

pub struct Receiver<T> {
    shared_state: Arc<SharedState<T>>,
    notify: Arc<Notify>,
    index: AtomicUsize,
}

impl<T: Clone + Send + Sync + 'static> Receiver<T> {
    pub async fn receive(&self) -> T {
        let message;
        loop {
            let index = self.index.load(Acquire);
            if index >= self.shared_state.messages.len() {
                self.notify.notified().await;
                continue;
            }
            let new_index = index + 1;
            match self.index.compare_exchange(index, new_index, AcqRel, Acquire) {
                Ok(_) => {
                    // The index was successfully incremented
                    message = self.shared_state.messages[index].clone();
                    break;
                },
                Err(_) => {
                    // The index was not incremented, retry the loop
                }
            }
        }
        message
    }

    // Adjust the constructor accordingly
    pub(crate) fn new(shared_state: Arc<SharedState<T>>) -> Self {
        let notify = {
            shared_state.add_receiver()
        };

        Receiver {
            shared_state,
            notify,
            index: AtomicUsize::new(0),
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
        let initial_count = 2 + channel.shared_state.notifiers.len();
        assert_eq!(initial_count, Arc::strong_count(&channel.shared_state));

        // Drop the receiver
        drop(receiver);

        // After dropping, the count should decrease by 1 (for the receiver)
        // and by the number of condition variables, since each receiver has one.
        let final_count = 1 + channel.shared_state.notifiers.len(); // Only `channel` and possibly senders hold a reference
        assert_eq!(final_count, Arc::strong_count(&channel.shared_state));
    }
}
