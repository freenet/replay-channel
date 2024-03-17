use orx_concurrent_vec::*;
use std::sync::Arc;
use tokio::sync::Notify;

pub(crate) struct SharedState<T> {
    pub(crate) messages: ConcurrentVec<T>,
    pub(crate) notifiers: ConcurrentVec<Arc<Notify>>, // Each receiver has an associated Condvar for notification
}

impl<T: Clone + Send + 'static> SharedState<T> {
    pub(crate) fn add_receiver(&self) -> Arc<Notify> {
        let condvar = Arc::new(Notify::new());
        self.notifiers.push(condvar.clone());
        condvar
    }
}
