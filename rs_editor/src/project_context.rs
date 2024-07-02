use crate::{
    build_config::{BuildConfig, EArchType, EBuildPlatformType, EBuildType},
    model_loader::ModelLoader,
    project::{Project, ASSET_FOLDER_NAME},
};
use anyhow::{anyhow, Context};
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::{DebouncedEvent, Debouncer};
use rs_artifact::{
    artifact::ArtifactAssetEncoder, shader_source_code::ShaderSourceCode, EEndianType,
};
use rs_engine::{
    content::content_file_type::EContentFileType, resource_manager::ResourceManager,
    thread_pool::ThreadPool, ASSET_SCHEME,
};
use rs_hotreload_plugin::hot_reload::HotReload;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
};

pub enum EFolderUpdateType {
    Asset,
}

#[derive(Serialize, Deserialize)]
pub struct RecentProjects {
    pub paths: HashSet<std::path::PathBuf>,
}

impl RecentProjects {
    pub fn load() -> RecentProjects {
        let path = Path::new("./recent_projects.json");
        if path.exists() {
            let file = std::fs::File::open(path).unwrap();
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        } else {
            RecentProjects {
                paths: HashSet::new(),
            }
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Path::new("./recent_projects.json");
        std::fs::write(path, serde_json::to_string(self)?)?;
        Ok(())
    }
}

pub struct ProjectContext {
    pub project: Project,
    project_folder_path: PathBuf,
    project_file_path: PathBuf,
    _shader_folder_path: PathBuf,
    pub hot_reload: rs_hotreload_plugin::hot_reload::HotReload,
    folder_receiver: Option<
        std::sync::mpsc::Receiver<std::result::Result<Vec<DebouncedEvent>, Vec<notify::Error>>>,
    >,
    folder_debouncer: Option<Debouncer<ReadDirectoryChangesWatcher>>,
}

impl ProjectContext {
    pub fn open(project_file_path: &Path) -> anyhow::Result<ProjectContext> {
        let project_folder_path =
            project_file_path
                .parent()
                .ok_or(crate::error::Error::OpenProjectFailed(Some(
                    "Can not find parent folder.".to_string(),
                )))?;
        let file = std::fs::File::open(project_file_path)
            .context(format!("Can not open file: {:?}", project_file_path))?;
        let reader = std::io::BufReader::new(file);
        let project: Project = serde_json::de::from_reader(reader)
            .context("Failed to deserialize JSON data to a project data structure.")?;
        #[cfg(debug_assertions)]
        let lib_folder = project_folder_path.join("target").join("debug");
        #[cfg(not(debug_assertions))]
        let lib_folder = project_folder_path.join("target").join("release");
        let hot_reload = HotReload::new(&project_folder_path, &lib_folder, &project.project_name)?;
        let mut context = ProjectContext {
            project,
            project_file_path: project_file_path.to_path_buf(),
            project_folder_path: project_folder_path.to_path_buf(),
            hot_reload,
            _shader_folder_path: project_folder_path.join("shader"),
            folder_receiver: None,
            folder_debouncer: None,
        };
        context.watch_project_folder()?;
        Ok(context)
    }

    fn watch_project_folder(&mut self) -> anyhow::Result<()> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let mut debouncer = notify_debouncer_mini::new_debouncer(
            std::time::Duration::from_millis(200),
            None,
            sender,
        )
        .map_err(|err| anyhow!("{:?}", err))?;
        let watch_folder_path = self.get_project_folder_path();

        debouncer.watcher().watch(
            &std::path::Path::new(&watch_folder_path),
            notify::RecursiveMode::Recursive,
        )?;
        self.folder_receiver = Some(receiver);
        self.folder_debouncer = Some(debouncer);
        log::trace!("Watch project folder. {:?}", watch_folder_path);
        Ok(())
    }

    pub fn check_folder_notification(&mut self) -> Option<EFolderUpdateType> {
        let asset_folder_path = self.get_asset_folder_path();
        let Some(receiver) = self.folder_receiver.as_mut() else {
            return None;
        };
        let mut is_need_update = false;
        for events in receiver.try_iter() {
            if is_need_update {
                break;
            }
            let Ok(events) = events else {
                continue;
            };
            for event in events {
                if event.path.starts_with(asset_folder_path.clone()) {
                    is_need_update = true;
                    break;
                }
            }
        }

        if is_need_update {
            return Some(EFolderUpdateType::Asset);
        }
        None
    }

