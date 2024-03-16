use std::sync::{Arc, Mutex};
use crate::shared_state::SharedState;

pub struct Sender<T> {
    shared_state: Arc<Mutex<SharedState<T>>>,
}


impl<T: Clone + Send + 'static> Sender<T> {
    pub fn send(&self, message: T) {
        // Attempt to acquire the lock, handling poison errors gracefully
        let mut state = self.shared_state.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

        state.messages.push_back(message.clone());
        for condvar in &state.condvars {
            condvar.notify_one();
        }
    }

    pub(crate) fn new(shared_state: Arc<Mutex<SharedState<T>>>) -> Self {
        Sender { shared_state }
    }
}

#[cfg(test)]
mod tests {
    use crate::ReplayChannel;
    use super::*;

    #[test]
    fn sender_resource_cleanup() {
        let channel : ReplayChannel<u8> = ReplayChannel::new();
        let sender = channel.sender();

        // Check the initial reference count (should be 2: one for `channel` and one for `sender`)
        assert_eq!(2, Arc::strong_count(&channel.shared_state));

        // Drop the sender
        drop(sender);

        // After dropping, the count should go back to 1 (only `channel` holds a reference)
        assert_eq!(1, Arc::strong_count(&channel.shared_state));
    }

}