use std::sync::{Arc, Mutex};

lazy_static! {
    static ref GLOBAL_THREAD_POOL: Arc<Mutex<rayon::ThreadPool>> = Arc::new(Mutex::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(std::thread::available_parallelism().unwrap().get())
            .build()
            .unwrap(),
    ));
    static ref GLOBAL_IO_THREAD_POOL: Arc<Mutex<rayon::ThreadPool>> = Arc::new(Mutex::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap(),
    ));
    static ref GLOBAL_AUDIO_THREAD_POOL: Arc<Mutex<rayon::ThreadPool>> = Arc::new(Mutex::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build()
            .unwrap(),
    ));
    static ref VIRTUAL_TEXTURE_CACHE_THREAD_POOL: Arc<Mutex<rayon::ThreadPool>> =
        Arc::new(Mutex::new(
            rayon::ThreadPoolBuilder::new()
                .num_threads(1)
                .build()
                .unwrap(),
        ));
}

pub struct ThreadPool {}

impl ThreadPool {
    pub fn global() -> Arc<Mutex<rayon::ThreadPool>> {
        GLOBAL_THREAD_POOL.clone()
    }

    pub fn io() -> Arc<Mutex<rayon::ThreadPool>> {
        GLOBAL_IO_THREAD_POOL.clone()
    }

    pub fn audio() -> Arc<Mutex<rayon::ThreadPool>> {
        GLOBAL_AUDIO_THREAD_POOL.clone()
    }

    pub fn virtual_texture_cache() -> Arc<Mutex<rayon::ThreadPool>> {
        VIRTUAL_TEXTURE_CACHE_THREAD_POOL.clone()
    }
}
