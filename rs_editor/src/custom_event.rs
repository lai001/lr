#[derive(Debug, Clone)]
pub enum EFileDialogType {
    NewProject(String),
    OpenProject,
}

#[derive(Debug, Clone)]
pub enum ECustomEventType {
    OpenFileDialog(EFileDialogType),
}
