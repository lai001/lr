#[derive(Debug)]
pub struct PrimitiveData {
    pub vertex_colors: Vec<glam::Vec4>,
    pub vertex_positions: Vec<glam::Vec3>,
    pub vertex_normals: Vec<glam::Vec3>,
    pub vertex_tangents: Vec<glam::Vec3>,
    pub vertex_bitangents: Vec<glam::Vec3>,
    pub vertex_tex_coords: Vec<glam::Vec2>,
    pub indices: Vec<u32>,
}

impl PrimitiveData {
    pub fn quad() -> PrimitiveData {
        let top_left = glam::vec3(-1.0, 0.0, -1.0);
        let top_right = glam::vec3(1.0, 0.0, -1.0);
        let bottom_left = glam::vec3(-1.0, 0.0, 1.0);
        let bottom_right = glam::vec3(1.0, 0.0, 1.0);

        let top_left_coord = glam::vec2(0.0, 0.0);
        let top_right_coord = glam::vec2(1.0, 0.0);
        let bottom_left_coord = glam::vec2(0.0, 1.0);
        let bottom_right_coord = glam::vec2(1.0, 1.0);

        PrimitiveData {
            vertex_colors: vec![glam::vec4(0.0, 0.0, 0.0, 1.0); 4],
            vertex_positions: vec![top_left, top_right, bottom_right, bottom_left],
            vertex_normals: vec![glam::vec3(0.5, 0.5, 1.0,); 4],
            vertex_tangents: vec![glam::Vec3::X; 4],
            vertex_bitangents: vec![glam::Vec3::Y; 4],
            vertex_tex_coords: vec![
                top_left_coord,
                top_right_coord,
                bottom_right_coord,
                bottom_left_coord,
            ],
            indices: vec![0, 1, 3, 1, 2, 3],
        }
    }
}
