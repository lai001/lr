use rs_engine::{build_content_file_url, content::content_file_type::EContentFileType};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ContentFolder {
    relative_path: PathBuf,
    parent_folder: Option<PathBuf>,
    folders: Vec<PathBuf>,
    files: Vec<EContentFileType>,
}

impl ContentFolder {
    pub fn new(relative_path: PathBuf) -> Self {
        assert!(relative_path.is_relative());

        let mut parent_folder: Option<PathBuf> = None;
        if let Some(p) = relative_path.parent() {
            if let Some(first) = p.components().next() {
                if first.as_os_str() != ".." {
                    parent_folder = Some(p.to_path_buf());
                }
            }
        }
        Self {
            relative_path,
            files: vec![],
            folders: vec![],
            parent_folder,
        }
    }

    pub fn get_url(&self) -> url::Url {
        let components = self.relative_path.components();
        let mut parts = Vec::new();
        for component in components {
            if component.as_os_str() == "." {
            } else {
                parts.push(component.as_os_str().to_str().unwrap().to_string());
            }
        }
        let path = parts.join("/");
        build_content_file_url(path).expect("Valid url")
    }

    pub fn parent_folder(&self) -> Option<&PathBuf> {
        self.parent_folder.as_ref()
    }

    pub fn relative_path(&self) -> &PathBuf {
        &self.relative_path
    }

    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }

    pub fn files(&self) -> &[EContentFileType] {
        &self.files
    }

    pub(crate) fn insert_file(&mut self, file: EContentFileType) {
        let mut base_url = self.get_url();
        base_url.set_path(&format!("{}/{}", self.get_url().path(), file.get_name()));
        assert_eq!(
            base_url,
            file.get_url(),
            "url path: {}, relative_path: {}",
            self.get_url().path(),
            self.relative_path.display()
        );
        assert!(
            self.files
                .iter()
                .map(|x| x.get_name())
                .all(|x| x != file.get_name()),
        );
        self.files.push(file);
    }

    pub(crate) fn insert_sub_folder(&mut self, folder: PathBuf) {
        assert!(folder.is_relative());
        if !self.folders.contains(&folder) {
            self.folders.push(folder);
        }
    }

    pub(crate) fn on_folders_removed(&mut self, content_root_folder_path: &Path) {
        self.folders.retain(|x| {
            let abs_path = content_root_folder_path.join(x);
            abs_path.exists()
        });
    }

    pub(crate) fn create_sub_folder(
        &mut self,
        content_root_folder_path: &Path,
        folder_name: &str,
    ) -> crate::error::Result<PathBuf> {
        let suffix = self.relative_path.join(folder_name);
        let abs_path = content_root_folder_path.join(&suffix);
        if !abs_path.exists() {
            std::fs::create_dir_all(abs_path)?;
        }
        self.insert_sub_folder(suffix.clone());
        Ok(suffix)
    }
}

impl Default for ContentFolder {
    fn default() -> ContentFolder {
        ContentFolder::new(Path::new(".").to_path_buf())
    }
}

#[cfg(test)]
mod test {
    use crate::content_folder::ContentFolder;
    use std::path::Path;

    #[test]
    fn test_case() {
        let content_folder = ContentFolder::default();
        let expected = url::Url::parse("content://Content").unwrap();
        assert_eq!(content_folder.get_url(), expected);

        let content_folder = ContentFolder::new(Path::new("./SubFoler").to_path_buf());
        let expected = url::Url::parse("content://Content/SubFoler").unwrap();
        assert_eq!(content_folder.get_url(), expected);

        let content_folder = ContentFolder::new(Path::new("./SubFoler/SubFoler2").to_path_buf());
        let expected = url::Url::parse("content://Content/SubFoler/SubFoler2").unwrap();
        assert_eq!(content_folder.get_url(), expected);
    }
}
