#[derive(Debug, Clone)]
pub enum EFileDialogType {
    NewProject(String),
    OpenProject,
    ImportAsset,
}

#[derive(Debug, Clone)]
pub enum ECustomEventType {
    OpenFileDialog(EFileDialogType),
}
