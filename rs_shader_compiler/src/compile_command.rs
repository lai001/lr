use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompileCommand {
    pub arguments: Vec<String>,
    pub file: String,
}
