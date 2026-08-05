use crate::{
    build_config::{BuildConfig, EArchType, EBuildPlatformType, EBuildType},
    content_edit::ContentEdit,
    project::{ASSET_FOLDER_NAME, CONTENT_FOLDER_NAME, Project},
};
use anyhow::{Context, anyhow};
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::{DebouncedEvent, Debouncer};
use rs_artifact::{
    EEndianType, artifact::ArtifactAssetEncoder, shader_source_code::ShaderSourceCode,
};
use rs_content_manager::content_manager::ContentManager;
use rs_engine::{ASSET_SCHEME, thread_pool::ThreadPool};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_hotreload_plugin::hot_reload::HotReload;
use rs_model_loader::model_loader::ModelLoader;
use rs_module::types::ModuleManager;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

pub enum EFolderUpdateType {
    Asset,
}

#[derive(Serialize, Deserialize)]
pub struct RecentProjects {
    pub paths: Vec<std::path::PathBuf>,
}

impl RecentProjects {
    pub fn load() -> RecentProjects {
        let path = Path::new("./recent_projects.json");
        if path.exists() {
            let file = std::fs::File::open(path).unwrap();
            let reader = std::io::BufReader::new(file);
            serde_json::from_reader(reader).unwrap()
        } else {
            RecentProjects { paths: Vec::new() }
        }
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        self.remove_duplicated();
        let path = Path::new("./recent_projects.json");
        std::fs::write(path, serde_json::to_string(self)?)?;
        Ok(())
    }

    pub fn remove_duplicated(&mut self) {
        self.paths.dedup();
    }
}

