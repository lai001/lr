use crate::content::level::Level;
use crate::engine::Engine;
use crate::handle::SamplerHandle;
use crate::thread_pool::ThreadPool;
use crate::{error::Result, handle::HandleManager};
use lazy_static::lazy_static;
use rs_artifact::asset::Asset;
use rs_artifact::resource_info::ResourceInfo;
use rs_artifact::sound::Sound;
use rs_artifact::static_mesh::StaticMesh;
use rs_artifact::{
    artifact::ArtifactReader, resource_type::EResourceType, shader_source_code::ShaderSourceCode,
};
use rs_core_minimal::name_generator;
use rs_render::command::IBLTexturesKey;
use std::collections::VecDeque;
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};

struct LoadResult {
    key: String,
    image: image::ImageResult<image::DynamicImage>,
}

#[derive(Clone)]
pub struct IBLTextures {
    pub brdflut: crate::handle::TextureHandle,
    pub pre_filter_cube_map: crate::handle::TextureHandle,
    pub irradiance: crate::handle::TextureHandle,
}

impl IBLTextures {
    pub fn to_key(&self) -> IBLTexturesKey {
        IBLTexturesKey {
            brdflut_texture: *self.brdflut,
            pre_filter_cube_map_texture: *self.pre_filter_cube_map,
            irradiance_texture: *self.irradiance,
        }
    }
}

#[derive(Clone)]
pub struct BuiltinResources {
    pub global_sampler_handle: SamplerHandle,
}

struct STResourceManager {
    image_sync_cache: moka::sync::Cache<String, Arc<image::DynamicImage>>,
    textures: HashMap<url::Url, crate::handle::TextureHandle>,
    ui_textures: HashMap<url::Url, crate::handle::EGUITextureHandle>,
    virtual_textures: HashMap<url::Url, crate::handle::TextureHandle>,
    artifact_reader: Option<ArtifactReader>,
    handle_manager: HandleManager,
    static_meshs: HashMap<url::Url, Arc<StaticMesh>>,
    skin_meshes: HashMap<url::Url, Arc<rs_artifact::skin_mesh::SkinMesh>>,
    skeleton_animations: HashMap<url::Url, Arc<rs_artifact::skeleton_animation::SkeletonAnimation>>,
    skeletons: HashMap<url::Url, Arc<rs_artifact::skeleton::Skeleton>>,
    ibl_textures: HashMap<url::Url, IBLTextures>,
    // mesh_buffers: HashMap<url::Url, Arc<MeshBuffer>>,
    // material_render_pipelines: HashMap<url::Url, crate::handle::MaterialRenderPipelineHandle>,
    pending_destroy_textures: Vec<crate::handle::TextureHandle>,

    buffer_handles: VecDeque<crate::handle::BufferHandle>,

    sounds: HashMap<url::Url, Arc<Sound>>,

    builtin_resources: Option<Arc<BuiltinResources>>,
}

impl STResourceManager {
    fn new() -> STResourceManager {
        STResourceManager {
            image_sync_cache: moka::sync::Cache::new(1000),
            textures: HashMap::new(),
            virtual_textures: HashMap::new(),
            artifact_reader: None,
            handle_manager: HandleManager::new(),
            static_meshs: HashMap::new(),
            skin_meshes: HashMap::new(),
            skeleton_animations: HashMap::new(),
            skeletons: HashMap::new(),
            ibl_textures: HashMap::new(),
            ui_textures: HashMap::new(),
            pending_destroy_textures: vec![],
            buffer_handles: VecDeque::new(),
            sounds: HashMap::new(),
            builtin_resources: None,
            // mesh_buffers: HashMap::new(),
            // material_render_pipelines: HashMap::new(),
        }
    }

    fn create_builtin_resources(&mut self, engine: &mut Engine) {
        assert!(self.builtin_resources.is_none());
        let global_sampler_handle = self.next_sampler();
        let command =
            rs_render::command::RenderCommand::CreateSampler(rs_render::command::CreateSampler {
                handle: *global_sampler_handle,
                sampler_descriptor: wgpu::SamplerDescriptor::default(),
            });
        engine.send_render_command(command);
        let builtin_resources = BuiltinResources {
            global_sampler_handle,
        };
        self.builtin_resources = Some(Arc::new(builtin_resources));
    }

    fn get_builtin_resources(&self) -> Arc<BuiltinResources> {
        self.builtin_resources.clone().unwrap().clone()
    }

    fn add_sound(&mut self, url: url::Url, sound: Arc<Sound>) -> Option<Arc<Sound>> {
        self.sounds.insert(url, sound)
    }

    fn get_sound(&self, url: &url::Url) -> Option<Arc<Sound>> {
        self.sounds.get(url).cloned()
    }

    fn add_skin_mesh(
        &mut self,
        url: url::Url,
        skin_mesh: Arc<rs_artifact::skin_mesh::SkinMesh>,
    ) -> Option<Arc<rs_artifact::skin_mesh::SkinMesh>> {
        self.skin_meshes.insert(url, skin_mesh)
    }

    fn get_skin_mesh(&mut self, url: &url::Url) -> Option<Arc<rs_artifact::skin_mesh::SkinMesh>> {
        if let Some(skin_meshe) = self.skin_meshes.get(url) {
            return Some(skin_meshe.clone());
        }
        None
    }

