use crate::{image::Image, static_mesh::StaticMesh};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Artifact {
    version: String,
    meshes: Vec<StaticMesh>,
    images: Vec<Image>,
}

#[cfg(test)]
mod test {
    use super::Artifact;

    #[test]
    fn test_case_artifact() {
        let mut artifact = Artifact::default();
        let encoded: Vec<u8> = bincode::serialize(&artifact).unwrap();
        let decoded: Artifact = bincode::deserialize(&encoded[..]).unwrap();
    }
}
