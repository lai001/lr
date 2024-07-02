use anyhow::anyhow;
use rs_core_minimal::{misc::get_md5_from_string, path_ext::CanonicalizeSlashExt};
use rs_engine::thread_pool::ThreadPool;
use rs_foundation::new::{MultipleThreadMut, MultipleThreadMutType};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

pub struct ThumbnailCache {
    cache: MultipleThreadMutType<HashMap<PathBuf, PathBuf>>,
    image_loading_paths: MultipleThreadMutType<Vec<PathBuf>>,
}

impl ThumbnailCache {
    pub fn new() -> ThumbnailCache {
        ThumbnailCache {
            cache: MultipleThreadMut::new(HashMap::new()),
            image_loading_paths: MultipleThreadMut::new(Vec::new()),
        }
    }

    pub fn load_image(&mut self, image_path: &Path) {
        ThreadPool::global().spawn({
            let cache = self.cache.clone();
            let image_loading_paths = self.image_loading_paths.clone();
            let image_path = image_path.to_path_buf();
            move || {
                let is_loading = {
                    let image_loading_paths = image_loading_paths.lock().unwrap();
                    image_loading_paths.contains(&image_path)
                };
                let is_loaded = {
                    let cache = cache.lock().unwrap();
                    cache.contains_key(&image_path)
                };
                if is_loaded || is_loading {
                    return;
                }
                {
                    let mut image_loading_paths = image_loading_paths.lock().unwrap();
                    image_loading_paths.push(image_path.clone());
                }
                let load_result = Self::load_image_thread(&image_path);
                match load_result {
                    Ok(path) => {
                        let mut cache = cache.lock().unwrap();
                        cache.insert(image_path.clone(), path);
                        let mut image_loading_paths = image_loading_paths.lock().unwrap();
                        image_loading_paths.retain(|x| *x != image_path);
                    }
                    Err(err) => {
                        log::warn!("{}", err);
                    }
                }
            }
        });
    }

    pub fn get_image_file_uri(&self, image_path: &Path) -> Option<String> {
        let cache = self.cache.lock().unwrap();
        let path = match cache.get(image_path) {
            Some(path) => path.canonicalize_slash(),
            None => return None,
        };
        let path = match path {
            Ok(path) => path,
            Err(err) => {
                log::warn!("{:?}", err);
                return None;
            }
        };
        let url = match url::Url::from_file_path(&path) {
            Ok(url) => url,
            Err(_) => {
                log::warn!(
                    "Can not convert ot file scheme, {:?}",
                    path.canonicalize_slash()
                );
                return None;
            }
        };
        Some(url.to_string())
    }

    fn load_image_thread(image_path: &Path) -> anyhow::Result<PathBuf> {
        let span = tracy_client::span!();
        span.emit_text(&format!("load image from path done: {:?}", image_path));
        let mut image = image::open(image_path)?;
        image = image.thumbnail(50, 50);
        image = image::DynamicImage::ImageRgba8(image.to_rgba8());
        let filename = get_md5_from_string(image_path.to_str().ok_or(anyhow!(""))?);
        let file_path = Path::new("tmp").join(format!("{}.png", filename));
        if !Path::new("tmp").exists() {
            std::fs::create_dir("tmp")?;
        }
        image.save(file_path.clone())?;
        Ok(file_path)
    }
}
