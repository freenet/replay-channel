use crate::shared_state::SharedState;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Sender<T> {
    shared_state: Arc<RwLock<SharedState<T>>>,
}

impl<T: Clone + Send + 'static> Sender<T> {
    pub fn send(&self, message: T) {
        {
            let mut state = self.shared_state.write();
            state.messages.push_back(message.clone());
        }
        for condvar in &self.shared_state.read().notifiers {
            condvar.notify_one();
        }
    }

    pub(crate) fn new(shared_state: Arc<RwLock<SharedState<T>>>) -> Self {
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
