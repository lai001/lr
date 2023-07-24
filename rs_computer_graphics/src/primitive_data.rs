use crate::brigde_data::mesh_vertex::MeshVertex;
use glam::{Vec3Swizzles, Vec4Swizzles};

#[derive(Debug)]
pub struct PrimitiveData {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
}

impl PrimitiveData {
    pub fn cube() -> PrimitiveData {
        let (vertices, indices) = Self::create_cube_vertices();
        PrimitiveData { vertices, indices }
    }

    fn vertex(position: glam::Vec4, tex_coord: glam::Vec2) -> MeshVertex {
        MeshVertex {
            position: position.xyz(),
            tex_coord,
            vertex_color: glam::vec4(0.0, 0.0, 0.0, 0.0),
            normal: glam::vec3(0.0, 0.0, 1.0),
        }
    }

    fn append_component(vector: &glam::Vec3) -> glam::Vec4 {
        let mut ret = vector.xyzx();
        ret.w = 1.0;
        ret
    }

    fn create_cube_vertices() -> (Vec<MeshVertex>, Vec<u32>) {
        let base_plane_data = [
            Self::vertex(glam::vec4(-1.0, 1.0, 0.0, 1.0), glam::vec2(0.0, 0.0)),
            Self::vertex(glam::vec4(1.0, 1.0, 0.0, 1.0), glam::vec2(1.0, 0.0)),
            Self::vertex(glam::vec4(1.0, -1.0, 0.0, 1.0), glam::vec2(1.0, 1.0)),
            Self::vertex(glam::vec4(-1.0, -1.0, 0.0, 1.0), glam::vec2(0.0, 1.0)),
        ];

        let front_plane_data = base_plane_data.map(|item| {
            let translation = glam::Mat4::from_translation(glam::Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            });
            Self::vertex(
                translation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let back_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_y(180.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let left_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_y(-90.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let right_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_y(90.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let top_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_x(-90.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let bottom_plane_data = front_plane_data.map(|item| {
            let rotation = glam::Mat4::from_rotation_x(90.0_f32.to_radians());
            Self::vertex(
                rotation * Self::append_component(&item.position),
                item.tex_coord,
            )
        });

        let front_plane_index: Vec<u32> = [2, 1, 0, 3, 2, 0].to_vec();
        let back_plane_index: Vec<u32> = front_plane_index.iter().map(|item| item + 4).collect();
        let left_plane_index: Vec<u32> = back_plane_index.iter().map(|item| item + 4).collect();
        let right_plane_index: Vec<u32> = left_plane_index.iter().map(|item| item + 4).collect();
        let top_plane_index: Vec<u32> = right_plane_index.iter().map(|item| item + 4).collect();
        let bottom_plane_index: Vec<u32> = top_plane_index.iter().map(|item| item + 4).collect();

        (
            [
                front_plane_data,
                back_plane_data,
                left_plane_data,
                right_plane_data,
                top_plane_data,
                bottom_plane_data,
            ]
            .concat()
            .to_vec(),
            [
                front_plane_index,
                back_plane_index,
                left_plane_index,
                right_plane_index,
                top_plane_index,
                bottom_plane_index,
            ]
            .concat()
            .to_vec(),
        )
    }
}
