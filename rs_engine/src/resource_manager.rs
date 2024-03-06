use crate::thread_pool::ThreadPool;
use crate::{error::Result, handle::HandleManager};
use lazy_static::lazy_static;
use rs_artifact::asset::Asset;
use rs_artifact::level::Level;
use rs_artifact::resource_info::ResourceInfo;
use rs_artifact::static_mesh::StaticMesh;
use rs_artifact::{
    artifact::ArtifactReader, resource_type::EResourceType, shader_source_code::ShaderSourceCode,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

struct LoadResult {
    key: String,
    image: image::ImageResult<image::DynamicImage>,
}

struct STResourceManager {
    image_sync_cache: moka::sync::Cache<String, Arc<image::DynamicImage>>,
    textures: HashMap<url::Url, crate::handle::TextureHandle>,
    virtual_textures: HashMap<url::Url, crate::handle::TextureHandle>,
    artifact_reader: Option<ArtifactReader>,
    handle_manager: HandleManager,
    static_meshs: HashMap<url::Url, Arc<StaticMesh>>,
}

impl STResourceManager {
    fn new() -> STResourceManager {
        STResourceManager {
            image_sync_cache: moka::sync::Cache::new(1000),
            artifact_reader: None,
            handle_manager: HandleManager::new(),
            static_meshs: HashMap::new(),
            textures: HashMap::new(),
            virtual_textures: HashMap::new(),
        }
    }

    fn load_static_meshs(&mut self) {
        let Some(reader) = self.artifact_reader.as_mut() else {
            return;
        };

        for (url, resource_info) in reader.get_artifact_file_header().resource_map.clone() {
            if resource_info.resource_type != EResourceType::StaticMesh {
                continue;
            }
            let static_mesh = reader
                .get_resource::<rs_artifact::static_mesh::StaticMesh>(
                    &url,
                    Some(EResourceType::StaticMesh),
                )
                .expect("Never");
            self.static_meshs
                .insert(static_mesh.url.clone(), Arc::new(static_mesh));
        }
    }

    fn get_shader_source_code(&mut self, url: &url::Url) -> Result<ShaderSourceCode> {
        let reader = self
            .artifact_reader
            .as_mut()
            .ok_or(crate::error::Error::ArtifactReaderNotSet)?;
        let shader = reader
            .get_resource::<rs_artifact::shader_source_code::ShaderSourceCode>(
                url,
                Some(EResourceType::ShaderSourceCode),
            )
            .map_err(|err| crate::error::Error::Artifact(err, None))?;
        Ok(shader)
    }

    fn get_level(&mut self, url: &url::Url) -> Result<Level> {
        let reader = self
            .artifact_reader
            .as_mut()
            .ok_or(crate::error::Error::ArtifactReaderNotSet)?;
        let level = reader
            .get_resource::<rs_artifact::level::Level>(url, Some(EResourceType::Level))
            .map_err(|err| crate::error::Error::Artifact(err, None))?;
        Ok(level)
    }

    fn get_static_mesh(&mut self, url: &url::Url) -> Result<Arc<StaticMesh>> {
        if let Some(loaded_mesh) = self.static_meshs.get(url) {
            return Ok(loaded_mesh.clone());
        }
        let reader = self
            .artifact_reader
            .as_mut()
            .ok_or(crate::error::Error::ArtifactReaderNotSet)?;
        let static_mesh = reader
            .get_resource::<rs_artifact::static_mesh::StaticMesh>(
                url,
                Some(EResourceType::StaticMesh),
            )
            .map_err(|err| crate::error::Error::Artifact(err, None))?;
        let static_mesh = Arc::new(static_mesh);
        self.static_meshs.insert(url.clone(), static_mesh.clone());
        Ok(static_mesh)
    }

    fn get_resource_map(&self) -> Result<HashMap<url::Url, ResourceInfo>> {
        let reader = self
            .artifact_reader
            .as_ref()
            .ok_or(crate::error::Error::ArtifactReaderNotSet)?;
        Ok(reader.get_artifact_file_header().resource_map.clone())
    }

    fn get_resource<T: Asset>(
        &mut self,
        url: &url::Url,
        expected_resource_type: Option<EResourceType>,
    ) -> Result<T> {
        let reader = self
            .artifact_reader
            .as_mut()
            .ok_or(crate::error::Error::ArtifactReaderNotSet)?;
        let level = reader
            .get_resource::<T>(url, expected_resource_type)
            .map_err(|err| crate::error::Error::Artifact(err, None))?;
        Ok(level)
    }

    fn get_all_shader_source_codes(&mut self) -> Vec<ShaderSourceCode> {
        let mut codes: Vec<ShaderSourceCode> = vec![];
        let Some(reader) = self.artifact_reader.as_mut() else {
            return codes;
        };
        for (url, resource_info) in reader.get_artifact_file_header().resource_map.clone() {
            if resource_info.resource_type != EResourceType::ShaderSourceCode {
                continue;
            }
            let shader = reader
                .get_resource::<rs_artifact::shader_source_code::ShaderSourceCode>(
                    &url,
                    Some(EResourceType::ShaderSourceCode),
                )
                .expect("Never");
            codes.push(shader);
        }
        codes
    }

    fn set_artifact_reader(&mut self, reader: Option<ArtifactReader>) {
        self.artifact_reader = reader;
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
            let result = (|| {
                let result = receiver
                    .recv()
                    .map_err(|err| crate::error::Error::RecvError(err))?;
                let image = result
                    .image
                    .map_err(|err| crate::error::Error::ImageError(err, None))?;
                self.cache_image(&result.key, Arc::new(image));
                Ok::<(), crate::error::Error>(())
            })();
            log::trace!("{:?}", result);
            count -= 1;
        }
    }

    fn next_texture(&mut self, url: url::Url) -> crate::handle::TextureHandle {
        let handle = self.handle_manager.next_texture();
        self.textures.insert(url, handle.clone());
        handle
    }

    fn next_virtual_texture(&mut self, url: url::Url) -> crate::handle::TextureHandle {
        let handle = self.handle_manager.next_virtual_texture();
        self.virtual_textures.insert(url, handle.clone());
        handle
    }

    fn next_ui_texture(&mut self) -> crate::handle::EGUITextureHandle {
        self.handle_manager.next_ui_texture()
    }

    fn next_buffer(&mut self) -> crate::handle::BufferHandle {
        self.handle_manager.next_buffer()
    }

    fn get_texture_by_url(&self, url: &url::Url) -> Option<crate::handle::TextureHandle> {
        self.textures.get(url).cloned()
    }

    fn get_virtual_texture_by_url(&self, url: &url::Url) -> Option<crate::handle::TextureHandle> {
        self.virtual_textures.get(url).cloned()
    }
}

