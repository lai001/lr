use crate::thread_pool::ThreadPool;
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};
use walkdir::WalkDir;

struct LoadResult {
    key: String,
    image: image::ImageResult<image::DynamicImage>,
}

struct STResourceManager {
    image_sync_cache: moka::sync::Cache<String, Arc<image::DynamicImage>>,
    // texture_sync_cache: moka::sync::Cache<String, Arc<wgpu::Texture>>,
}

impl STResourceManager {
    fn new() -> STResourceManager {
        STResourceManager {
            image_sync_cache: moka::sync::Cache::new(1000),
            // texture_sync_cache: moka::sync::Cache::new(1000),
        }
    }

    fn cache_image(&self, key: &str, image: Arc<image::DynamicImage>) {
        log::trace!("Cache image, key: {key}");
        self.image_sync_cache.insert(key.to_string(), image);
    }

    fn get_cache_image(&self, key: &str) -> Option<Arc<image::DynamicImage>> {
        self.image_sync_cache.get(key)
    }

    fn get_cache_or_load_image(&self, key: &str, path: &str) -> Option<Arc<image::DynamicImage>> {
        if !self.image_sync_cache.contains_key(key) {
            self.load_image_from_disk_and_cache(key, path);
        }
        self.image_sync_cache.get(key)
    }

    fn load_image_from_disk_and_cache(&self, key: &str, path: &str) {
        let image = image::open(path);
        match image {
            Ok(image) => {
                self.cache_image(key, Arc::new(image));
            }
            Err(error) => {
                log::warn!("Load image failed, {}", error);
            }
        }
    }

    fn load_images_from_disk_and_cache_parallel(&self, dic: HashMap<&str, &str>) {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut count = dic.len();
        for (key, path) in dic {
            ThreadPool::global().spawn({
                let path = path.to_string();
                let key = key.to_string();
                let sender = sender.clone();
                move || {
                    let _ = sender.send(LoadResult {
                        key,
                        image: image::open(path),
                    });
                }
            });
        }
        while count > 0 {
            match receiver.recv() {
                Ok(result) => {
                    match result.image {
                        Ok(image) => self.cache_image(&result.key, Arc::new(image)),
                        Err(error) => log::warn!("{error}"),
                    }
                    count -= 1;
                }
                Err(error) => {
                    log::warn!("{}", error);
                }
            }
        }
    }

    fn is_available_extension(extension: &str) -> bool {
        match extension {
            "png" => true,
            "jpg" => true,
            "exr" => true,
            _ => false,
        }
    }

    fn preload_from_disk(&self, dir: &str) {
        let dir_path = Path::new(dir);
        for entry in WalkDir::new(dir.clone()) {
            if let Ok(entry) = entry {
                if let Some(extension) = entry.path().extension() {
                    if Self::is_available_extension(&extension.to_str().unwrap()) {
                        if let Some(path) = entry.path().to_str() {
                            let key = Path::new(path);
                            let key = key
                                .strip_prefix(dir_path)
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string();
                            self.load_image_from_disk_and_cache(&key, path);
                        }
                    }
                }
            }
        }
    }
}

pub struct ResourceManager {
    inner: Mutex<STResourceManager>,
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager {
            inner: Mutex::new(STResourceManager::new()),
        }
    }

    pub fn default() -> Arc<ResourceManager> {
        GLOBAL_RESOURCE_MANAGER.clone()
    }

    pub fn cache_image(&mut self, key: &str, image: Arc<image::DynamicImage>) {
        self.inner.lock().unwrap().cache_image(key, image);
    }

    pub fn get_cache_image(&self, key: &str) -> Option<Arc<image::DynamicImage>> {
        self.inner.lock().unwrap().get_cache_image(key)
    }

    pub fn get_cache_or_load_image(
        &self,
        key: &str,
        path: &str,
    ) -> Option<Arc<image::DynamicImage>> {
        self.inner
            .lock()
            .unwrap()
            .get_cache_or_load_image(key, path)
    }

    pub fn load_image_from_disk_and_cache(&self, key: &str, path: &str) {
        self.inner
            .lock()
            .unwrap()
            .load_image_from_disk_and_cache(key, path);
    }

    pub fn load_images_from_disk_and_cache_parallel(&self, dic: HashMap<&str, &str>) {
        self.inner
            .lock()
            .unwrap()
            .load_images_from_disk_and_cache_parallel(dic);
    }
}

lazy_static! {
    static ref GLOBAL_RESOURCE_MANAGER: Arc<ResourceManager> = Arc::new(ResourceManager::new());
}
