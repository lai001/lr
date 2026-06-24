use crate::content_folder::ContentFolder;
use notify::ReadDirectoryChangesWatcher;
use notify_debouncer_full::{DebouncedEvent, Debouncer, FileIdMap};
use pathdiff::diff_paths;
use rs_core_minimal::path_ext::CanonicalizeSlashExt;
use rs_engine::{CONTENT_ROOT, CONTENT_SCHEME, content::content_file_type::EContentFileType};
use rs_foundation::new::SingleThreadMut;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
};
use walkdir::WalkDir;

pub const CONTENT_FILE_EXTENSION: &str = "content";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentMeta<T: Serialize> {
    #[serde(rename = "type")]
    pub ty: String,
    pub content: T,
}

type ContentCreator = Box<dyn Fn(Value) -> crate::error::Result<EContentFileType>>;
type ContentSaver = Box<dyn Fn(EContentFileType) -> crate::error::Result<Value>>;

pub struct ContentManager {
    content_root_folder_path: PathBuf,
    content_files: Vec<EContentFileType>,
    content_folders: HashMap<PathBuf, ContentFolder>,
    creators: HashMap<String, ContentCreator>,
    saver: HashMap<String, ContentSaver>,
    file_receiver: Option<Receiver<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>>,
    file_debouncer: Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>,
}

impl ContentManager {
    pub fn from_path(mut content_root_folder_path: PathBuf) -> Self {
        if let Ok(p) = content_root_folder_path.canonicalize_slash() {
            content_root_folder_path = p;
        }
        let mut creators = HashMap::new();
        let mut saver = HashMap::new();

        macro_rules! register_content_type {
            ($type_name:ident) => {{
                creators.insert(
                    stringify!($type_name).to_string(),
                    Box::new(|meta| {
                        Ok(EContentFileType::$type_name(SingleThreadMut::new(
                            serde_json::from_value(meta)?,
                        )))
                    }) as ContentCreator,
                );

                saver.insert(
                    stringify!($type_name).to_string(),
                    Box::new(|content| {
                        if let EContentFileType::$type_name(inner) = content {
                            Ok(serde_json::to_value(inner)?)
                        } else {
                            Err(crate::error::Error::Other(
                                "Content type mismatch".to_string(),
                            ))
                        }
                    }) as ContentSaver,
                );
            }};
        }

        register_content_type!(StaticMesh);
        register_content_type!(SkeletonMesh);
        register_content_type!(SkeletonAnimation);
        register_content_type!(Skeleton);
        register_content_type!(Texture);
        register_content_type!(Level);
        register_content_type!(Material);
        register_content_type!(IBL);
        register_content_type!(ParticleSystem);
        register_content_type!(Sound);
        register_content_type!(Curve);
        register_content_type!(BlendAnimations);
        register_content_type!(MaterialParamentersCollection);
        register_content_type!(RenderTarget2D);

        let mut manager = Self {
            content_root_folder_path,
            content_files: vec![],
            creators,
            saver: saver,
            file_receiver: None,
            file_debouncer: None,
            content_folders: HashMap::new(),
        };
        if let Err(err) = manager.watch_content_folder() {
            log::warn!("{err}");
        }
        if let Err(err) = manager.load() {
            log::warn!("{err}");
        }
        manager
    }

    pub fn new() -> Self {
        Self::from_path(PathBuf::new())
    }

