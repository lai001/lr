use std::{iter::zip, num::NonZeroUsize};

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

    pub fn cube() -> PrimitiveData {
        let front_top_left = glam::vec3(-1.0, 1.0, -1.0);
        let front_top_right = glam::vec3(1.0, 1.0, -1.0);
        let front_bottom_left = glam::vec3(-1.0, -1.0, -1.0);
        let front_bottom_right = glam::vec3(1.0, -1.0, -1.0);
        let back_top_left = glam::vec3(-1.0, 1.0, 1.0);
        let back_top_right = glam::vec3(1.0, 1.0, 1.0);
        let back_bottom_left = glam::vec3(-1.0, -1.0, 1.0);
        let back_bottom_right = glam::vec3(1.0, -1.0, 1.0);

        let front_top_left_coord = glam::vec2(0.0, 0.0);
        let front_top_right_coord = glam::vec2(1.0, 0.0);
        let front_bottom_left_coord = glam::vec2(0.0, 1.0);
        let front_bottom_right_coord = glam::vec2(1.0, 1.0);
        let back_top_left_coord = glam::vec2(0.0, 0.0);
        let back_top_right_coord = glam::vec2(1.0, 0.0);
        let back_bottom_left_coord = glam::vec2(0.0, 1.0);
        let back_bottom_right_coord = glam::vec2(1.0, 1.0);

        let indices: Vec<u32> = vec![
            Self::quad_index_to_triangles_clockwise([0, 1, 2, 3]),
            Self::quad_index_to_triangles_clockwise([5, 4, 7, 6]),
            Self::quad_index_to_triangles_clockwise([4, 5, 1, 0]),
            Self::quad_index_to_triangles_clockwise([3, 2, 6, 7]),
            Self::quad_index_to_triangles_clockwise([4, 0, 3, 7]),
            Self::quad_index_to_triangles_clockwise([1, 5, 6, 2]),
        ]
        .drain(..)
        .flat_map(|x| x)
        .flat_map(|x| x)
        .collect();

        let vertex_count = 8;
        PrimitiveData {
            vertex_colors: vec![glam::vec4(0.0, 0.0, 0.0, 1.0); vertex_count],
            vertex_positions: vec![
                front_top_left,
                front_top_right,
                front_bottom_right,
                front_bottom_left,
                back_top_left,
                back_top_right,
                back_bottom_right,
                back_bottom_left,
            ],
            vertex_normals: vec![glam::vec3(0.5, 0.5, 1.0,); vertex_count],
            vertex_tangents: vec![glam::Vec3::X; vertex_count],
            vertex_bitangents: vec![glam::Vec3::Y; vertex_count],
            vertex_tex_coords: vec![
                front_top_left_coord,
                front_top_right_coord,
                front_bottom_left_coord,
                front_bottom_right_coord,
                back_top_left_coord,
                back_top_right_coord,
                back_bottom_left_coord,
                back_bottom_right_coord,
            ],
            indices,
        }
    }

    fn quad_index_to_triangles_clockwise(index: [u32; 4]) -> [[u32; 3]; 2] {
        [
            [index[0], index[1], index[3]],
            [index[1], index[2], index[3]],
        ]
    }

    pub fn cube_lines() -> PrimitiveData {
        let front_top_left = glam::vec3(-1.0, 1.0, -1.0);
        let front_top_right = glam::vec3(1.0, 1.0, -1.0);
        let front_bottom_left = glam::vec3(-1.0, -1.0, -1.0);
        let front_bottom_right = glam::vec3(1.0, -1.0, -1.0);
        let back_top_left = glam::vec3(-1.0, 1.0, 1.0);
        let back_top_right = glam::vec3(1.0, 1.0, 1.0);
        let back_bottom_left = glam::vec3(-1.0, -1.0, 1.0);
        let back_bottom_right = glam::vec3(1.0, -1.0, 1.0);

        let front_top_left_coord = glam::vec2(0.0, 0.0);
        let front_top_right_coord = glam::vec2(1.0, 0.0);
        let front_bottom_left_coord = glam::vec2(0.0, 1.0);
        let front_bottom_right_coord = glam::vec2(1.0, 1.0);
        let back_top_left_coord = glam::vec2(0.0, 0.0);
        let back_top_right_coord = glam::vec2(1.0, 0.0);
        let back_bottom_left_coord = glam::vec2(0.0, 1.0);
        let back_bottom_right_coord = glam::vec2(1.0, 1.0);

        let indices: Vec<u32> = vec![
            [0, 1],
            [1, 2],
            [2, 3],
            [3, 0],
            [4, 5],
            [5, 6],
            [6, 7],
            [7, 4],
            [0, 4],
            [1, 5],
            [2, 6],
            [3, 7],
        ]
        .drain(..)
        .flat_map(|x| x)
        .collect();

        let vertex_count = 8;
        PrimitiveData {
            vertex_colors: vec![glam::vec4(0.0, 0.0, 0.0, 1.0); vertex_count],
            vertex_positions: vec![
                front_top_left,
                front_top_right,
                front_bottom_right,
                front_bottom_left,
                back_top_left,
                back_top_right,
                back_bottom_right,
                back_bottom_left,
            ],
            vertex_normals: vec![glam::vec3(0.5, 0.5, 1.0,); vertex_count],
            vertex_tangents: vec![glam::Vec3::X; vertex_count],
            vertex_bitangents: vec![glam::Vec3::Y; vertex_count],
            vertex_tex_coords: vec![
                front_top_left_coord,
                front_top_right_coord,
                front_bottom_left_coord,
                front_bottom_right_coord,
                back_top_left_coord,
                back_top_right_coord,
                back_bottom_left_coord,
                back_bottom_right_coord,
            ],
            indices,
        }
    }

    pub fn sphere(
        radius: f32,
        h_subdivide: NonZeroUsize,
        v_subdivide: NonZeroUsize,
        is_cylinder: bool,
    ) -> PrimitiveData {
        let h_subdivide = h_subdivide.get();
        let v_subdivide = v_subdivide.get();
        assert!(v_subdivide >= 2);

        let mut primitive_data = PrimitiveData {
            vertex_colors: vec![],
            vertex_positions: vec![],
            vertex_normals: vec![],
            vertex_tangents: vec![],
            vertex_bitangents: vec![],
            vertex_tex_coords: vec![],
            indices: vec![],
        };

        let north = glam::Vec3::Y;
        let south = glam::Vec3::NEG_Y;

        let mut vertexes: Vec<Vec<glam::Vec3>> =
            vec![vec![glam::Vec3::ZERO; v_subdivide]; h_subdivide];

        for i in 0..h_subdivide {
            let radian = (i + 1) as f32 * std::f32::consts::PI / (h_subdivide as f32 + 1.0f32);
            let y = (radian + std::f32::consts::FRAC_PI_2).sin();
            if is_cylinder {
                for j in 0..v_subdivide {
                    let radian = std::f32::consts::TAU * (j as f32 / v_subdivide as f32);
                    let x = radian.cos();
                    let z = radian.sin();
                    let vertex = glam::vec3(x, y, z);
                    vertexes[i][j] = vertex;
                }
            } else {
                let projection_length = radian.sin();
                for j in 0..v_subdivide {
                    let radian = std::f32::consts::TAU * (j as f32 / v_subdivide as f32);
                    let x = radian.cos() * projection_length;
                    let z = radian.sin() * projection_length;
                    let vertex = glam::vec3(x, y, z);
                    vertexes[i][j] = vertex;
                }
            }
        }

        let top = vertexes.first().expect("Not null");
        for item in top.windows(2) {
            let mut triangles = vec![north, item[0], item[1]];
            primitive_data.vertex_positions.append(&mut triangles);
        }

        for group in (0..vertexes.len())
            .map(|x| x)
            .collect::<Vec<usize>>()
            .windows(2)
        {
            let i = group[0];
            let j = group[1];
            let mut vertexes_0 = vertexes[i].clone();
            vertexes_0.push(vertexes[i][0]);
            let mut vertexes_1 = vertexes[j].clone();
            vertexes_1.push(vertexes[j][0]);

            for (item0, item1) in zip(vertexes_0.windows(2), vertexes_1.windows(2)) {
                let mut triangle = vec![item0[0], item0[1], item1[0]];
                primitive_data.vertex_positions.append(&mut triangle);
                let mut triangle = vec![item0[1], item1[1], item1[0]];
                primitive_data.vertex_positions.append(&mut triangle);
            }
        }

        let bottom = vertexes.last().expect("Not null");
        for item in bottom.windows(2) {
            let mut triangle = vec![south, item[0], item[1]];
            primitive_data.vertex_positions.append(&mut triangle);
        }

        for position in &mut primitive_data.vertex_positions {
            *position = *position * radius;
        }

        primitive_data
            .vertex_colors
            .resize(primitive_data.vertex_positions.len(), Default::default());
        primitive_data
            .vertex_normals
            .resize(primitive_data.vertex_positions.len(), Default::default());
        primitive_data
            .vertex_tangents
            .resize(primitive_data.vertex_positions.len(), Default::default());
        primitive_data
            .vertex_bitangents
            .resize(primitive_data.vertex_positions.len(), Default::default());
        primitive_data
            .vertex_tex_coords
            .resize(primitive_data.vertex_positions.len(), Default::default());
        primitive_data.indices = (0..primitive_data.vertex_positions.len())
            .map(|x| x as u32)
            .collect();

        primitive_data
    }
}

