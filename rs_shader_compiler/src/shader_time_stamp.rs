use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Serialize, Deserialize)]
pub struct ShaderTimeStamp {
    pub last_modified_time: SystemTime,
    pub file_path: PathBuf,
}

impl ShaderTimeStamp {
    pub fn from_shader_file(file_path: PathBuf) -> anyhow::Result<ShaderTimeStamp> {
        let metadata = file_path.metadata()?;
        let modified_time = metadata.modified()?;
        Ok(ShaderTimeStamp {
            last_modified_time: modified_time,
            file_path,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ShaderTimeStampFile {
    pub shader_time_stamps: HashMap<PathBuf, ShaderTimeStamp>,
}

impl ShaderTimeStampFile {
    pub fn read_from_disk() -> anyhow::Result<ShaderTimeStampFile> {
        let file_path = rs_core_minimal::file_manager::get_engine_output_target_dir()
            .join("shaders/time_stamp");
        let file = std::fs::File::open(file_path)?;
        let reader = std::io::BufReader::new(file);
        Ok(serde_json::from_reader(reader)?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let file_path = rs_core_minimal::file_manager::get_engine_output_target_dir()
            .join("shaders/time_stamp");
        let contents = serde_json::to_string(self)?;
        std::fs::write(&file_path, contents)?;
        Ok(())
    }

    pub fn is_shader_outdated(&self, file_path: &Path) -> bool {
        if !file_path.exists() {
            return true;
        }
        let Some(shader_time_stamp) = self.shader_time_stamps.get(file_path) else {
            return true;
        };
        let Ok(metadata) = file_path.metadata() else {
            return true;
        };
        let Ok(modified_time) = metadata.modified() else {
            return true;
        };
        modified_time != shader_time_stamp.last_modified_time
    }
}
