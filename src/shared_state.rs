use std::sync::Arc;
use append_only_vec::AppendOnlyVec;
use tokio::sync::broadcast;

pub(crate) struct SharedState<T> {
    pub(crate) messages: AppendOnlyVec<T>,
    pub(crate) notifier: broadcast::Sender<T>,
}

impl<T: Clone + Send + 'static> SharedState<T> {
    pub(crate) fn add_receiver(&self) -> broadcast::Receiver<T> {
        self.notifier.subscribe()
    }
}
