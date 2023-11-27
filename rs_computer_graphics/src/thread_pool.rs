use std::sync::{
    mpsc::{channel, sync_channel, Receiver, Sender, SyncSender, TryRecvError},
    Arc, Mutex,
};

lazy_static! {
    static ref GLOBAL_THREAD_POOL: Mutex<Arc<rayon::ThreadPool>> = Mutex::new(Arc::new(
        rayon::ThreadPoolBuilder::new()
            .thread_name(|_| { "Global".to_string() })
            .num_threads(std::thread::available_parallelism().unwrap().get())
            .build()
            .unwrap(),
    ));
    static ref GLOBAL_IO_THREAD_POOL: Mutex<Arc<rayon::ThreadPool>> = Mutex::new(Arc::new(
        rayon::ThreadPoolBuilder::new()
            .thread_name(|_| { "IO".to_string() })
            .num_threads(1)
            .build()
            .unwrap(),
    ));
    static ref GLOBAL_AUDIO_THREAD_POOL: Mutex<Arc<rayon::ThreadPool>> = Mutex::new(Arc::new(
        rayon::ThreadPoolBuilder::new()
            .thread_name(|_| { "Audio".to_string() })
            .num_threads(1)
            .build()
            .unwrap(),
    ));
    static ref VIRTUAL_TEXTURE_CACHE_THREAD_POOL: Mutex<Arc<rayon::ThreadPool>> =
        Mutex::new(Arc::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(1)
                .build()
                .unwrap(),
        ));
    static ref RENDER_THREAD_POOL: Mutex<Arc<rayon::ThreadPool>> = Mutex::new(Arc::new(
        rayon::ThreadPoolBuilder::new()
            .thread_name(|_| { "Render".to_string() })
            .num_threads(1)
            .build()
            .unwrap(),
    ));
    static ref VIDEO_DECODE_THREAD_POOL: Mutex<Arc<rayon::ThreadPool>> = Mutex::new(Arc::new(
        rayon::ThreadPoolBuilder::new()
            .thread_name(|_| { "Video Decode".to_string() })
            .num_threads(1)
            .build()
            .unwrap(),
    ));
}

pub enum SingleConsumeChnnelAPayload<T> {
    Message(T),
    Stop,
}

pub enum SingleConsumeChnnelBPayload<T> {
    Message(T),
    DidStop,
}

pub struct SingleConsumeChnnel<T, U> {
    a_thread_sender: Mutex<SyncSender<SingleConsumeChnnelAPayload<T>>>,
    b_thread_receiver: Mutex<Receiver<SingleConsumeChnnelAPayload<T>>>,
    b_thread_sender: Mutex<SyncSender<SingleConsumeChnnelBPayload<U>>>,
    a_thread_receiver: Mutex<Receiver<SingleConsumeChnnelBPayload<U>>>,
}

impl<T, U> SingleConsumeChnnel<T, U> {
    pub fn shared(
        a_bound: Option<usize>,
        b_bound: Option<usize>,
    ) -> Arc<SingleConsumeChnnel<T, U>> {
        let (a_thread_sender, b_thread_receiver) =
            sync_channel(a_bound.unwrap_or(u8::MAX as usize));
        let (b_thread_sender, a_receiver) = sync_channel(b_bound.unwrap_or(u8::MAX as usize));
        Arc::new(SingleConsumeChnnel::<T, U> {
            a_thread_sender: Mutex::new(a_thread_sender),
            b_thread_receiver: Mutex::new(b_thread_receiver),
            b_thread_sender: Mutex::new(b_thread_sender),
            a_thread_receiver: Mutex::new(a_receiver),
        })
    }

    pub fn send_stop_signal_and_wait(&self) {
        match self
            .a_thread_sender
            .lock()
            .unwrap()
            .send(SingleConsumeChnnelAPayload::Stop)
        {
            Ok(_) => loop {
                match self.a_thread_receiver.lock().unwrap().recv() {
                    Ok(payload) => match payload {
                        SingleConsumeChnnelBPayload::Message(_) => {}
                        SingleConsumeChnnelBPayload::DidStop => {
                            break;
                        }
                    },
                    Err(error) => panic!("{}", error),
                }
            },
            Err(error) => panic!("{}", error),
        }
    }