impl<'a> IntoIterator for &'a PrimitiveData {
    type Item = (
        &'a glam::Vec4,
        &'a glam::Vec3,
        &'a glam::Vec3,
        &'a glam::Vec3,
        &'a glam::Vec3,
        &'a glam::Vec2,
    );
    type IntoIter = PrimitiveDataIntoIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PrimitiveDataIntoIterator {
            primitive_data: self,
            index: 0,
        }
    }
}

pub struct PrimitiveDataIntoIterator<'a> {
    primitive_data: &'a PrimitiveData,
    index: usize,
}

impl<'a> Iterator for PrimitiveDataIntoIterator<'a> {
    type Item = (
        &'a glam::Vec4,
        &'a glam::Vec3,
        &'a glam::Vec3,
        &'a glam::Vec3,
        &'a glam::Vec3,
        &'a glam::Vec2,
    );
    fn next(&mut self) -> Option<Self::Item> {
        let vertex_color = self.primitive_data.vertex_colors.get(self.index);
        let vertex_position = self.primitive_data.vertex_positions.get(self.index);
        let vertex_normal = self.primitive_data.vertex_normals.get(self.index);
        let vertex_tangent = self.primitive_data.vertex_tangents.get(self.index);
        let vertex_bitangent = self.primitive_data.vertex_bitangents.get(self.index);
        let vertex_tex_coord = self.primitive_data.vertex_tex_coords.get(self.index);
        if let (
            Some(vertex_color),
            Some(vertex_position),
            Some(vertex_normal),
            Some(vertex_tangent),
            Some(vertex_bitangent),
            Some(vertex_tex_coord),
        ) = (
            vertex_color,
            vertex_position,
            vertex_normal,
            vertex_tangent,
            vertex_bitangent,
            vertex_tex_coord,
        ) {
            self.index += 1;
            Some((
                vertex_color,
                vertex_position,
                vertex_normal,
                vertex_tangent,
                vertex_bitangent,
                vertex_tex_coord,
            ))
        } else {
            None
        }
    }
}
