use crate::mesh_vertex::MeshVertex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct StaticMesh {
    pub name: String,
    pub id: uuid::Uuid,
    pub vertexes: Vec<MeshVertex>,
    pub indexes: Vec<u32>,
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
