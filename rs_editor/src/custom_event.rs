#[derive(Debug, Clone)]
pub enum EFileDialogType {
    NewProject(String),
    OpenProject,
    IBL,
}

#[derive(Debug, Clone)]
pub enum ECustomEventType {
    OpenFileDialog(EFileDialogType),
}