    pub fn is_need_reload_plugin(&self) -> bool {
        self.hot_reload.is_need_reload()
    }

    pub fn reload(&mut self) -> anyhow::Result<()> {
        Ok(self.hot_reload.reload()?)
    }

    pub fn get_asset_folder_path(&self) -> PathBuf {
        self.project_folder_path.join(ASSET_FOLDER_NAME)
    }

    pub fn get_asset_path_by_url(&self, url: &url::Url) -> PathBuf {
        if url.scheme() != ASSET_SCHEME {
            panic!()
        }
        self.project_folder_path.join(
            url.to_string()
                .strip_prefix(&format!("{}://", ASSET_SCHEME))
                .unwrap(),
        )
    }

    pub fn copy_file_to_asset_folder(&self, path: &Path) -> anyhow::Result<()> {
        let file_name = path.file_name().ok_or(anyhow!("No file name"))?;
        let to = self.get_asset_folder_path().join(file_name);
        let _ = std::fs::copy(path, to.clone())?;
        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let json_str = serde_json::ser::to_string_pretty(&self.project)?;
        let mut file = std::fs::File::create(self.project_file_path.clone())?;
        Ok(file.write_fmt(format_args!("{}", json_str))?)
    }

    pub fn get_project_folder_path(&self) -> PathBuf {
        self.project_folder_path.clone()
    }

    pub fn create_build_folder_if_not_exist(
        &self,
        build_config: &BuildConfig,
    ) -> anyhow::Result<PathBuf> {
        let path = self.try_create_build_dir()?;
        let platform: String;
        let build_type: String;
        let arch: String;
        match build_config.build_platform {
            EBuildPlatformType::Windows => {
                platform = String::from("windows");
            }
        }
        match build_config.build_type {
            EBuildType::Debug => {
                build_type = String::from("debug");
            }
            EBuildType::Release => {
                build_type = String::from("release");
            }
        }
        match build_config.arch_type {
            EArchType::X64 => {
                arch = String::from("x64");
            }
        }
        let path = path.join(platform).join(build_type).join(arch);
        std::fs::create_dir_all(path.clone())?;
        Ok(path)
    }

    pub fn build_static_mesh_url(file_path: &Path, mesh_name: &str) -> url::Url {
        url::Url::parse(&format!(
            "asset://static_mesh/{}/{}",
            file_path.to_str().unwrap(),
            mesh_name
        ))
        .unwrap()
    }

    pub fn build_shader_url(name: &str) -> url::Url {
        url::Url::parse(&format!("asset://shader/{}", name)).unwrap()
    }

    pub fn get_build_dir(&self) -> PathBuf {
        self.project_folder_path.join("build")
    }

