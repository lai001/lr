use lazy_static::lazy_static;
use std::sync::{Arc, Mutex};

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
