use crate::{
    build_config::{BuildConfig, EArchType, EBuildPlatformType, EBuildType},
    model_loader::{MeshCluster, ModelLoader},
    project::{Project, ASSET_FOLDER_NAME},
};
use anyhow::Context;
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::{DebouncedEvent, Debouncer};
use rs_artifact::{
    artifact::ArtifactAssetEncoder, property_value_type::EPropertyValueType,
    shader_source_code::ShaderSourceCode, static_mesh::StaticMesh, EEndianType,
};
use rs_hotreload_plugin::hot_reload::HotReload;
use std::{
    collections::{HashMap, HashSet},
    io::Write,
    path::{Path, PathBuf},
};

pub enum EFolderUpdateType {
    Asset,
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
        let hot_reload = HotReload::new(&project_folder_path, &lib_folder, &project.project_name);
        let mut context = ProjectContext {
            project,
            project_file_path: project_file_path.to_path_buf(),
            project_folder_path: project_folder_path.to_path_buf(),
            hot_reload,
            shader_folder_path: project_folder_path.join("shader"),
            folder_receiver: None,
            folder_debouncer: None,
        };
        context.watch_project_folder();
        Ok(context)
    }

    fn watch_project_folder(&mut self) {
        let (sender, receiver) = std::sync::mpsc::channel();

        let mut debouncer = notify_debouncer_mini::new_debouncer(
            std::time::Duration::from_millis(200),
            None,
            sender,
        )
        .unwrap();
        let watch_folder_path = self.get_project_folder_path();

        let _ = debouncer.watcher().watch(
            &std::path::Path::new(&watch_folder_path),
            notify::RecursiveMode::Recursive,
        );
        self.folder_receiver = Some(receiver);
        self.folder_debouncer = Some(debouncer);
        log::trace!("Watch project folder. {:?}", watch_folder_path);
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
            match events {
                Ok(events) => {
                    for event in events {
                        if event.path.starts_with(asset_folder_path.clone()) {
                            is_need_update = true;
                            break;
                        }
                    }
                }
                Err(errors) => {}
            }
        }

        if is_need_update {
            return Some(EFolderUpdateType::Asset);
        }
        return None;
    }

    pub fn is_need_reload_plugin(&self) -> bool {
        let result = self.hot_reload.is_need_reload();
        return result;
    }

    pub fn reload(&mut self) -> bool {
        let result = self.hot_reload.reload();
        return result;
    }

    pub fn get_asset_folder_path(&self) -> PathBuf {
        return self.project_folder_path.join(ASSET_FOLDER_NAME);
    }

    pub fn copy_file_to_asset_folder(&self, path: &Path) -> bool {
        let to = self.get_asset_folder_path().join(path.file_name().unwrap());
        match std::fs::copy(path, to.clone()) {
            Ok(_) => true,
            Err(err) => {
                log::warn!("{} {:?}", err, to);
                return false;
            }
        }
    }

    pub fn save(&self) -> bool {
        let json_str = match serde_json::ser::to_string_pretty(&self.project) {
            Ok(json_str) => json_str,
            Err(_) => {
                return false;
            }
        };
        let Ok(mut file) = std::fs::File::create(self.project_file_path.clone()) else {
            return false;
        };
        match file.write_fmt(format_args!("{}", json_str)) {
            Ok(_) => return true,
            Err(_) => return false,
        }
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

    fn node_to_artifact_node(node: &crate::level::Node) -> rs_artifact::level::ENodeType {
        let mesh_reference = &node.mesh_reference.as_ref().unwrap();
        let url = Self::build_static_mesh_url(
            &mesh_reference.file_path,
            &mesh_reference.referenced_mesh_name,
        );
        let mut childs: Vec<rs_artifact::level::ENodeType> = vec![];
        for x in node.childs.iter() {
            childs.push(Self::node_to_artifact_node(&x.borrow()));
        }
        let mut values: HashMap<String, EPropertyValueType> = HashMap::new();
        for (key, value) in &node.values {
            values.insert(key.clone(), value.clone());
        }
        let node3d = rs_artifact::level::Node3D {
            name: node.name.clone(),
            id: uuid::Uuid::new_v4(),
            url: None,
            mesh_url: Some(url),
            childs,
            values,
        };
        rs_artifact::level::ENodeType::Node3D(node3d)
    }

    fn level_to_level(level: &crate::level::Level) -> rs_artifact::level::Level {
        let mut nodes: Vec<rs_artifact::level::ENodeType> = vec![];
        for x in level.nodes.iter() {
            nodes.push(Self::node_to_artifact_node(&x.borrow()));
        }
        return rs_artifact::level::Level {
            name: level.name.clone(),
            id: uuid::Uuid::new_v4(),
            url: url::Url::parse(&format!("asset://level/{}", level.name)).unwrap(),
            nodes,
        };
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

    pub fn export(&mut self) -> anyhow::Result<PathBuf> {
        let output_folder_path = self.try_create_build_dir()?;
        let _ = std::fs::create_dir(output_folder_path.clone());
        // let output_filename = self.project.project_name.clone() + ".rs";
        let output_filename = "main.rs";

        let mut referenced_meshs: HashMap<PathBuf, HashSet<String>> = HashMap::new();
        let mut static_meshs: Vec<StaticMesh> = Vec::new();
        let mut images: Vec<rs_artifact::image::Image> = Vec::new();
        let mut artifact_asset_encoder = ArtifactAssetEncoder::new(
            Some(EEndianType::Little),
            &output_folder_path.join(output_filename),
        );
        for node in &self.project.level.borrow().nodes {
            self.walk_node(&node.borrow(), &mut |child_node| {
                Self::collect_resource(&mut referenced_meshs, child_node);
            });
        }

        for (file_path, mesh_names) in referenced_meshs.iter() {
            let mut mesh_clusters_map: HashMap<&String, &MeshCluster> = HashMap::new();
            let mesh_clusters =
                ModelLoader::load_from_file(&self.get_asset_folder_path().join(file_path), &[]);
            if let Some(mesh_clusters) = &mesh_clusters {
                for mesh_cluster in mesh_clusters {
                    mesh_clusters_map.insert(&mesh_cluster.name, mesh_cluster);
                }
            }

            for mesh_name in mesh_names {
                if let Some(mesh_cluster) = mesh_clusters_map.get(mesh_name) {
                    let static_mesh = StaticMesh {
                        name: mesh_cluster.name.clone(),
                        id: uuid::Uuid::new_v4(),
                        url: Self::build_static_mesh_url(file_path, mesh_name),
                        vertexes: mesh_cluster.vertex_buffer.clone(),
                        indexes: mesh_cluster.index_buffer.clone(),
                    };
                    static_meshs.push(static_mesh);
                }
            }
        }

        let mut texture_files: Vec<&crate::texture::TextureFile> = Vec::new();
        Self::collect_texture_files(&self.project.texture_folder, &mut texture_files);
        let image_files = Self::collect_image_files(&texture_files);
        for image_file_path in image_files {
            let absolute_image_file_path =
                self.get_asset_folder_path().join(image_file_path.clone());
            let file_stem = image_file_path.file_stem().unwrap();
            let name = file_stem.to_str().unwrap().to_string();
            let url = rs_engine::build_asset_url(image_file_path.to_str().unwrap()).unwrap();
            let buffer = std::fs::read(absolute_image_file_path.clone()).context(format!(
                "Failed to read from {:?}",
                absolute_image_file_path
            ))?;
            let _ = image::load_from_memory(&buffer).context(format!(
                "{:?} is not a valid image file.",
                absolute_image_file_path
            ))?;
            let format = image::guess_format(&buffer)?;
            let image = rs_artifact::image::Image {
                name,
                url,
                image_format: rs_artifact::image::ImageFormat::from_external_format(format),
                data: buffer,
            };
            images.push(image);
        }

        // FIXME: Out of memory
        #[cfg(feature = "editor")]
        for (name, code) in Self::pre_process_shaders() {
            let shader_source_code = ShaderSourceCode {
                name: name.clone(),
                id: uuid::Uuid::new_v4(),
                url: Self::build_shader_url(&name),
                code,
            };
            artifact_asset_encoder.encode(&shader_source_code);
        }
        artifact_asset_encoder.encode(&Self::level_to_level(&self.project.level.borrow()));
        for static_mesh in static_meshs.iter() {
            artifact_asset_encoder.encode(static_mesh);
        }
        for image in images.iter() {
            artifact_asset_encoder.encode(image);
        }
        let result = artifact_asset_encoder.finish();
        if result {
            Ok(output_folder_path.join(output_filename))
        } else {
            Err(crate::error::Error::ExportFailed(None).into())
        }
    }

    fn collect_image_files(files: &[&crate::texture::TextureFile]) -> HashSet<PathBuf> {
        let mut image_paths = HashSet::new();
        for file in files {
            if let Some(image_reference) = &file.image_reference {
                let value = image_reference;
                image_paths.insert(value.clone());
            }
        }
        image_paths
    }

    fn collect_texture_files<'a>(
        texture_folder: &'a crate::texture::TextureFolder,
        files: &mut Vec<&'a crate::texture::TextureFile>,
    ) {
        for texture_file in &texture_folder.texture_files {
            files.push(texture_file);
        }
        for sub_folder in &texture_folder.texture_folders {
            Self::collect_texture_files(sub_folder, files);
        }
    }

    pub fn walk_node<T>(&self, node: &crate::level::Node, walk: &mut T)
    where
        T: FnMut(&crate::level::Node),
    {
        walk(node);
        for node in node.childs.iter() {
            self.walk_node(&node.borrow(), walk);
        }
    }

    pub fn collect_resource(
        referenced_meshs: &mut HashMap<PathBuf, HashSet<String>>,
        node: &crate::level::Node,
    ) {
        if let Some(mesh_reference) = &node.mesh_reference {
            if let Some(names) = referenced_meshs.get_mut(&mesh_reference.file_path) {
                names.insert(mesh_reference.referenced_mesh_name.clone());
            } else {
                referenced_meshs.insert(
                    mesh_reference.file_path.clone(),
                    HashSet::from([mesh_reference.referenced_mesh_name.clone()]),
                );
            }
        }
    }

    #[cfg(feature = "editor")]
    pub fn pre_process_shaders() -> HashMap<String, String> {
        let mut shaders = HashMap::new();
        let buildin_shaders = rs_render::global_shaders::get_buildin_shaders();
        for buildin_shader in buildin_shaders {
            let description = buildin_shader.get_shader_description();
            let name = buildin_shader.get_name();
            let processed_code = rs_shader_compiler::pre_process::pre_process(
                &description.shader_path,
                description.include_dirs.iter(),
                description.definitions.iter(),
            )
            .unwrap();
            shaders.insert(name, processed_code);
        }
        shaders
    }
}
