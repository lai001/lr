use crate::{asset::Asset, default_url, mesh_vertex::MeshVertex, resource_type::EResourceType};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaticMesh {
    pub name: String,
    // pub id: uuid::Uuid,
    pub url: url::Url,
    pub vertexes: Vec<MeshVertex>,
    pub indexes: Vec<u32>,
}

impl Asset for StaticMesh {
    fn get_url(&self) -> url::Url {
        self.url.clone()
    }

    fn get_resource_type(&self) -> EResourceType {
        EResourceType::StaticMesh
    }
}

impl Default for StaticMesh {
    fn default() -> Self {
        Self {
            name: Default::default(),
            // id: Default::default(),
            url: default_url().clone(),
            vertexes: Default::default(),
            indexes: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::StaticMesh;
    use crate::mesh_vertex::MeshVertex;

    #[test]
    fn test_case_static_mesh() {
        let mut mesh = StaticMesh::default();
        mesh.name = String::from("mesh");
        mesh.vertexes.push(MeshVertex::default());
        mesh.vertexes[0].position.x = 10.0;
        let encoded: Vec<u8> = bincode::serialize(&mesh).unwrap();
        let decoded: StaticMesh = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(decoded.name, "mesh");
        assert_eq!(decoded.vertexes.len(), 1);
        assert_eq!(decoded.vertexes[0].position.x, 10.0);
    }
}
