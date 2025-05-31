use rapier3d::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DebugRenderStyle {
    pub subdivisions: u32,
    pub border_subdivisions: u32,
    pub collider_dynamic_color: glam::Vec4,
    pub collider_fixed_color: glam::Vec4,
    pub collider_kinematic_color: glam::Vec4,
    pub collider_parentless_color: glam::Vec4,
    pub impulse_joint_anchor_color: glam::Vec4,
    pub impulse_joint_separation_color: glam::Vec4,
    pub multibody_joint_anchor_color: glam::Vec4,
    pub multibody_joint_separation_color: glam::Vec4,
    pub sleep_color_multiplier: [f32; 4],
    pub disabled_color_multiplier: [f32; 4],
    pub rigid_body_axes_length: f32,
    pub contact_depth_color: glam::Vec4,
    pub contact_normal_color: glam::Vec4,
    pub contact_normal_length: f32,
    pub collider_aabb_color: glam::Vec4,
}

impl Default for DebugRenderStyle {
    fn default() -> Self {
        Self {
            subdivisions: 20,
            border_subdivisions: 5,
            collider_dynamic_color: [340.0, 1.0, 0.3, 1.0].into(),
            collider_kinematic_color: [20.0, 1.0, 0.3, 1.0].into(),
            collider_fixed_color: [30.0, 1.0, 0.4, 1.0].into(),
            collider_parentless_color: [30.0, 1.0, 0.4, 1.0].into(),
            impulse_joint_anchor_color: [240.0, 0.5, 0.4, 1.0].into(),
            impulse_joint_separation_color: [0.0, 0.5, 0.4, 1.0].into(),
            multibody_joint_anchor_color: [300.0, 1.0, 0.4, 1.0].into(),
            multibody_joint_separation_color: [0.0, 1.0, 0.4, 1.0].into(),
            sleep_color_multiplier: [1.0, 1.0, 0.2, 1.0],
            disabled_color_multiplier: [0.0, 0.0, 1.0, 1.0],
            rigid_body_axes_length: 0.5,
            contact_depth_color: [120.0, 1.0, 0.4, 1.0].into(),
            contact_normal_color: [0.0, 1.0, 1.0, 1.0].into(),
            contact_normal_length: 0.3,
            collider_aabb_color: [124.0, 1.0, 0.4, 1.0].into(),
        }
    }
}

pub struct RenderRigidBodiesBundle {
    pub start: glam::Vec3,
    pub end: glam::Vec3,
    pub color: glam::Vec4,
}

pub struct PhysicsDebugRender {
    pub style: DebugRenderStyle,
}

impl PhysicsDebugRender {
    pub fn new() -> PhysicsDebugRender {
        PhysicsDebugRender {
            style: Default::default(),
        }
    }

    pub fn render_rigid_bodies(&self, bodies: &RigidBodySet) -> Vec<RenderRigidBodiesBundle> {
        let mut bundle = vec![];
        for (_, rb) in bodies.iter() {
            if self.style.rigid_body_axes_length != 0.0 {
                let basis = rb.rotation().to_rotation_matrix().into_inner();
                let coeff = if !rb.is_enabled() {
                    self.style.disabled_color_multiplier
                } else if rb.is_sleeping() {
                    self.style.sleep_color_multiplier
                } else {
                    [1.0; 4]
                };
                let colors = [
                    [0.0 * coeff[0], 1.0 * coeff[1], 0.25 * coeff[2], coeff[3]],
                    [120.0 * coeff[0], 1.0 * coeff[1], 0.1 * coeff[2], coeff[3]],
                    [240.0 * coeff[0], 1.0 * coeff[1], 0.2 * coeff[2], coeff[3]],
                ];

                let com = rb
                    .position()
                    .transform_point(&rb.mass_properties().local_mprops.local_com);

                for k in 0..DIM {
                    let axis = basis.column(k) * self.style.rigid_body_axes_length;
                    let com = Self::point_to_vec3(&com);
                    let axis = glam::vec3(axis.x, axis.y, axis.z);
                    bundle.push(RenderRigidBodiesBundle {
                        start: com,
                        end: com + axis,
                        color: colors[k].into(),
                    });
                }
            }
        }
        bundle
    }