    pub fn watch_content_folder(&mut self) -> crate::error::Result<()> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let mut debouncer = notify_debouncer_full::new_debouncer(Self::timeout(), None, sender)?;
        debouncer.watch(
            &std::path::Path::new(&self.content_root_folder_path),
            notify::RecursiveMode::Recursive,
        )?;
        self.file_receiver = Some(receiver);
        self.file_debouncer = Some(debouncer);
        log::trace!("Watch content folder. {:?}", self.content_root_folder_path);
        Ok(())
    }

    pub fn timeout() -> std::time::Duration {
        std::time::Duration::from_millis(100)
    }

    pub fn process_file_changed_notification(&mut self) {
        let mut all_events: Vec<DebouncedEvent> = vec![];

        if let Some(receiver) = self.file_receiver.as_ref() {
            for events in receiver.try_iter() {
                let Ok(mut events) = events else {
                    continue;
                };
                all_events.append(&mut events);
            }
        }

        for event in all_events {
            match event.kind {
                notify::EventKind::Create(_) => {
                    for path in &event.paths {
                        if path.is_dir() {
                            let relative_path =
                                Self::make_relative_path(path, &self.content_root_folder_path);
                            self.content_folders.entry(relative_path).or_default();
                            if let Some(parent) = path.parent() {
                                let relative_path = Self::make_relative_path(
                                    parent,
                                    &self.content_root_folder_path,
                                );
                                self.content_folders
                                    .entry(relative_path.clone())
                                    .or_default()
                                    .insert_sub_folder(relative_path);
                            }
                        } else {
                            todo!();
                        }
                    }
                }
                notify::EventKind::Remove(_) => {
                    self.on_delete_by_paths(&event.paths);
                }
                _ => {}
            }
        }
    }

    pub fn load(&mut self) -> crate::error::Result<()> {
        let _ = self.content_root_folder_path.try_exists()?;
        self.content_files.clear();
        self.content_folders.clear();
        for entry in WalkDir::new(&self.content_root_folder_path) {
            let entry = entry?;
            let path = entry.path();

            match Self::load_content(
                path,
                &self.content_root_folder_path,
                &self.creators,
                &mut self.content_folders,
            ) {
                Ok(content_file) => {
                    self.content_files.push(content_file);
                }
                Err(err) => log::warn!("{err}"),
            }
        }
        Ok(())
    }

    pub fn save(&self, content: EContentFileType) -> crate::error::Result<()> {
        let p = Self::try_create_path(&self.content_root_folder_path, &content.get_url())?;

        let type_text = content.get_type_text();
        let save = self
            .saver
            .get(&type_text)
            .ok_or(crate::error::Error::MissingValue(format!("{type_text}")))?;
        let content = save(content)?;
        let content = ContentMeta {
            ty: type_text,
            content,
        };
        let contents = serde_json::to_string_pretty(&content)?;
        Ok(std::fs::write(p, contents)?)
    }

    fn try_create_path(
        content_root_folder_path: &Path,
        url: &url::Url,
    ) -> crate::error::Result<PathBuf> {
        let p = Self::get_path(content_root_folder_path, url)
            .ok_or(crate::error::Error::MissingValue(format!("{url}")))?;
        if let Some(parent) = p.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        } else {
            return Err(std::io::Error::from(std::io::ErrorKind::NotADirectory))?;
        }
        Ok(p)
    }

    fn get_path(content_root_folder_path: &Path, url: &url::Url) -> Option<PathBuf> {
        let prefix = format!("{}://{}/", CONTENT_SCHEME, CONTENT_ROOT);
        let relative_path =
            Path::new(url.as_str().strip_prefix(&prefix)?).with_extension(CONTENT_FILE_EXTENSION);
        let abs_path = content_root_folder_path.join(relative_path);
        Some(abs_path)
    }

    fn load_content(
        path: &Path,
        content_root_folder_path: &Path,
        creators: &HashMap<String, ContentCreator>,
        content_folders: &mut HashMap<PathBuf, ContentFolder>,
    ) -> crate::error::Result<EContentFileType> {
        if path.is_file() {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let object: serde_json::Value = serde_json::from_reader(reader)?;
            let type_text = object.get("type").map(|x| x.as_str()).flatten().ok_or(
                crate::error::Error::MissingValue("Not a type text".to_string()),
            )?;
            let creator = creators
                .get(type_text)
                .ok_or(crate::error::Error::MissingValue(format!(
                    "No createor for {}",
                    type_text
                )))?;
            let content = object
                .get("content")
                .ok_or(crate::error::Error::MissingValue(
                    "No content field".to_string(),
                ))?;
            let content = creator(content.clone())?;
            if let Some(parent) = path.parent() {
                let relative_path = Self::make_relative_path(parent, content_root_folder_path);

                content_folders
                    .entry(relative_path.clone())
                    .or_insert_with(|| ContentFolder::new(relative_path))
                    .insert_file(content.clone());
            }
            return Ok(content);
        } else if path.is_dir() {
            let relative_path = Self::make_relative_path(path, content_root_folder_path);

            content_folders
                .entry(relative_path.clone())
                .or_insert_with(|| ContentFolder::new(relative_path.clone()));
            let parent = relative_path.parent();
            if let Some(parent) = parent {
                if let Some(content_folder) = content_folders.get_mut(parent) {
                    content_folder.insert_sub_folder(relative_path);
                }
            }
        }
        Err(crate::error::Error::Other("Unknown error".to_string()))
    }

    pub fn content_root_relative_path() -> &'static Path {
        Path::new(".")
    }

    fn make_relative_path(path: &Path, root_path: &Path) -> PathBuf {
        assert!(path.is_absolute());
        let relative_path = diff_paths(path, root_path).expect("Valid path");
        let relative_path = Self::content_root_relative_path().join(relative_path);
        relative_path
    }

    pub fn sync_disk(&self) -> HashMap<url::Url, crate::error::Error> {
        let mut errors = HashMap::new();
        for content in self.content_files.clone() {
            match self.save(content.clone()) {
                Ok(_) => {}
                Err(err) => {
                    errors.insert(content.get_url(), err);
                }
            }
        }
        errors
    }

    pub fn set_content_root_folder_path(&mut self, content_root_folder_path: PathBuf) {
        if content_root_folder_path == self.content_root_folder_path {
            return;
        }
        self.content_root_folder_path = content_root_folder_path;
        self.content_files.clear();
        self.content_folders.clear();
        self.file_receiver = None;
        self.file_debouncer = None;
    }

    pub fn content_files_map(&self) -> HashMap<url::Url, EContentFileType> {
        let mut map = HashMap::new();
        for content in self.content_files.clone() {
            map.insert(content.get_url(), content);
        }
        map
    }

    pub fn content_files(&self) -> &[EContentFileType] {
        &self.content_files
    }

    pub fn content_folders(&self) -> &HashMap<PathBuf, ContentFolder> {
        &self.content_folders
    }

    pub fn root_content_folder(&self) -> Option<&ContentFolder> {
        let root_folder = self.content_folders.get(Self::content_root_relative_path());
        root_folder
    }

    pub fn append(
        &mut self,
        new_files: Vec<EContentFileType>,
    ) -> HashMap<url::Url, crate::error::Error> {
        let mut errors = HashMap::new();
        for new_file in new_files {
            let url = new_file.get_url();
            let urls = self
                .content_files
                .iter()
                .map(|x| x.get_url())
                .collect::<Vec<url::Url>>();
            if !urls.contains(&url)
                && let Some(path) = Self::get_path(&self.content_root_folder_path, &url)
                && let Some(parent) = path.parent()
                && let relative_path = Self::make_relative_path( parent,  &self.content_root_folder_path)
                && let Some(folder) = self.content_folders.get_mut(&relative_path)
            {
                folder.insert_file(new_file.clone());
                self.content_files.push(new_file);
            } else {
                errors.insert(url, crate::error::Error::Other(format!("")));
            }
        }
        errors
    }

    pub fn delete_contents(&mut self, contents: Vec<EContentFileType>) {
        let content_root_folder_path = self.content_root_folder_path.clone();
        let paths = contents
            .iter()
            .flat_map(|x| {
                let path = Self::get_path(&content_root_folder_path, &x.get_url());
                path
            })
            .collect::<Vec<PathBuf>>();
        self.on_delete_by_paths(&paths);
        for path in paths {
            if let Err(err) = std::fs::remove_file(path) {
                log::warn!("{err}");
            }
        }
    }

    pub fn on_delete_by_paths(&mut self, paths: &[PathBuf]) {
        let content_root_folder_path = self.content_root_folder_path.clone();
        self.content_files.retain(|x| {
            let Some(path) = Self::get_path(&content_root_folder_path, &x.get_url()) else {
                return true;
            };
            let is_remove = paths.contains(&path);
            !is_remove
        });
        self.content_folders.retain(|k, _| {
            let abs_path = content_root_folder_path.join(k);
            abs_path.exists()
        });

        for content_folder in self.content_folders.values_mut() {
            content_folder.on_folders_removed(&content_root_folder_path);
        }
    }

    pub fn create_sub_folder(
        &mut self,
        target_folder: &mut ContentFolder,
        new_folder_name: &str,
    ) -> crate::error::Result<PathBuf> {
        if let Some(folder) = self.content_folders.get_mut(target_folder.relative_path()) {
            let sub_folder_path =
                folder.create_sub_folder(&self.content_root_folder_path, new_folder_name)?;
            target_folder.insert_sub_folder(sub_folder_path.clone());
            self.content_folders.insert(
                sub_folder_path.clone(),
                ContentFolder::new(sub_folder_path.clone()),
            );
            return Ok(sub_folder_path);
        }
        return Err(std::io::Error::from(std::io::ErrorKind::NotADirectory))?;
    }
}

