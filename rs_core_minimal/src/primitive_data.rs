use std::{f32::consts::PI, iter::zip, num::NonZeroUsize};

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

    pub fn cube_lines(color: Option<glam::Vec4>) -> PrimitiveData {
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
        let color = color.unwrap_or_else(|| glam::vec4(0.0, 0.0, 0.0, 1.0));
        PrimitiveData {
            vertex_colors: vec![color; vertex_count],
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
        color: Option<glam::Vec4>,
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

        let color = color.unwrap_or_else(|| Default::default());
        primitive_data
            .vertex_colors
            .resize(primitive_data.vertex_positions.len(), color);
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

    pub fn arrow(arrow_options: ArrowOptions) -> PrimitiveData {
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut tangents = Vec::new();
        let mut bitangents = Vec::new();
        let mut tex_coords = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();

        let ArrowOptions {
            segments,
            shaft_height,
            shaft_radius,
            cone_height,
            cone_radius,
        } = arrow_options;

        for i in 0..segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let next = (i + 1) % segments;
            let x0 = shaft_radius * theta.cos();
            let z0 = shaft_radius * theta.sin();
            let x1 = shaft_radius * (2.0 * PI * next as f32 / segments as f32).cos();
            let z1 = shaft_radius * (2.0 * PI * next as f32 / segments as f32).sin();

            let base0 = glam::Vec3::new(x0, 0.0, z0);
            let top0 = glam::Vec3::new(x0, shaft_height, z0);
            let base1 = glam::Vec3::new(x1, 0.0, z1);
            let top1 = glam::Vec3::new(x1, shaft_height, z1);

            let base_index = positions.len() as u32;
            positions.extend([base0, top0, base1, top1]);

            let normal = glam::Vec3::new(x0, 0.0, z0).normalize();
            normals.extend([normal; 4]);

            tangents.extend([glam::Vec3::X; 4]);
            bitangents.extend([glam::Vec3::Y; 4]);

            tex_coords.extend([
                glam::Vec2::new(i as f32 / segments as f32, 0.0),
                glam::Vec2::new(i as f32 / segments as f32, 1.0),
                glam::Vec2::new(next as f32 / segments as f32, 0.0),
                glam::Vec2::new(next as f32 / segments as f32, 1.0),
            ]);

            colors.extend([glam::vec4(1.0, 0.0, 0.0, 1.0); 4]);

            indices.extend_from_slice(&[
                base_index,
                base_index + 1,
                base_index + 2,
                base_index + 1,
                base_index + 3,
                base_index + 2,
            ]);
        }

        let tip = glam::Vec3::new(0.0, shaft_height + cone_height, 0.0);
        let tip_index = positions.len() as u32;
        positions.push(tip);
        normals.push(glam::Vec3::Y);
        tangents.push(glam::Vec3::X);
        bitangents.push(glam::Vec3::Z);
        tex_coords.push(glam::Vec2::new(0.5, 1.0));
        colors.push(glam::Vec4::ONE);

        for i in 0..segments {
            let theta = 2.0 * PI * i as f32 / segments as f32;
            let next = (i + 1) % segments;
            let x0 = cone_radius * theta.cos();
            let z0 = cone_radius * theta.sin();
            let x1 = cone_radius * (2.0 * PI * next as f32 / segments as f32).cos();
            let z1 = cone_radius * (2.0 * PI * next as f32 / segments as f32).sin();

            let base0 = glam::Vec3::new(x0, shaft_height, z0);
            let base1 = glam::Vec3::new(x1, shaft_height, z1);

            let base_index = positions.len() as u32;
            positions.extend([base0, base1]);
            normals.extend([glam::Vec3::Y; 2]);
            tangents.extend([glam::Vec3::X; 2]);
            bitangents.extend([glam::Vec3::Z; 2]);
            tex_coords.extend([glam::Vec2::new(0.0, 0.0); 2]);
            colors.extend([glam::Vec4::ONE; 2]);

            indices.extend_from_slice(&[tip_index, base_index, base_index + 1]);
        }

        PrimitiveData {
            vertex_positions: positions,
            vertex_normals: normals,
            vertex_tangents: tangents,
            vertex_bitangents: bitangents,
            vertex_tex_coords: tex_coords,
            vertex_colors: colors,
            indices,
        }
    }

    pub fn apply_transformation(&mut self, transformation: glam::Mat4) {
        self.vertex_positions = self
            .vertex_positions
            .iter()
            .map(|x| transformation.project_point3(*x))
            .collect();
    }
}

pub struct ArrowOptions {
    pub segments: i32,
    pub shaft_height: f32,
    pub shaft_radius: f32,
    pub cone_height: f32,
    pub cone_radius: f32,
}

impl Default for ArrowOptions {
    fn default() -> Self {
        Self {
            segments: 16,
            shaft_height: 1.0,
            shaft_radius: 0.05,
            cone_height: 0.3,
            cone_radius: 0.15,
        }
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
