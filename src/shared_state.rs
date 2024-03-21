use append_only_vec::AppendOnlyVec;

pub(crate) struct SharedState<T> {
    pub(crate) messages: AppendOnlyVec<T>,
    pub(crate) sender: async_broadcast::Sender<T>,
}