    fn add_skeleton_animation(
        &mut self,
        url: url::Url,
        skin_animation: Arc<rs_artifact::skeleton_animation::SkeletonAnimation>,
    ) -> Option<Arc<rs_artifact::skeleton_animation::SkeletonAnimation>> {
        self.skeleton_animations.insert(url, skin_animation)
    }

    fn get_skeleton_animation(
        &mut self,
        url: &url::Url,
    ) -> Option<Arc<rs_artifact::skeleton_animation::SkeletonAnimation>> {
        if let Some(skeleton_animation) = self.skeleton_animations.get(url) {
            return Some(skeleton_animation.clone());
        }
        None
    }

    fn add_skeleton(
        &mut self,
        url: url::Url,
        skeleton: Arc<rs_artifact::skeleton::Skeleton>,
    ) -> Option<Arc<rs_artifact::skeleton::Skeleton>> {
        self.skeletons.insert(url, skeleton)
    }

    fn get_skeleton(&mut self, url: &url::Url) -> Option<Arc<rs_artifact::skeleton::Skeleton>> {
        if let Some(skeleton) = self.skeletons.get(url) {
            return Some(skeleton.clone());
        }

        None
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
            .get_resource::<Level>(
                url,
                Some(EResourceType::Content(
                    rs_artifact::content_type::EContentType::Level,
                )),
            )
            .map_err(|err| crate::error::Error::Artifact(err, None))?;
        Ok(level)
    }

    fn add_static_mesh(&mut self, url: url::Url, mesh: Arc<StaticMesh>) -> Option<Arc<StaticMesh>> {
        self.static_meshs.insert(url, mesh)
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
        if let Some(pending_destroy_texture) = self.textures.insert(url, handle.clone()) {
            self.pending_destroy_textures.push(pending_destroy_texture);
        }
        handle
    }

    fn next_ibl_textures(&mut self, url: url::Url) -> IBLTextures {
        let brdflut = self.handle_manager.next_texture();
        let pre_filter_cube_map = self.handle_manager.next_texture();
        let irradiance = self.handle_manager.next_texture();
        let ibl_textures = IBLTextures {
            brdflut,
            pre_filter_cube_map,
            irradiance,
        };
        self.ibl_textures.insert(url, ibl_textures.clone());
        ibl_textures
    }

    fn next_virtual_texture(&mut self, url: url::Url) -> crate::handle::TextureHandle {
        let handle = self.handle_manager.next_virtual_texture();
        self.virtual_textures.insert(url, handle.clone());
        handle
    }

    fn next_ui_texture(&mut self, url: url::Url) -> crate::handle::EGUITextureHandle {
        let handle = self.handle_manager.next_ui_texture();
        self.ui_textures.insert(url, handle.clone());
        handle
    }

    fn next_buffer(&mut self) -> crate::handle::BufferHandle {
        let handle = self.handle_manager.next_buffer();
        self.buffer_handles.push_back(handle.clone());
        handle
    }

    fn next_sampler(&mut self) -> crate::handle::SamplerHandle {
        self.handle_manager.next_sampler()
    }

    fn next_material_render_pipeline(&mut self) -> crate::handle::MaterialRenderPipelineHandle {
        self.handle_manager.next_material_render_pipeline()
    }

    fn get_texture_by_url(&self, url: &url::Url) -> Option<crate::handle::TextureHandle> {
        self.textures.get(url).cloned()
    }

    fn get_ui_texture_by_url(&self, url: &url::Url) -> Option<crate::handle::EGUITextureHandle> {
        self.ui_textures.get(url).cloned()
    }

    fn get_virtual_texture_by_url(&self, url: &url::Url) -> Option<crate::handle::TextureHandle> {
        self.virtual_textures.get(url).cloned()
    }

    fn get_ibl_textures(&self) -> HashMap<url::Url, IBLTextures> {
        self.ibl_textures.clone()
    }

    fn get_texture_urls(&self) -> Vec<url::Url> {
        self.textures
            .keys()
            .map(|x| x.clone())
            .collect::<Vec<url::Url>>()
    }

    fn get_pending_destroy_textures(&self) -> Vec<crate::handle::TextureHandle> {
        self.pending_destroy_textures.clone()
    }

    fn collect_buffer_handles_release(&mut self) -> Vec<crate::handle::BufferHandle> {
        let mut handles = vec![];
        self.buffer_handles.retain_mut(|handle| {
            if handle.only_self() {
                handles.push(handle.clone());
                false
            } else {
                true
            }
        });
        handles
    }

    fn get_unique_texture_url(&self, url: &url::Url) -> url::Url {
        let names = self.textures.keys().map(|x| x.to_string()).collect();
        let new_name = url.to_string();
        let new_name = name_generator::make_unique_name(names, new_name);
        url::Url::parse(&new_name).expect(&format!("The new name ({}) should be unique", new_name))
    }
}

#[derive(Clone, rs_proc_macros::MultipleThreadFunctionsGenerator)]
#[file("rs_engine/src/resource_manager.rs", "STResourceManager")]
#[ignore_functions("new")]
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
}

lazy_static! {
    static ref GLOBAL_RESOURCE_MANAGER: ResourceManager = ResourceManager::new();
}