    pub fn try_create_build_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.get_build_dir();
        let _ = std::fs::create_dir_all(path.clone())?;
        Ok(path)
    }

    pub fn get_virtual_texture_cache_dir(&self) -> PathBuf {
        self.project_folder_path.join("build/cache/virtual_texture")
    }

    pub fn get_ibl_bake_cache_dir(&self, sub_folder: &Path) -> PathBuf {
        self.project_folder_path
            .join("build/cache/ibl")
            .join(sub_folder)
    }

    pub fn try_create_virtual_texture_cache_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.get_virtual_texture_cache_dir();
        let _ = std::fs::create_dir_all(path.clone())?;
        Ok(path)
    }

    pub fn try_create_ibl_bake_cache_dir(&self, sub_folder: &Path) -> anyhow::Result<PathBuf> {
        let path = self.get_ibl_bake_cache_dir(sub_folder);
        let _ = std::fs::create_dir_all(path.clone())
            .context(anyhow!("Can not create {:?}", path.clone()))?;
        Ok(path)
    }

    pub fn export(&mut self, model_loader: &mut ModelLoader) -> anyhow::Result<PathBuf> {
        let output_folder_path = self.try_create_build_dir()?;
        if !output_folder_path.exists() {
            std::fs::create_dir(output_folder_path.clone())?;
        }
        let output_filename = "main.rs";
        let project_folder_path = self.get_project_folder_path();

        let mut artifact_asset_encoder = ArtifactAssetEncoder::new(
            Some(EEndianType::Little),
            self.project.settings.borrow().clone(),
            &output_folder_path.join(output_filename),
        );

        let mut images: HashMap<url::Url, rs_artifact::image::Image> = HashMap::new();
        let mut shader_source_codes: HashMap<
            url::Url,
            rs_artifact::shader_source_code::ShaderSourceCode,
        > = HashMap::new();
        let static_meshes: HashMap<url::Url, rs_artifact::static_mesh::StaticMesh> = HashMap::new();
        let mut skin_meshes: HashMap<url::Url, rs_artifact::skin_mesh::SkinMesh> = HashMap::new();
        let mut skeletons: HashMap<url::Url, rs_artifact::skeleton::Skeleton> = HashMap::new();
        let mut skeleton_animations: HashMap<
            url::Url,
            rs_artifact::skeleton_animation::SkeletonAnimation,
        > = HashMap::new();
        let mut ibl_bakings: HashMap<url::Url, rs_artifact::ibl_baking::IBLBaking> = HashMap::new();
        let mut materials: HashMap<url::Url, rs_artifact::material::Material> = HashMap::new();
        let mut material_contents: HashMap<url::Url, rs_engine::content::material::Material> =
            HashMap::new();

        for file in &self.project.content.borrow().files {
            match file {
                EContentFileType::StaticMesh(asset) => {
                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::SkeletonMesh(asset) => {
                    let file_path = project_folder_path.join(&asset.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    let loaded_skin_mesh = model_loader.to_runtime_skin_mesh(
                        &asset.borrow(),
                        &project_folder_path,
                        ResourceManager::default(),
                    );
                    skin_meshes.insert(
                        loaded_skin_mesh.url.clone(),
                        loaded_skin_mesh.deref().clone(),
                    );

                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::SkeletonAnimation(asset) => {
                    let file_path = project_folder_path.join(&asset.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    let loaded_skeleton_animation = model_loader.to_runtime_skeleton_animation(
                        asset.clone(),
                        &project_folder_path,
                        ResourceManager::default(),
                    );
                    skeleton_animations.insert(
                        loaded_skeleton_animation.url.clone(),
                        loaded_skeleton_animation.deref().clone(),
                    );

                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::Skeleton(asset) => {
                    let file_path = project_folder_path.join(&asset.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    let loaded_skeleton = model_loader.to_runtime_skeleton(
                        asset.clone(),
                        &project_folder_path,
                        ResourceManager::default(),
                    );
                    skeletons.insert(loaded_skeleton.url.clone(), loaded_skeleton.deref().clone());

                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::Texture(asset) => {
                    let asset = asset.borrow();
                    if let Some(image_reference) = &asset.image_reference {
                        let absolute_image_file_path = self.get_asset_path_by_url(image_reference);

                        let buffer = std::fs::read(absolute_image_file_path.clone()).context(
                            format!("Failed to read from {:?}", absolute_image_file_path),
                        )?;
                        let _ = image::load_from_memory(&buffer).context(format!(
                            "{:?} is not a valid image file.",
                            absolute_image_file_path
                        ))?;
                        let format = image::guess_format(&buffer)?;
                        let image = rs_artifact::image::Image {
                            url: image_reference.clone(),
                            image_format: rs_artifact::image::ImageFormat::from_external_format(
                                format,
                            ),
                            data: buffer,
                        };
                        images.insert(image_reference.clone(), image);
                    }
                    artifact_asset_encoder.encode(&*asset);
                }
                EContentFileType::Level(asset) => {
                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::Material(material_content) => {
                    let find = self
                        .project
                        .materials
                        .iter()
                        .find(|x| x.borrow().url == material_content.borrow().asset_url)
                        .cloned();
                    if let Some(material_editor) = find {
                        if let Ok(resolve_result) =
                            crate::material_resolve::resolve(&material_editor.borrow().snarl)
                        {
                            materials.insert(
                                material_content.borrow().asset_url.clone(),
                                rs_artifact::material::Material {
                                    url: material_content.borrow().asset_url.clone(),
                                    code: resolve_result.shader_code,
                                    material_info: resolve_result.material_info,
                                },
                            );
                            material_contents.insert(
                                material_content.borrow().url.clone(),
                                rs_engine::content::material::Material::new(
                                    material_content.borrow().url.clone(),
                                    material_content.borrow().asset_url.clone(),
                                ),
                            );
                        }
                    }
                }
                EContentFileType::IBL(ibl) => {
                    let ibl_baking = (|| {
                        let url = ibl.borrow().url.clone();
                        let image_reference = &ibl.borrow().image_reference;
                        let Some(image_reference) = image_reference.as_ref() else {
                            return Ok(None);
                        };
                        let file_path = project_folder_path.join(image_reference);
                        if !file_path.exists() {
                            return Err(anyhow!("The file is not exist"));
                        }
                        if !self.get_ibl_bake_cache_dir(image_reference).exists() {
                            return Err(anyhow!("The file is not exist"));
                        }
                        let name = rs_engine::url_extension::UrlExtension::get_name_in_editor(&url);
                        let ibl_baking = rs_artifact::ibl_baking::IBLBaking {
                            name,
                            url: url.clone(),
                            brdf_data: std::fs::read(
                                self.get_ibl_bake_cache_dir(image_reference)
                                    .join("brdf.dds"),
                            )?,
                            pre_filter_data: std::fs::read(
                                self.get_ibl_bake_cache_dir(image_reference)
                                    .join("pre_filter.dds"),
                            )?,
                            irradiance_data: std::fs::read(
                                self.get_ibl_bake_cache_dir(image_reference)
                                    .join("irradiance.dds"),
                            )?,
                        };
                        Ok(Some(ibl_baking))
                    })()?;
                    if let Some(ibl_baking) = ibl_baking {
                        ibl_bakings.insert(ibl_baking.url.clone(), ibl_baking);
                    }
                }
                EContentFileType::ParticleSystem(_) => todo!(),
            }
        }

        for (name, code) in Self::pre_process_shaders() {
            let url = Self::build_shader_url(&name);
            let shader_source_code = ShaderSourceCode {
                name: name.clone(),
                id: uuid::Uuid::new_v4(),
                url: Self::build_shader_url(&name),
                code,
            };
            shader_source_codes.insert(url, shader_source_code);
        }

        // FIXME: Out of memory
        for asset in images.values() {
            artifact_asset_encoder.encode(asset);
        }
        for asset in shader_source_codes.values() {
            artifact_asset_encoder.encode(asset);
        }
        for asset in static_meshes.values() {
            artifact_asset_encoder.encode(asset);
        }
        for asset in skin_meshes.values() {
            artifact_asset_encoder.encode(asset);
        }
        for asset in skeletons.values() {
            artifact_asset_encoder.encode(asset);
        }
        for asset in skeleton_animations.values() {
            artifact_asset_encoder.encode(asset);
        }
        for asset in ibl_bakings.values() {
            artifact_asset_encoder.encode(asset);
        }
        for asset in materials.values() {
            artifact_asset_encoder.encode(asset);
        }
        for asset in material_contents.values() {
            artifact_asset_encoder.encode(asset);
        }

        let _ = artifact_asset_encoder.finish()?;
        Ok(output_folder_path.join(output_filename))
    }

    pub fn pre_process_shaders() -> HashMap<String, String> {
        let _span = tracy_client::span!();
        let mut shaders = HashMap::new();
        let buildin_shaders = rs_render::global_shaders::get_buildin_shaders();
        let (sender, receiver) = std::sync::mpsc::channel();
        struct TaskResult {
            name: String,
            code: anyhow::Result<String>,
        }
        let mut is_finish = buildin_shaders.len();
        for buildin_shader in buildin_shaders {
            ThreadPool::global().spawn({
                let description = buildin_shader.get_shader_description();
                let name = buildin_shader.get_name();
                let sender = sender.clone();
                move || {
                    let span = tracy_client::span!();
                    span.emit_text(&format!("Pre process shader: {}", name));
                    if rs_core_minimal::misc::is_dev_mode() {
                        let pre_process_code = rs_shader_compiler::pre_process::pre_process(
                            &description.shader_path,
                            description.include_dirs.iter(),
                            description.definitions.iter(),
                        );
                        let result = TaskResult {
                            name: name.clone(),
                            code: pre_process_code.map_err(|err| anyhow::Error::from(err)),
                        };
                        let _ = sender.send(result);
                    } else {
                        let path = rs_render::get_buildin_shader_dir().join(name.clone());
                        let code = std::fs::read_to_string(path.clone());
                        let result = TaskResult {
                            name: name.clone(),
                            code: code.map_err(|err| anyhow::Error::from(err)),
                        };
                        let _ = sender.send(result);
                    }
                }
            });
        }
        while let Ok(task_result) = receiver.recv() {
            let name = task_result.name;
            match task_result.code {
                Ok(code) => {
                    if shaders.insert(name.clone(), code).is_some() {
                        panic!("{} is already exists", name);
                    }
                }
                Err(err) => {
                    log::warn!("{}", err);
                }
            }
            is_finish -= 1;
            if is_finish == 0 {
                break;
            }
        }

        shaders
    }
}
