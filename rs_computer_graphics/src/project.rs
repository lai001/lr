use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectDescriptionUserScriptField {
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectDescriptionDotnetField {
    pub config_path: String,
    pub assembly_path: String,
    pub type_name: String,
    pub method_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectDescriptionFileField {
    pub resource_dir: String,
    pub shader_dir: String,
    pub intermediate_dir: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectDescription {
    paths: ProjectDescriptionFileField,
    dotnet: ProjectDescriptionDotnetField,
    user_script: ProjectDescriptionUserScriptField,
}

impl ProjectDescription {
    fn new(project_json_path: String) -> ProjectDescription {
        let project_json_path = Path::new(&project_json_path);
        let mut file = File::open(project_json_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let project_description: ProjectDescription = serde_json::from_str(&contents).unwrap();
        log::trace!("{:?}", project_description);
        if !std::path::Path::new(&project_description.paths.intermediate_dir).exists() {
            if let Ok(_) = std::fs::create_dir(project_description.paths.intermediate_dir.clone()) {
                log::trace!(
                    "create_dir: {}",
                    &project_description.paths.intermediate_dir
                );
            }
        }
        project_description
    }

    pub fn default() -> Arc<Mutex<ProjectDescription>> {
        GLOBAL_PROJECT_DESCRIPTION.clone()
    }

    pub fn get_paths(&self) -> &ProjectDescriptionFileField {
        &self.paths
    }

    pub fn get_dotnet(&self) -> &ProjectDescriptionDotnetField {
        &self.dotnet
    }

    pub fn get_user_script(&self) -> &ProjectDescriptionUserScriptField {
        &self.user_script
    }
}

lazy_static! {
    static ref GLOBAL_PROJECT_DESCRIPTION: Arc<Mutex<ProjectDescription>> = Arc::new(Mutex::new(
        ProjectDescription::new("./Project.json".to_string())
    ));
}
