use std::sync::{Arc};
use std::collections::VecDeque;
use parking_lot::{Condvar, Mutex};

pub(crate) struct SharedState<T> {
    pub(crate) messages: VecDeque<T>,
    pub(crate) condvars: Vec<Arc<Condvar>>,  // Each receiver has an associated Condvar for notification
}

impl<T: Clone + Send + 'static> SharedState<T> {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(SharedState {
            messages: VecDeque::new(),
            condvars: vec![],
        }))
    }

    pub fn add_receiver(&mut self) -> Arc<Condvar> {
        let condvar = Arc::new(Condvar::new());
        self.condvars.push(condvar.clone());
        condvar
    }
}