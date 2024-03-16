use std::sync::{Arc, Condvar, Mutex};
use crate::shared_state::SharedState;

pub struct Receiver<T> {
    shared_state: Arc<Mutex<SharedState<T>>>,
    condvar: Arc<Condvar>,
    index: usize,  // Track the index of the next message to receive
}

impl<T: Clone + Send + 'static> Receiver<T> {
    pub fn receive(&mut self) -> T {
        let mut state = self.shared_state.lock().unwrap();

        // Loop until a message is available.
        while self.index >= state.messages.len() {
            // This call blocks until a new message is available.
            state = self.condvar.wait(state).unwrap();
        }

        // At this point, there is a guarantee that a message is available.
        let message = state.messages[self.index].clone();
        self.index += 1;
        message
    }
}

impl<T: Clone + Send + 'static> Receiver<T> {
    pub(crate) fn new(shared_state: Arc<Mutex<SharedState<T>>>) -> Self {
        let condvar = {
            let mut state = shared_state.lock().unwrap();
            state.add_receiver()
        };

        Receiver {
            shared_state,
            condvar,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ReplayChannel;
    use super::*;

    #[test]
    fn receiver_resource_cleanup() {
        let channel : ReplayChannel<u8> = ReplayChannel::new();
        let _sender = channel.sender(); // Keep a sender to prevent premature cleanup
        let receiver = channel.receiver();

        // Initially, there should be an additional reference for the receiver
        // and one for each condvar inside the shared state.
        let initial_count = 2 + channel.shared_state.lock().unwrap().condvars.len();
        assert_eq!(initial_count, Arc::strong_count(&channel.shared_state));

        // Drop the receiver
        drop(receiver);

        // After dropping, the count should decrease by 1 (for the receiver)
        // and by the number of condition variables, since each receiver has one.
        let final_count = 1 + channel.shared_state.lock().unwrap().condvars.len(); // Only `channel` and possibly senders hold a reference
        assert_eq!(final_count, Arc::strong_count(&channel.shared_state));
    }

}