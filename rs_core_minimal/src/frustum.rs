use crate::plane_3d::Plane3D;

pub struct Line {
    pub p_0: glam::Vec3,
    pub p_1: glam::Vec3,
}

pub struct FrustumPlanes {
    pub left_plane: Plane3D,
    pub right_plane: Plane3D,
    pub top_plane: Plane3D,
    pub bottom_plane: Plane3D,
    pub front_plane: Plane3D,
    pub back_plane: Plane3D,
}

impl FrustumPlanes {
    pub fn new(frustum: &Frustum) -> FrustumPlanes {
        let left_plane = Plane3D::new(
            (frustum.far_2 - frustum.near_2)
                .cross(frustum.near_3 - frustum.near_2)
                .normalize(),
            (frustum.far_2 + frustum.far_3 + frustum.near_2 + frustum.near_3) / 4.0,
        );
        let right_plane = Plane3D::new(
            (frustum.near_0 - frustum.near_1)
                .cross(frustum.far_1 - frustum.near_1)
                .normalize(),
            (frustum.far_0 + frustum.far_1 + frustum.near_0 + frustum.near_1) / 4.0,
        );
        let top_plane = Plane3D::new(
            (frustum.near_3 - frustum.near_0)
                .cross(frustum.far_0 - frustum.near_0)
                .normalize(),
            (frustum.far_0 + frustum.far_3 + frustum.near_0 + frustum.near_3) / 4.0,
        );
        let bottom_plane = Plane3D::new(
            (frustum.far_1 - frustum.near_1)
                .cross(frustum.near_2 - frustum.near_1)
                .normalize(),
            (frustum.far_1 + frustum.far_2 + frustum.near_1 + frustum.near_2) / 4.0,
        );
        let front_plane = Plane3D::new(
            (frustum.near_2 - frustum.near_1)
                .cross(frustum.near_0 - frustum.near_1)
                .normalize(),
            (frustum.near_0 + frustum.near_1 + frustum.near_2 + frustum.near_3) / 4.0,
        );
        let back_plane = Plane3D::new(
            (frustum.far_0 - frustum.far_1)
                .cross(frustum.far_2 - frustum.far_1)
                .normalize(),
            (frustum.far_0 + frustum.far_1 + frustum.far_2 + frustum.far_3) / 4.0,
        );
        FrustumPlanes {
            left_plane,
            right_plane,
            top_plane,
            bottom_plane,
            front_plane,
            back_plane,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Frustum {
    pub near_0: glam::Vec3,
    pub near_1: glam::Vec3,
    pub near_2: glam::Vec3,
    pub near_3: glam::Vec3,
    pub far_0: glam::Vec3,
    pub far_1: glam::Vec3,
    pub far_2: glam::Vec3,
    pub far_3: glam::Vec3,
}

impl Frustum {
    pub fn transform(&self, transform: &glam::Mat4) -> Frustum {
        Frustum {
            near_0: transform.transform_point3(self.near_0),
            near_1: transform.transform_point3(self.near_1),
            near_2: transform.transform_point3(self.near_2),
            near_3: transform.transform_point3(self.near_3),
            far_0: transform.transform_point3(self.far_0),
            far_1: transform.transform_point3(self.far_1),
            far_2: transform.transform_point3(self.far_2),
            far_3: transform.transform_point3(self.far_3),
        }
    }

    pub fn make_lines(&self) -> Vec<Line> {
        let mut near_lines = Self::lines_from_points(vec![
            self.near_0,
            self.near_1,
            self.near_2,
            self.near_3,
            self.near_0,
        ]);
        let mut far_lines = Self::lines_from_points(vec![
            self.far_0, self.far_1, self.far_2, self.far_3, self.far_0,
        ]);
        let mut tr_lines = Self::lines_from_points(vec![self.near_0, self.far_0]);
        let mut br_lines = Self::lines_from_points(vec![self.near_1, self.far_1]);
        let mut tl_lines = Self::lines_from_points(vec![self.near_2, self.far_2]);
        let mut bl_lines = Self::lines_from_points(vec![self.near_3, self.far_3]);
        let mut lines = vec![];

        lines.append(&mut near_lines);
        lines.append(&mut far_lines);
        lines.append(&mut tr_lines);
        lines.append(&mut br_lines);
        lines.append(&mut tl_lines);
        lines.append(&mut bl_lines);

        lines
    }

    pub fn make_normal_lines(&self, length: f32) -> Vec<Line> {
        let mut lines = Vec::with_capacity(6);

        let FrustumPlanes {
            left_plane,
            right_plane,
            top_plane,
            bottom_plane,
            front_plane,
            back_plane,
        } = FrustumPlanes::new(self);

        lines.push(Line {
            p_0: left_plane.point,
            p_1: left_plane.point + left_plane.normal_vector * length,
        });
        lines.push(Line {
            p_0: right_plane.point,
            p_1: right_plane.point + right_plane.normal_vector * length,
        });
        lines.push(Line {
            p_0: top_plane.point,
            p_1: top_plane.point + top_plane.normal_vector * length,
        });
        lines.push(Line {
            p_0: bottom_plane.point,
            p_1: bottom_plane.point + bottom_plane.normal_vector * length,
        });
        lines.push(Line {
            p_0: front_plane.point,
            p_1: front_plane.point + front_plane.normal_vector * length,
        });
        lines.push(Line {
            p_0: back_plane.point,
            p_1: back_plane.point + back_plane.normal_vector * length,
        });

        lines
    }

    fn lines_from_points(points: Vec<glam::Vec3>) -> Vec<Line> {
        points
            .windows(2)
            .map(|x| Line {
                p_0: x[0],
                p_1: x[1],
            })
            .collect()
    }
}
