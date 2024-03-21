use std::ops::Add;
use crate::shared_state::SharedState;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize};
use std::sync::atomic::Ordering::{AcqRel, Acquire};

pub struct Receiver<T> {
    shared_state: Arc<SharedState<T>>,
    broadcast_receiver: async_broadcast::Receiver<T>,
    index: AtomicUsize,
}

impl<T: Clone + Send + Sync + 'static> Receiver<T> {
    pub async fn receive(&mut self) -> T {
        let index = self.index.load(Acquire);
        if index < self.shared_state.messages.len() {
            // safely increment the index and return the message
            self.index.fetch_add(1, AcqRel);
            return self.shared_state.messages[index].clone()
        }
        self.broadcast_receiver.recv().await.expect("broadcast receiver should not be dropped")
    }

    pub(crate) fn new(shared_state: Arc<SharedState<T>>) -> Self {
        Receiver {
            shared_state: shared_state.clone(),
            broadcast_receiver : shared_state.sender.new_receiver(),
            index: AtomicUsize::new(0),
        }
    }
}

#[cfg(test)]
mod tests {

}