    pub fn from_b_try_recv(&self) -> Result<U, TryRecvError> {
        match self.a_thread_receiver.lock().unwrap().try_recv() {
            Ok(payload) => match payload {
                SingleConsumeChnnelBPayload::Message(message) => Ok(message),
                SingleConsumeChnnelBPayload::DidStop => panic!(),
            },
            Err(error) => Err(error),
        }
    }

    pub fn to_b(&self, message: T) {
        let _ = self
            .a_thread_sender
            .lock()
            .unwrap()
            .send(SingleConsumeChnnelAPayload::Message(message));
    }

    pub fn to_a(&self, message: U) {
        let _ = self
            .b_thread_sender
            .lock()
            .unwrap()
            .send(SingleConsumeChnnelBPayload::Message(message));
    }

    pub fn from_a_block_current_thread<F>(&self, mut closure: F)
    where
        F: FnMut(T) -> (),
    {
        loop {
            match self.b_thread_receiver.lock().unwrap().recv() {
                Ok(payload) => match payload {
                    SingleConsumeChnnelAPayload::Message(message) => {
                        closure(message);
                    }
                    SingleConsumeChnnelAPayload::Stop => {
                        break;
                    }
                },
                Err(error) => {
                    log::warn!("{error}");
                    break;
                }
            }
        }
        log::trace!("Thread exit.");
        let _ = self
            .b_thread_sender
            .lock()
            .unwrap()
            .send(SingleConsumeChnnelBPayload::DidStop);
    }
}

pub struct SyncWait {
    owned_thread_sender: Mutex<Sender<()>>,
    parallelism_thread_receiver: Mutex<Receiver<()>>,
    parallelism_thread_sender: Mutex<Sender<()>>,
    owned_thread_receiver: Mutex<Receiver<()>>,
}

impl SyncWait {
    pub fn shared() -> Arc<SyncWait> {
        let (owned_thread_sender, parallelism_thread_receiver) = channel::<()>();
        let (parallelism_thread_sender, owned_thread_receiver) = channel::<()>();
        Arc::new(SyncWait {
            owned_thread_sender: Mutex::new(owned_thread_sender),
            parallelism_thread_receiver: Mutex::new(parallelism_thread_receiver),
            parallelism_thread_sender: Mutex::new(parallelism_thread_sender),
            owned_thread_receiver: Mutex::new(owned_thread_receiver),
        })
    }

    pub fn send_stop_signal_and_wait(&self) {
        match self.owned_thread_sender.lock().unwrap().send(()) {
            Ok(_) => match self.owned_thread_receiver.lock().unwrap().recv() {
                Ok(_) => {}
                Err(error) => panic!("{}", error),
            },
            Err(error) => panic!("{}", error),
        }
    }

    pub fn is_stop(&self) -> bool {
        match self.parallelism_thread_receiver.lock().unwrap().try_recv() {
            Ok(_) => true,
            Err(error) => match error {
                std::sync::mpsc::TryRecvError::Empty => false,
                std::sync::mpsc::TryRecvError::Disconnected => true,
            },
        }
    }

    pub fn finish(&self) {
        match self.parallelism_thread_sender.lock().unwrap().send(()) {
            Ok(_) => {}
            Err(error) => panic!("{}", error),
        }
    }
}

pub struct MultipleSyncWait {
    inner: Arc<SyncWait>,
    count: Arc<Mutex<u32>>,
}

