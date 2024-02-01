use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Deserialize, Serialize)]
pub enum EResourceType {
    Image,
    StaticMesh,
    ShaderSourceCode,
    Level,
    Binary,
}