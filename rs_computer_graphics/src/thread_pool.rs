use std::sync::Mutex;

lazy_static! {
    pub static ref GLOBAL_THREAD_POOL: Mutex<rayon::ThreadPool> = Mutex::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(std::thread::available_parallelism().unwrap().get())
            .build()
            .unwrap(),
    );
    pub static ref GLOBAL_IO_THREAD_POOL: Mutex<rayon::ThreadPool> = Mutex::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .unwrap(),
    );
}
