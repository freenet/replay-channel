use crate::shared_state::SharedState;
use std::sync::Arc;

pub struct Sender<T> {
    shared_state: Arc<SharedState<T>>,
}

impl<T: Clone + Send + 'static> Sender<T> {
    pub async fn send(&self, message: T) {
        {
            self.shared_state.messages.push(message.clone());
        }
        self.shared_state.sender.broadcast_direct(message).await.expect("broadcast should not fail");
    }

    pub(crate) fn new(shared_state: Arc<SharedState<T>>) -> Self {
        Sender { shared_state }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ReplayChannel;

    #[test]
    fn sender_resource_cleanup() {
        let channel: ReplayChannel<u8> = ReplayChannel::new();
        let sender = channel.sender();

        // Check the initial reference count (should be 2: one for `channel` and one for `sender`)
        assert_eq!(2, Arc::strong_count(&channel.shared_state));

        // Drop the sender
        drop(sender);

        // After dropping, the count should go back to 1 (only `channel` holds a reference)
        assert_eq!(1, Arc::strong_count(&channel.shared_state));
    }
}
