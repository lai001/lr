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
    content::{content_file_type::EContentFileType, texture::TextureFile},
    resource_manager::ResourceManager,
};
use rs_hotreload_plugin::hot_reload::HotReload;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    io::Write,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
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
    shader_folder_path: PathBuf,
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
            shader_folder_path: project_folder_path.join("shader"),
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

    pub fn try_create_virtual_texture_cache_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.get_virtual_texture_cache_dir();
        let _ = std::fs::create_dir_all(path.clone())?;
        Ok(path)
    }

    pub fn export(&mut self, model_loader: &mut ModelLoader) -> anyhow::Result<PathBuf> {
        let output_folder_path = self.try_create_build_dir()?;
        if !output_folder_path.exists() {
            std::fs::create_dir(output_folder_path.clone())?;
        }
        let output_filename = "main.rs";
        let asset_folder_path = self.get_asset_folder_path();

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
        let mut static_meshes: HashMap<url::Url, rs_artifact::static_mesh::StaticMesh> =
            HashMap::new();
        let mut skin_meshes: HashMap<url::Url, rs_artifact::skin_mesh::SkinMesh> = HashMap::new();
        let mut skeletons: HashMap<url::Url, rs_artifact::skeleton::Skeleton> = HashMap::new();
        let mut skeleton_animations: HashMap<
            url::Url,
            rs_artifact::skeleton_animation::SkeletonAnimation,
        > = HashMap::new();

        for file in &self.project.content.borrow().files {
            match file {
                EContentFileType::StaticMesh(asset) => {
                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::SkeletonMesh(asset) => {
                    let file_path = asset_folder_path.join(&asset.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    let loaded_skin_mesh = model_loader.to_runtime_skin_mesh(
                        asset.clone(),
                        &asset_folder_path,
                        ResourceManager::default(),
                    );
                    skin_meshes.insert(
                        loaded_skin_mesh.url.clone(),
                        loaded_skin_mesh.deref().clone(),
                    );

                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::SkeletonAnimation(asset) => {
                    let file_path = asset_folder_path.join(&asset.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    let loaded_skeleton_animation = model_loader.to_runtime_skeleton_animation(
                        asset.clone(),
                        &asset_folder_path,
                        ResourceManager::default(),
                    );
                    skeleton_animations.insert(
                        loaded_skeleton_animation.url.clone(),
                        loaded_skeleton_animation.deref().clone(),
                    );

                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::Skeleton(asset) => {
                    let file_path = asset_folder_path.join(&asset.borrow().get_relative_path());
                    model_loader.load(&file_path).unwrap();
                    let loaded_skeleton = model_loader.to_runtime_skeleton(
                        asset.clone(),
                        &asset_folder_path,
                        ResourceManager::default(),
                    );
                    skeletons.insert(loaded_skeleton.url.clone(), loaded_skeleton.deref().clone());

                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::Texture(asset) => {
                    if let Some(image_file_path) = &asset.borrow().image_reference {
                        let absolute_image_file_path =
                            self.get_asset_folder_path().join(image_file_path.clone());
                        let file_stem =
                            image_file_path.file_stem().ok_or(anyhow!("No file stem"))?;
                        let name = file_stem.to_str().ok_or(anyhow!("Fail to convert str"))?;
                        let image_file_path = image_file_path
                            .to_str()
                            .ok_or(anyhow!("Fail to convert str"))?;
                        let url = rs_engine::build_asset_url(image_file_path)?;
                        let buffer = std::fs::read(absolute_image_file_path.clone()).context(
                            format!("Failed to read from {:?}", absolute_image_file_path),
                        )?;
                        let _ = image::load_from_memory(&buffer).context(format!(
                            "{:?} is not a valid image file.",
                            absolute_image_file_path
                        ))?;
                        let format = image::guess_format(&buffer)?;
                        let image = rs_artifact::image::Image {
                            name: name.to_string(),
                            url: url.clone(),
                            image_format: rs_artifact::image::ImageFormat::from_external_format(
                                format,
                            ),
                            data: buffer,
                        };
                        images.insert(url, image);
                    }
                    artifact_asset_encoder.encode(&*asset.borrow());
                }
                EContentFileType::Level(asset) => {
                    artifact_asset_encoder.encode(&*asset.borrow());
                }
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

        let _ = artifact_asset_encoder.finish()?;
        Ok(output_folder_path.join(output_filename))
    }

    fn collect_image_files(files: &[Rc<RefCell<TextureFile>>]) -> HashSet<PathBuf> {
        let mut image_paths = HashSet::new();
        for file in files {
            if let Some(image_reference) = &file.borrow().image_reference {
                let value = image_reference;
                image_paths.insert(value.clone());
            }
        }
        image_paths
    }

    pub fn pre_process_shaders() -> HashMap<String, String> {
        let mut shaders = HashMap::new();
        let buildin_shaders = rs_render::global_shaders::get_buildin_shaders();
        for buildin_shader in buildin_shaders {
            let description = buildin_shader.get_shader_description();
            let name = buildin_shader.get_name();
            if rs_core_minimal::misc::is_dev_mode() {
                let pre_process_code = rs_shader_compiler::pre_process::pre_process(
                    &description.shader_path,
                    description.include_dirs.iter(),
                    description.definitions.iter(),
                );
                match pre_process_code {
                    Ok(code) => {
                        if shaders.insert(name.clone(), code).is_some() {
                            panic!("{} is already exists", name);
                        }
                        continue;
                    }
                    Err(err) => {
                        log::trace!("{err}");
                    }
                }
            } else {
                let path = rs_render::get_buildin_shader_dir().join(name.clone());
                let code = std::fs::read_to_string(path.clone());
                match code {
                    Ok(code) => {
                        if shaders.insert(name.clone(), code).is_some() {
                            panic!("{} is already exists", name);
                        }
                    }
                    Err(err) => {
                        panic!("{}, {:?}", err, path);
                    }
                }
            }
        }
        shaders
    }
}
