use slotmap::new_key_type;
use std::time::Duration;

new_key_type! {
    pub struct RawKey;
}

pub struct TaskKey<I, O> {
    pub raw: RawKey,
    pub _marker: std::marker::PhantomData<(I, O)>,
}

impl<I, O> Copy for TaskKey<I, O> {}
impl<I, O> Clone for TaskKey<I, O> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct TaskProfile {
    pub queue_time: Duration,
    pub exec_time: Duration,
    pub thread_id: std::thread::ThreadId,
}

pub struct TaskIO {
    pub inputs: Vec<String>,
    pub output: Option<String>,
}