    pub fn render_colliders(
        &mut self,
        bodies: &RigidBodySet,
        colliders: &ColliderSet,
    ) -> Vec<RenderRigidBodiesBundle> {
        let mut bundle = vec![];

        for (_, co) in colliders.iter() {
            let color = if let Some(parent) = co.parent().and_then(|p| bodies.get(p)) {
                let coeff = if !parent.is_enabled() || !co.is_enabled() {
                    self.style.disabled_color_multiplier
                } else if parent.is_sleeping() {
                    self.style.sleep_color_multiplier
                } else {
                    [1.0; 4]
                };
                let c = match parent.body_type() {
                    RigidBodyType::Fixed => self.style.collider_fixed_color,
                    RigidBodyType::Dynamic => self.style.collider_dynamic_color,
                    RigidBodyType::KinematicPositionBased
                    | RigidBodyType::KinematicVelocityBased => self.style.collider_kinematic_color,
                };

                [
                    c[0] * coeff[0],
                    c[1] * coeff[1],
                    c[2] * coeff[2],
                    c[3] * coeff[3],
                ]
                .into()
            } else {
                self.style.collider_parentless_color
            };

            let mut sub_bundle = self.render_shape(co.shape(), co.position(), color);
            bundle.append(&mut sub_bundle);
        }

        bundle
    }

    fn render_shape(
        &mut self,
        shape: &dyn Shape,
        pos: &Isometry<f32>,
        color: glam::Vec4,
    ) -> Vec<RenderRigidBodiesBundle> {
        let mut bundle = vec![];
        match shape.as_typed_shape() {
            TypedShape::Ball(_) => todo!(),
            TypedShape::Cuboid(cuboid) => {
                let cuboid_outline = Cuboid::new(cuboid.half_extents).to_outline();
                let (vertexes, indexes) = cuboid_outline;
                let mut lines: Vec<RenderRigidBodiesBundle> = Vec::with_capacity(indexes.len());
                for index in indexes {
                    let line = RenderRigidBodiesBundle {
                        start: Self::point_to_vec3(&(pos * vertexes[index[0] as usize])),
                        end: Self::point_to_vec3(&(pos * vertexes[index[1] as usize])),
                        color,
                    };
                    lines.push(line);
                }
                bundle.append(&mut lines);
            }
            TypedShape::Capsule(_) => todo!(),
            TypedShape::Segment(_) => todo!(),
            TypedShape::Triangle(triangle) => {
                let line1 = RenderRigidBodiesBundle {
                    start: Self::point_to_vec3(&(pos * triangle.a)),
                    end: Self::point_to_vec3(&(pos * triangle.b)),
                    color,
                };

                let line2 = RenderRigidBodiesBundle {
                    start: Self::point_to_vec3(&(pos * triangle.b)),
                    end: Self::point_to_vec3(&(pos * triangle.c)),
                    color,
                };

                let line3 = RenderRigidBodiesBundle {
                    start: Self::point_to_vec3(&(pos * triangle.c)),
                    end: Self::point_to_vec3(&(pos * triangle.a)),
                    color,
                };
                bundle.append(&mut vec![line1, line2, line3]);
            }
            TypedShape::TriMesh(tri_mesh) => {
                for triangle in tri_mesh.triangles() {
                    let mut shape_bundle = self.render_shape(&triangle, pos, color);
                    bundle.append(&mut shape_bundle);
                }
            }
            TypedShape::Polyline(_) => todo!(),
            TypedShape::HalfSpace(_) => todo!(),
            TypedShape::HeightField(_) => todo!(),
            TypedShape::Compound(compound) => {
                for (sub_pos, shape) in compound.shapes() {
                    let mut shape_bundle = self.render_shape(&**shape, &(pos * sub_pos), color);
                    bundle.append(&mut shape_bundle);
                }
            }
            TypedShape::ConvexPolyhedron(_) => todo!(),
            TypedShape::Cylinder(_) => todo!(),
            TypedShape::Cone(_) => todo!(),
            TypedShape::RoundCuboid(_) => todo!(),
            TypedShape::RoundTriangle(_) => todo!(),
            TypedShape::RoundCylinder(_) => todo!(),
            TypedShape::RoundCone(_) => todo!(),
            TypedShape::RoundConvexPolyhedron(_) => todo!(),
            TypedShape::Custom(_) => todo!(),
            TypedShape::Voxels(_) => todo!(),
        }
        bundle
    }

    fn point_to_vec3(point: &Point<f32>) -> glam::Vec3 {
        glam::vec3(point.x, point.y, point.z)
    }
}
