use serde::{Deserialize, Serialize};
use type_layout::TypeLayout;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, TypeLayout, Deserialize, Serialize)]
pub struct MeshVertex {
    pub vertex_color: glam::Vec4,
    pub position: glam::Vec3,
    pub normal: glam::Vec3,
    pub tangent: glam::Vec3,
    pub bitangent: glam::Vec3,
    pub tex_coord: glam::Vec2,
}

// impl Serialize for MeshVertex {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut state = serializer.serialize_struct(STRUCT_NAME, 6)?;
//         state.serialize_field(VERTEX_COLOR_FIELD, &self.vertex_color.to_array())?;
//         state.serialize_field(POSITION_FIELD, &self.position.to_array())?;
//         state.serialize_field(NORMAL_FIELD, &self.normal.to_array())?;
//         state.serialize_field(TANGENT_FIELD, &self.tangent.to_array())?;
//         state.serialize_field(BITANGENT_FIELD, &self.bitangent.to_array())?;
//         state.serialize_field(TEX_COORD_FIELD, &self.tex_coord.to_array())?;
//         state.end()
//     }
// }

// impl<'de> Deserialize<'de> for MeshVertex {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let mesh_vertex = deserializer.deserialize_struct(STRUCT_NAME, FIELDS, MeshVertexVisitor);
//         mesh_vertex
//     }
// }

#[cfg(test)]
mod test {
    use super::MeshVertex;

    #[test]
    fn test_case_mesh_vertex() {
        let mut vertex = MeshVertex::default();
        vertex.vertex_color = glam::vec4(10.0, 0.0, 0.0, 0.0);
        vertex.position = glam::vec3(20.0, 0.0, 0.0);
        vertex.normal = glam::vec3(30.0, 0.0, 0.0);
        vertex.bitangent = glam::vec3(40.0, 0.0, 0.0);
        vertex.tangent = glam::vec3(50.0, 0.0, 0.0);
        vertex.tex_coord = glam::vec2(60.0, 0.0);
        let encoded: Vec<u8> = bincode::serialize(&vertex).unwrap();
        let decoded: MeshVertex = bincode::deserialize(&encoded[..]).unwrap();
        assert_eq!(decoded.vertex_color, glam::vec4(10.0, 0.0, 0.0, 0.0));
        assert_eq!(decoded.position, glam::vec3(20.0, 0.0, 0.0));
        assert_eq!(decoded.normal, glam::vec3(30.0, 0.0, 0.0));
        assert_eq!(decoded.bitangent, glam::vec3(40.0, 0.0, 0.0));
        assert_eq!(decoded.tangent, glam::vec3(50.0, 0.0, 0.0));
        assert_eq!(decoded.tex_coord, glam::vec2(60.0, 0.0));
    }
}