impl MultipleSyncWait {
    pub fn new() -> MultipleSyncWait {
        MultipleSyncWait {
            inner: SyncWait::shared(),
            count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn clone(&self) -> MultipleSyncWait {
        let count = self.count.clone();
        *count.lock().unwrap() += 1;
        MultipleSyncWait {
            inner: self.inner.clone(),
            count,
        }
    }

    pub fn send_stop_signal_and_wait(&self) {
        let mut count = *self.count.lock().unwrap();
        while count > 0 {
            self.inner.send_stop_signal_and_wait();
            count -= 1
        }
    }

    pub fn is_stop(&self) -> bool {
        self.inner.is_stop()
    }

    pub fn finish(&self) {
        self.inner.finish()
    }
}

pub struct ThreadPool {}

impl ThreadPool {
    pub fn global() -> Arc<rayon::ThreadPool> {
        GLOBAL_THREAD_POOL.lock().unwrap().clone()
    }

    pub fn io() -> Arc<rayon::ThreadPool> {
        GLOBAL_IO_THREAD_POOL.lock().unwrap().clone()
    }

    pub fn audio() -> Arc<rayon::ThreadPool> {
        GLOBAL_AUDIO_THREAD_POOL.lock().unwrap().clone()
    }

    pub fn render() -> Arc<rayon::ThreadPool> {
        RENDER_THREAD_POOL.lock().unwrap().clone()
    }

    pub fn virtual_texture_cache() -> Arc<rayon::ThreadPool> {
        VIRTUAL_TEXTURE_CACHE_THREAD_POOL.lock().unwrap().clone()
    }

    pub fn video_decode() -> Arc<rayon::ThreadPool> {
        VIDEO_DECODE_THREAD_POOL.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_case_multiple_sync_wait0() {
        let wait = MultipleSyncWait::new();
        for i in 1..=50 {
            ThreadPool::global().spawn({
                let wait = wait.clone();
                move || {
                    let is_stop = wait.is_stop();
                    if is_stop {
                        println!("Stop work. {}", i);
                    } else {
                        std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                        println!("Finish work. {}", i);
                    }
                    wait.finish();
                }
            });
        }
        wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_multiple_sync_wait1() {
        let wait = MultipleSyncWait::new();
        for i in 1..=50 {
            ThreadPool::global().spawn({
                let wait = wait.clone();
                move || {
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                    println!("Finish work. {}", i);
                    wait.finish();
                }
            });
        }
        wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_multiple_sync_wait2() {
        let wait = MultipleSyncWait::new();
        for i in 1..=10 {
            ThreadPool::global().spawn({
                let wait = wait.clone();
                move || {
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                    println!("Finish work. {}", i);
                    wait.finish();
                }
            });
        }
        std::thread::sleep(std::time::Duration::from_secs_f32(10.0f32));
        wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_multiple_sync_wait3() {
        let wait = MultipleSyncWait::new();
        for i in 1..=10 {
            ThreadPool::global().spawn({
                let wait = wait.clone();
                move || {
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                    println!("Finish work. {}", i);
                    wait.finish();
                }
            });
        }
        std::thread::sleep(std::time::Duration::from_secs_f32(10.0f32));
        wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_sync_wait0() {
        let sync_wait = SyncWait::shared();
        ThreadPool::render().spawn({
            let sync_wait = sync_wait.clone();
            move || {
                println!("Finish work. {}", sync_wait.is_stop());
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                println!("Finish work. {}", sync_wait.is_stop());
                sync_wait.finish();
            }
        });
        std::thread::sleep(std::time::Duration::from_secs_f32(3.0f32));
        sync_wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_sync_wait1() {
        let sync_wait = SyncWait::shared();
        ThreadPool::render().spawn({
            let sync_wait = sync_wait.clone();
            move || {
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                println!("Finish work.");
                sync_wait.finish();
            }
        });
        sync_wait.send_stop_signal_and_wait();
        println!("Exit.");
    }

    #[test]
    fn test_case_sync_wait2() {
        let sync_wait = SyncWait::shared();
        ThreadPool::render().spawn({
            let sync_wait = sync_wait.clone();
            move || {
                while sync_wait.is_stop() == false {
                    println!("Working.");
                    std::thread::sleep(std::time::Duration::from_secs_f32(1.0f32));
                }
                println!("Finish work.");
                sync_wait.finish();
            }
        });
        std::thread::sleep(std::time::Duration::from_secs_f32(3.0f32));
        sync_wait.send_stop_signal_and_wait();
        println!("Exit.");
    }
}
