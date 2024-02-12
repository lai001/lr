#[derive(Debug, Clone)]
pub enum EFileDialogType {
    NewProject(String),
    OpenProject,
    ImportAsset,
    IBL,
}

#[derive(Debug, Clone)]
pub enum ECustomEventType {
    OpenFileDialog(EFileDialogType),
}