#[cfg(test)]
mod test {
    use crate::content_manager::{CONTENT_FILE_EXTENSION, ContentManager, ContentMeta};
    use rs_engine::{build_content_file_url, content::static_mesh::AssetInfo};
    use std::path::PathBuf;

    fn mock_contents() {
        let path = rs_core_minimal::file_manager::get_engine_build_tmp_dir()
            .join("content_manager")
            .join("content");
        let _ = std::fs::remove_dir_all(&path);
        let _ = std::fs::create_dir_all(&path);
        let mock_mesh = rs_engine::content::static_mesh::StaticMesh {
            url: build_content_file_url("Untitled1").unwrap(),
            asset_info: AssetInfo {
                relative_path: PathBuf::new(),
                path: "".to_string(),
            },
            is_enable_multiresolution: false,
        };
        let content_meta = ContentMeta {
            ty: "StaticMesh".to_string(),
            content: mock_mesh,
        };
        let data = serde_json::to_string_pretty(&content_meta).unwrap();
        let _ = std::fs::write(
            &path
                .join("Untitled1")
                .with_added_extension(CONTENT_FILE_EXTENSION),
            data,
        );

        let path = path.join("SubFoler");
        let _ = std::fs::create_dir_all(&path);
        let mock_mesh = rs_engine::content::static_mesh::StaticMesh {
            url: build_content_file_url("SubFoler/Untitled2").unwrap(),
            asset_info: AssetInfo {
                relative_path: PathBuf::new(),
                path: "".to_string(),
            },
            is_enable_multiresolution: false,
        };
        let content_meta = ContentMeta {
            ty: "StaticMesh".to_string(),
            content: mock_mesh,
        };
        let data = serde_json::to_string_pretty(&content_meta).unwrap();
        let _ = std::fs::write(
            &path
                .join("Untitled2")
                .with_added_extension(CONTENT_FILE_EXTENSION),
            data,
        );

        let path = path.join("SubFoler2");
        let _ = std::fs::create_dir_all(&path);
        let mock_mesh = rs_engine::content::static_mesh::StaticMesh {
            url: build_content_file_url("SubFoler/SubFoler2/Untitled3").unwrap(),
            asset_info: AssetInfo {
                relative_path: PathBuf::new(),
                path: "".to_string(),
            },
            is_enable_multiresolution: false,
        };
        let content_meta = ContentMeta {
            ty: "StaticMesh".to_string(),
            content: mock_mesh,
        };
        let data = serde_json::to_string_pretty(&content_meta).unwrap();
        let _ = std::fs::write(
            &path
                .join("Untitled3")
                .with_added_extension(CONTENT_FILE_EXTENSION),
            data,
        );
    }

    #[test]
    fn test_case() {
        mock_contents();
        let path = rs_core_minimal::file_manager::get_engine_build_tmp_dir()
            .join("content_manager")
            .join("content");
        let mut content_manager = ContentManager::from_path(path.clone());

        match content_manager.content_files[0].clone() {
            rs_engine::content::content_file_type::EContentFileType::StaticMesh(mesh) => {
                mesh.borrow_mut().is_enable_multiresolution = true;
            }
            _ => {
                panic!();
            }
        }
        assert_eq!(content_manager.content_files[0].get_name(), "Untitled3");
        content_manager
            .save(content_manager.content_files[0].clone())
            .unwrap();

        std::fs::remove_dir_all(path.join("SubFoler/SubFoler2")).unwrap();
        std::thread::sleep(2 * ContentManager::timeout());

        assert_eq!(content_manager.content_folders.len(), 3);
        content_manager.process_file_changed_notification();
        assert_eq!(content_manager.content_folders.len(), 2);

        std::fs::create_dir_all(path.join("SubFoler/SubFoler2/SubFoler3")).unwrap();
        std::thread::sleep(2 * ContentManager::timeout());
        content_manager.process_file_changed_notification();
        assert_eq!(content_manager.content_folders.len(), 4);
    }
}
