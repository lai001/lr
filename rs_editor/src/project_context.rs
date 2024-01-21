use crate::{
    error::Result,
    model_loader::{MeshCluster, ModelLoader},
    project::{Project, ASSET_FOLDER_NAME},
};
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_mini::{DebouncedEvent, Debouncer};
use rs_artifact::{static_mesh::StaticMesh, EEndianType};
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
    pub fn open(project_file_path: &Path) -> Result<ProjectContext> {
        let project_folder_path = match project_file_path.parent() {
            Some(project_folder_path) => project_folder_path,
            None => {
                return Err(crate::error::Error::OpenProjectFailed(Some(
                    "Can not find parent folder.".to_string(),
                )));
            }
        };
        let file = match std::fs::File::open(project_file_path) {
            Ok(file) => file,
            Err(err) => {
                return Err(crate::error::Error::IO(err, None));
            }
        };
        let reader = std::io::BufReader::new(file);
        let project: Project = match serde_json::de::from_reader(reader) {
            Ok(project) => project,
            Err(err) => {
                return Err(crate::error::Error::OpenProjectFailed(Some(
                    err.to_string(),
                )));
            }
        };
        let lib_folder = project_folder_path.join("target").join("debug");
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

    pub fn reload_if_need(&mut self) -> bool {
        let result = self.hot_reload.reload_if_need();
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

    pub fn export(&mut self) -> Result<()> {
        let output_folder_path = self.project_folder_path.join("artifact");
        let _ = std::fs::create_dir(output_folder_path.clone());
        let output_filename = self.project.project_name.clone() + ".rs";

        let mut referenced_meshs: HashMap<PathBuf, HashSet<String>> = HashMap::new();
        let mut static_meshs: Vec<StaticMesh> = Vec::new();

        for node in &self.project.level.nodes {
            self.walk_node(node, &mut |child_node| {
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
                        url: url::Url::parse(&format!(
                            "asset://{}/{}",
                            file_path.to_str().unwrap(),
                            mesh_name
                        ))
                        .unwrap(),
                        vertexes: mesh_cluster.vertex_buffer.clone(),
                        indexes: mesh_cluster.index_buffer.clone(),
                    };
                    static_meshs.push(static_mesh);
                }
            }
        }

        let result = rs_artifact::artifact::encode_artifact_assets_disk(
            &static_meshs,
            Some(EEndianType::Little),
            &output_folder_path.join(output_filename),
        );

        if result {
            Ok(())
        } else {
            Err(crate::error::Error::ExportFailed(None))
        }
    }

    pub fn walk_node<T>(&self, node: &crate::level::Node, walk: &mut T)
    where
        T: FnMut(&crate::level::Node),
    {
        walk(node);
        for node in node.childs.iter() {
            self.walk_node(node, walk);
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

    fn node_to_node(node: &crate::level::Node) -> crate::data_source::Node {
        let mut childs: Vec<crate::data_source::Node> = Vec::new();
        for child_node in &node.childs {
            childs.push(Self::node_to_node(child_node));
        }
        crate::data_source::Node {
            name: node.name.clone(),
            childs,
            id: uuid::Uuid::new_v4(),
        }
    }

    pub fn build_ui_level(&self) -> Option<crate::data_source::Level> {
        let level = &self.project.level;
        let mut nodes: Vec<crate::data_source::Node> = Vec::new();
        for node in &level.nodes {
            nodes.push(Self::node_to_node(&node));
        }
        return Some(crate::data_source::Level {
            name: level.name.clone(),
            nodes,
        });
    }
}
