use std::sync::{Arc};
use std::collections::VecDeque;
use tokio::sync::{Notify};

pub(crate) struct SharedState<T> {
    pub(crate) messages: VecDeque<T>,
    pub(crate) notifiers: Vec<Arc<Notify>>,  // Each receiver has an associated Condvar for notification
}

impl<T: Clone + Send + 'static> SharedState<T> {
    pub(crate) fn add_receiver(&mut self) -> Arc<Notify> {
        let condvar = Arc::new(Notify::new());
        self.notifiers.push(condvar.clone());
        condvar
    }
}