pub struct ProjectContext {
    pub project: Project,
    project_folder_path: PathBuf,
    project_file_path: PathBuf,
    _shader_folder_path: PathBuf,
    pub hot_reload: rs_hotreload_plugin::hot_reload::HotReload,
    folder_receiver:
        Option<std::sync::mpsc::Receiver<std::result::Result<Vec<DebouncedEvent>, notify::Error>>>,
    folder_debouncer: Option<Debouncer<ReadDirectoryChangesWatcher>>,
    pub content_manager: SingleThreadMutType<ContentManager>,
    pub module_manager: SingleThreadMutType<ModuleManager>,
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
        // #[cfg(debug_assertions)]
        // let lib_folder = project_folder_path.join("target").join("debug");
        // #[cfg(not(debug_assertions))]
        // let lib_folder = project_folder_path.join("target").join("release");
        let lib_folder = std::env::current_dir()?.join("deps");
        let hot_reload = HotReload::new(&lib_folder, &lib_folder, &project.project_name)?;
        let module_manager = SingleThreadMut::new(ModuleManager::new());
        let mut context = ProjectContext {
            project,
            project_file_path: project_file_path.to_path_buf(),
            project_folder_path: project_folder_path.to_path_buf(),
            hot_reload,
            _shader_folder_path: project_folder_path.join("shader"),
            folder_receiver: None,
            folder_debouncer: None,
            content_manager: SingleThreadMut::new(ContentManager::from_path(
                project_folder_path.join(CONTENT_FOLDER_NAME),
            )),
            module_manager,
        };
        context.watch_project_folder()?;
        Ok(context)
    }

    fn watch_project_folder(&mut self) -> anyhow::Result<()> {
        let (sender, receiver) = std::sync::mpsc::channel();

        let mut debouncer =
            notify_debouncer_mini::new_debouncer(std::time::Duration::from_millis(200), sender)
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
        let errors = self.content_manager.borrow().sync_disk();
        for (url, error) in errors {
            log::warn!("url: {}, error: {}", url, error);
        }
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
        let _ = std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn try_create_mesh_cluster_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.get_mesh_cluster_dir();
        let _ = std::fs::create_dir_all(&path)?;
        Ok(path)
    }

    pub fn get_mesh_cluster_dir(&self) -> PathBuf {
        self.get_build_dir().join("cache/mesh_cluster")
    }

    pub fn get_virtual_texture_cache_dir(&self) -> PathBuf {
        self.project_folder_path.join("build/cache/virtual_texture")
    }

    pub fn get_derive_data_dir(&self) -> PathBuf {
        self.project_folder_path.join("build/cache/derivedata")
    }

    pub fn try_create_derive_data_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.get_derive_data_dir();
        let _ = std::fs::create_dir_all(path.clone())?;
        Ok(path)
    }

    pub fn get_tmp_dir(&self) -> PathBuf {
        self.project_folder_path.join("build/tmp")
    }

    pub fn try_create_tmp_dir(&self) -> anyhow::Result<PathBuf> {
        let path = self.get_tmp_dir();
        let _ = std::fs::create_dir_all(path.clone())?;
        Ok(path)
    }

    pub fn get_ibl_bake_cache_dir(&self, sub_folder: &Path) -> PathBuf {
        Self::make_ibl_bake_cache_dir(&self.project_folder_path, sub_folder)
    }

    pub fn make_ibl_bake_cache_dir(project_folder_path: &Path, sub_folder: &Path) -> PathBuf {
        project_folder_path.join("build/cache/ibl").join(sub_folder)
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

    #[cfg(feature = "plugin_dotnet")]
    pub fn get_dotnet_script_shared_lib_path(&self) -> PathBuf {
        self.project_folder_path.join(format!(
            "dotnet/{}/bin/Debug/{}.dll",
            &self.project.project_name, &self.project.project_name
        ))
    }

    #[cfg(all(feature = "plugin_v8"))]
    pub fn get_js_script_entry_path(&self) -> PathBuf {
        self.project_folder_path.join(format!(
            "js/{}/{}.js",
            &self.project.project_name, &self.project.project_name
        ))
    }

    #[cfg(all(feature = "plugin_v8"))]
    pub fn get_js_script_root_dir(&self) -> PathBuf {
        self.project_folder_path.join(format!("js"))
    }

    pub fn get_content_folder_path(&self) -> PathBuf {
        self.project_folder_path.join(CONTENT_FOLDER_NAME)
    }

    pub fn try_create_content_folder_path(&self) -> anyhow::Result<PathBuf> {
        let path = self.get_content_folder_path();
        let _ = std::fs::create_dir_all(path.clone())?;
        Ok(path)
    }

    pub fn export(
        &mut self,
        model_loader: &mut ModelLoader,
        content_edit: &mut ContentEdit,
    ) -> anyhow::Result<PathBuf> {
        let _span = tracy_client::span!();

        let output_folder_path = self.try_create_build_dir()?;
        if !output_folder_path.exists() {
            std::fs::create_dir(output_folder_path.clone())?;
        }
        let output_filename = "main.rs";

        let mut artifact_asset_encoder = ArtifactAssetEncoder::new(
            Some(EEndianType::Little),
            self.project.settings.borrow().clone(),
            &output_folder_path.join(output_filename),
        );

        let mut shader_source_codes: HashMap<
            url::Url,
            rs_artifact::shader_source_code::ShaderSourceCode,
        > = HashMap::new();
        let mut associated_assets: HashMap<url::Url, Box<dyn rs_artifact::asset::Asset>> =
            HashMap::new();

        for content in self.content_manager.borrow().content_files() {
            let editable = content_edit.editable(content.borrow().as_ref());
            if let Some(editable) = editable {
                let _ = editable.export(
                    content.clone(),
                    &mut artifact_asset_encoder,
                    &mut associated_assets,
                    model_loader,
                    self,
                )?;
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
        for asset in shader_source_codes.values() {
            artifact_asset_encoder.encode(asset);
        }
        let _ = artifact_asset_encoder.finish()?;
        Ok(output_folder_path.join(output_filename))
    }

    pub fn load_shader_naga_modules() -> HashMap<String, naga::Module> {
        let _span = tracy_client::span!();
        let mut shaders = HashMap::new();
        let buildin_shaders = rs_render::global_shaders::get_buildin_shaders();
        let read_folder_path: PathBuf =
            rs_core_minimal::file_manager::get_engine_output_target_dir().join("shaders");

        for buildin_shader in buildin_shaders {
            let name = buildin_shader.get_name();
            let read_path = read_folder_path.join(format!("{}.nagamodule", &name));
            let file = match std::fs::File::open(&read_path) {
                Ok(file) => file,
                Err(err) => {
                    log::warn!(
                        "Failed to read naga module, consider rebuilding shader compiler program and recompiling shader again, {}. {}",
                        &read_path.to_string_lossy(),
                        err
                    );
                    continue;
                }
            };

            let mut buf_reader = std::io::BufReader::new(file);
            let module = match rs_artifact::bincode_legacy::deserialize_from::<
                std::io::BufReader<std::fs::File>,
                naga::Module,
            >(&mut buf_reader, None)
            {
                Ok(module) => module,
                Err(err) => {
                    log::warn!(
                        "Failed to read naga module, consider rebuilding shader compiler program and recompiling shader again, {}. {}",
                        &read_path.to_string_lossy(),
                        err
                    );
                    continue;
                }
            };
            shaders.insert(name, module);
        }

        shaders
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
                        let pre_process_code = rs_shader_compiler_core::pre_process::pre_process(
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
                        let path = Path::new("../shaders").join(name.clone());
                        let code = std::fs::read_to_string(path.clone());
                        let result = TaskResult {
                            name: name.clone(),
                            code: code.map_err(|err| anyhow!("{:?}, {}", path, err)),
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