#[derive(Clone)]
pub struct ResourceManager {
    inner: Arc<Mutex<STResourceManager>>,
}

impl ResourceManager {
    pub fn new() -> ResourceManager {
        ResourceManager {
            inner: Arc::new(Mutex::new(STResourceManager::new())),
        }
    }

    pub fn default() -> ResourceManager {
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

    pub fn set_artifact_reader(&mut self, reader: Option<ArtifactReader>) {
        self.inner.lock().unwrap().set_artifact_reader(reader);
    }

    pub fn get_shader_source_code(&mut self, url: &url::Url) -> Result<ShaderSourceCode> {
        self.inner.lock().unwrap().get_shader_source_code(url)
    }

    pub fn get_all_shader_source_codes(&mut self) -> Vec<ShaderSourceCode> {
        self.inner.lock().unwrap().get_all_shader_source_codes()
    }

    pub fn next_texture(&mut self, url: url::Url) -> crate::handle::TextureHandle {
        self.inner.lock().unwrap().next_texture(url)
    }

    pub fn next_ui_texture(&mut self) -> crate::handle::EGUITextureHandle {
        self.inner.lock().unwrap().next_ui_texture()
    }

    pub fn next_buffer(&mut self) -> crate::handle::BufferHandle {
        self.inner.lock().unwrap().next_buffer()
    }

    pub fn get_level(&mut self, url: &url::Url) -> Result<Level> {
        self.inner.lock().unwrap().get_level(url)
    }

    pub fn get_resource_map(&self) -> Result<HashMap<url::Url, ResourceInfo>> {
        self.inner.lock().unwrap().get_resource_map()
    }

    pub fn get_resource<T: Asset>(
        &mut self,
        url: &url::Url,
        expected_resource_type: Option<EResourceType>,
    ) -> Result<T> {
        self.inner
            .lock()
            .unwrap()
            .get_resource(url, expected_resource_type)
    }

    pub fn get_static_mesh(&mut self, url: &url::Url) -> Result<Arc<StaticMesh>> {
        self.inner.lock().unwrap().get_static_mesh(url)
    }

    pub fn load_static_meshs(&mut self) {
        self.inner.lock().unwrap().load_static_meshs();
    }

    pub fn get_texture_by_url(&self, url: &url::Url) -> Option<crate::handle::TextureHandle> {
        self.inner.lock().unwrap().get_texture_by_url(url)
    }

    pub fn next_virtual_texture(&mut self, url: url::Url) -> crate::handle::TextureHandle {
        self.inner.lock().unwrap().next_virtual_texture(url)
    }

    pub fn get_virtual_texture_by_url(
        &self,
        url: &url::Url,
    ) -> Option<crate::handle::TextureHandle> {
        self.inner.lock().unwrap().get_virtual_texture_by_url(url)
    }
}

lazy_static! {
    static ref GLOBAL_RESOURCE_MANAGER: ResourceManager = ResourceManager::new();
}
