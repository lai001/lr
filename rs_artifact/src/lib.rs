// pub mod artifact;
pub mod file_header;
pub mod image;
pub mod mesh_vertex;
pub mod mesh_vertex_visitor;
pub mod static_mesh;
pub mod type_expected;

pub enum EEndianType {
    Big,
    Little,
    Native,
}
