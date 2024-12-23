use crate::{
    camera::Camera,
    components::{component::Component, point_light_component::PointLightComponent},
    engine::Engine,
};
use rs_core_minimal::{
    misc::{is_sphere_visible_to_frustum, split_frustum},
    sphere_3d::Sphere3D,
};
use rs_render::{constants::ClusterLightIndex, global_uniform::CameraFrustum};

struct ResolveResult {
    cluster_light_handle: crate::handle::BufferHandle,
    cluster_light_index_handle: crate::handle::BufferHandle,
}

pub struct ClusterLight {
    pub point_lights_handle: crate::handle::BufferHandle,
    pub cluster_light_handle: crate::handle::BufferHandle,
    pub cluster_light_index_handle: crate::handle::BufferHandle,
    pub scene_points_lights: Option<rs_render::command::SceneLight>,
    pub is_enable_light_culling_acceleration: bool,
}

impl ClusterLight {
    pub fn new(
        engine: &mut Engine,
        camera: &Camera,
        point_light_components: Vec<&PointLightComponent>,
        is_enable_light_culling_acceleration: bool,
    ) -> crate::error::Result<ClusterLight> {
        let frustum = camera.get_frustum_apply_tramsformation();
        let mut point_light_shapes = Vec::with_capacity(point_light_components.len());
        let ResolveResult {
            cluster_light_handle,
            cluster_light_index_handle,
        } = if is_enable_light_culling_acceleration {
            for point_light_component in &point_light_components {
                let shape = Self::get_sphere_of_point_light(point_light_component);
                let render_shape = rs_render::constants::Sphere3D::new(shape.center, shape.radius);
                point_light_shapes.push(render_shape);
            }

            let cluster_light_handle = engine.get_resource_manager().next_buffer();
            let cluster_light_index_handle = engine.get_resource_manager().next_buffer();
            ResolveResult {
                cluster_light_handle,
                cluster_light_index_handle,
            }
        } else {
            let result = Self::resolve(engine, camera, point_light_components.clone());
            let fallback = Self::no_lights_fall_back(engine);
            result.or(fallback)?
        };
        let scene_points_lights = if is_enable_light_culling_acceleration {
            let scene_points_lights = rs_render::command::SceneLight {
                point_light_shapes,
                frustum: CameraFrustum::new(
                    frustum.near_0,
                    frustum.near_1,
                    frustum.near_2,
                    frustum.near_3,
                    frustum.far_0,
                    frustum.far_1,
                    frustum.far_2,
                    frustum.far_3,
                ),
                cluster_lights_placeholder: *cluster_light_handle,
                cluster_light_indices_placeholder: *cluster_light_index_handle,
            };
            Some(scene_points_lights)
        } else {
            None
        };

        let point_lights_handle =
            Self::get_point_lights_buffer_handle(engine, point_light_components)?;
        Ok(ClusterLight {
            point_lights_handle,
            cluster_light_handle,
            cluster_light_index_handle,
            scene_points_lights,
            is_enable_light_culling_acceleration,
        })
    }

    fn no_lights_fall_back(engine: &mut Engine) -> crate::error::Result<ResolveResult> {
        let cluster_light: Vec<u32> = vec![0];
        let content = rs_foundation::cast_to_raw_buffer(&cluster_light);
        let cluster_light_handle = engine.create_buffer(
            content.to_vec(),
            wgpu::BufferUsages::STORAGE,
            Some("ClusterLight".to_string()),
        )?;

        let cluster_light_index: Vec<ClusterLightIndex> = vec![ClusterLightIndex {
            offset: 0,
            count: 0,
        }];
        let content = rs_foundation::cast_to_raw_buffer(&cluster_light_index);
        let cluster_light_index_handle = engine.create_buffer(
            content.to_vec(),
            wgpu::BufferUsages::STORAGE,
            Some("ClusterLightIndex".to_string()),
        )?;

        Ok(ResolveResult {
            cluster_light_handle,
            cluster_light_index_handle,
        })
    }

    fn get_point_lights_buffer_handle(
        engine: &mut Engine,
        point_light_components: Vec<&PointLightComponent>,
    ) -> crate::error::Result<crate::handle::BufferHandle> {
        let point_lights = if point_light_components.is_empty() {
            let point_lights = vec![rs_render::constants::PointLight::default()];
            point_lights
        } else {
            let point_lights = point_light_components
                .iter()
                .map(|x| {
                    let mut p = rs_render::constants::PointLight::default();
                    p.ambient = x.point_light.ambient;
                    p.diffuse = x.point_light.diffuse;
                    p.specular = x.point_light.specular;
                    p.quadratic = x.point_light.quadratic;
                    p.linear = x.point_light.linear;
                    p.constant = x.point_light.constant;
                    p.position = x
                        .get_final_transformation()
                        .to_scale_rotation_translation()
                        .2;
                    p
                })
                .collect::<Vec<rs_render::constants::PointLight>>();
            point_lights
        };
        let content = rs_foundation::cast_to_raw_buffer(&point_lights);
        let point_lights_handle = engine.create_buffer(
            content.to_vec(),
            wgpu::BufferUsages::STORAGE,
            Some("PointLightsBuffer".to_string()),
        )?;
        Ok(point_lights_handle)
    }

    fn resolve(
        engine: &mut Engine,
        camera: &Camera,
        point_light_components: Vec<&PointLightComponent>,
    ) -> crate::error::Result<ResolveResult> {
        let _ = tracy_client::span!();

        if point_light_components.is_empty() {
            return Self::no_lights_fall_back(engine);
        }

        let frustum = camera.get_frustum_apply_tramsformation();
        const SPLIT_NUM: usize = 9;
        let frustums = split_frustum(&frustum, SPLIT_NUM, SPLIT_NUM, SPLIT_NUM);

        let mut cluster_light: Vec<u32> = vec![];
        let mut cluster_light_index: Vec<ClusterLightIndex> = Vec::with_capacity(frustums.len());

        for frustum in frustums.iter() {
            let mut cluster = vec![];
            for (light_index, point_light_component) in point_light_components.iter().enumerate() {
                let sphere = Self::get_sphere_of_point_light(&point_light_component);
                let is_visible = is_sphere_visible_to_frustum(&sphere, frustum);
                if is_visible {
                    cluster.push(light_index as u32);
                }
            }
            cluster_light_index.push(ClusterLightIndex {
                offset: cluster_light.len() as u32,
                count: cluster.len() as u32,
            });
            cluster_light.append(&mut cluster);
        }

        let content = rs_foundation::cast_to_raw_buffer(&cluster_light);
        let cluster_light_handle = engine.create_buffer(
            content.to_vec(),
            wgpu::BufferUsages::STORAGE,
            Some("ClusterLight".to_string()),
        )?;

        let content = rs_foundation::cast_to_raw_buffer(&cluster_light_index);
        let cluster_light_index_handle = engine.create_buffer(
            content.to_vec(),
            wgpu::BufferUsages::STORAGE,
            Some("ClusterLightIndex".to_string()),
        )?;

        Ok(ResolveResult {
            cluster_light_handle,
            cluster_light_index_handle,
        })
    }

    fn get_sphere_of_point_light(point_light_component: &PointLightComponent) -> Sphere3D {
        let radius = point_light_component.get_radius();
        let center = point_light_component
            .get_final_transformation()
            .to_scale_rotation_translation()
            .2;
        Sphere3D { center, radius }
    }
}
