use crate::file_manager::FileManager;
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use walkdir::WalkDir;

lazy_static! {
    static ref GLOBAL_RESOURCE_MANAGER: Arc<Mutex<ResourceManager>> =
        Arc::new(Mutex::new(ResourceManager::new()));
}

pub struct ResourceManager {
    image_sync_cache: moka::sync::Cache<String, Arc<image::DynamicImage>>,
    texture_sync_cache: moka::sync::Cache<String, Arc<wgpu::Texture>>,
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager {
            image_sync_cache: moka::sync::Cache::new(1000),
            texture_sync_cache: moka::sync::Cache::new(1000),
        }
    }

    pub fn default() -> Arc<Mutex<ResourceManager>> {
        let rm = GLOBAL_RESOURCE_MANAGER.clone();
        // rm.lock().unwrap().preload_from_disk(
        //     &FileManager::default()
        //         .lock()
        //         .unwrap()
        //         .get_resource_dir_path(),
        // );
        rm
    }

    pub fn cache_image(&mut self, key: &str, image: Arc<image::DynamicImage>) {
        self.image_sync_cache.insert(key.to_string(), image);
    }

    pub fn get_cache_image(&self, key: &str) -> Option<Arc<image::DynamicImage>> {
        self.image_sync_cache.get(key)
    }

    pub fn get_cache_or_load_image(&mut self, key: &str, path: &str) -> Option<Arc<image::DynamicImage>> {
        if !self.image_sync_cache.contains_key(key) {
            self.load_image_from_disk_and_cache(key, path);
        }
        self.image_sync_cache.get(key)
    }

    pub fn load_image_from_disk_and_cache(&mut self, key: &str, path: &str) {
        let image = image::open(path);
        match image {
            Ok(image) => {
                log::trace!("Cache image, key: {}", &key);
                self.image_sync_cache
                    .insert(key.to_string(), Arc::new(image));
            }
            Err(error) => {
                log::warn!("Load image failed, {}", error);
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

    fn preload_from_disk(&mut self, dir: &str) {
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
