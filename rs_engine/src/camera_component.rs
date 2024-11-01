use crate::{
    build_built_in_resouce_url,
    camera::Camera,
    content::content_file_type::EContentFileType,
    drawable::{CustomDrawObject, EDrawObjectType},
    engine::Engine,
    input_mode::EInputMode,
    player_viewport::PlayerViewport,
};
use rapier3d::prelude::{ColliderSet, RigidBodySet};
use rs_core_minimal::{frustum::Frustum, misc::frustum_from_perspective};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use rs_render::{
    command::{DrawObject, EBindingResource, TextureDescriptorCreateInfo},
    constants,
    renderer::{EBuiltinPipelineType, EPipelineType},
    vertex_data_type::mesh_vertex::MeshVertex3,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct CameraComponentRuntime {
    pub player_viewport: SingleThreadMutType<PlayerViewport>,
    pub parent_final_transformation: glam::Mat4,
    pub final_transformation: glam::Mat4,
    draw_object: EDrawObjectType,
    constants_handle: crate::handle::BufferHandle,
    constants: constants::Constants,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CameraComponent {
    pub name: String,
    pub transformation: glam::Mat4,
    pub width: u32,
    pub height: u32,
    pub is_enable: bool,
    #[serde(skip)]
    pub run_time: Option<CameraComponentRuntime>,
}

impl CameraComponent {
    pub fn get_transformation_mut(&mut self) -> &mut glam::Mat4 {
        &mut self.transformation
    }

    pub fn get_transformation(&self) -> &glam::Mat4 {
        &self.transformation
    }

    pub fn set_parent_final_transformation(&mut self, parent_final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.parent_final_transformation = parent_final_transformation;
    }

    pub fn get_parent_final_transformation(&self) -> glam::Mat4 {
        let Some(run_time) = self.run_time.as_ref() else {
            return glam::Mat4::IDENTITY;
        };
        run_time.parent_final_transformation
    }

    pub fn set_final_transformation(&mut self, final_transformation: glam::Mat4) {
        let Some(run_time) = self.run_time.as_mut() else {
            return;
        };
        run_time.final_transformation = final_transformation;
    }

    pub fn get_final_transformation(&self) -> glam::Mat4 {
        self.run_time
            .as_ref()
            .map(|x| x.final_transformation)
            .unwrap_or_default()
    }

    pub fn new(name: String, transformation: glam::Mat4) -> CameraComponent {
        CameraComponent {
            name,
            transformation,
            run_time: None,
            width: 1024,
            height: 1024,
            is_enable: true,
        }
    }

    pub fn initialize(
        &mut self,
        engine: &mut Engine,
        files: &[EContentFileType],
        level_player_viewport: &mut PlayerViewport,
    ) {
        let _ = files;
        let Ok(rt_url) = build_built_in_resouce_url("CameraComponent.RT") else {
            return;
        };
        let Ok(depth_url) = build_built_in_resouce_url("CameraComponent.Depth") else {
            return;
        };
        let info = TextureDescriptorCreateInfo {
            label: Some("CameraComponent.RT".to_string()),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: Some(vec![wgpu::TextureFormat::Rgba8UnormSrgb]),
        };
        let color_texture_handle = engine.create_texture(&rt_url, info);
        let info = TextureDescriptorCreateInfo {
            label: Some("CameraComponent.Depth".to_string()),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: Some(vec![wgpu::TextureFormat::Depth32Float]),
        };
        let depth_texture_handle = engine.create_texture(&depth_url, info);

        let player_viewport = PlayerViewport::from_frame_buffer(
            color_texture_handle,
            depth_texture_handle,
            self.width,
            self.height,
            engine,
            EInputMode::Game,
            false,
        );

        let z_near = player_viewport.camera.get_z_near();
        let z_far = player_viewport.camera.get_z_far();
        let frustum = match player_viewport.camera.get_camera_type() {
            crate::camera::ECameraType::Perspective(perspective_properties) => {
                let frustum = frustum_from_perspective(
                    perspective_properties.fov_y_radians,
                    perspective_properties.aspect_ratio,
                    z_near,
                    z_far,
                );
                frustum
            }
            crate::camera::ECameraType::Orthographic(_) => unimplemented!(),
        };

        let (draw_object, constants_handle) =
            Self::make_draw_object(engine, &frustum, level_player_viewport);

        let render_target_type = *level_player_viewport.get_render_target_type();
        self.run_time = Some(CameraComponentRuntime {
            final_transformation: glam::Mat4::IDENTITY,
            parent_final_transformation: glam::Mat4::IDENTITY,
            player_viewport: SingleThreadMut::new(player_viewport),
            draw_object: EDrawObjectType::Custom(CustomDrawObject {
                draw_object,
                render_target_type,
            }),
            constants_handle,
            constants: constants::Constants::default(),
        })
    }

    fn make_draw_object(
        engine: &mut Engine,
        frustum: &Frustum,
        player_viewport: &mut PlayerViewport,
    ) -> (DrawObject, crate::handle::BufferHandle) {
        let lines = frustum.make_lines();
        let mut v1 = lines[0..4]
            .iter()
            .flat_map(|x| {
                vec![
                    MeshVertex3 {
                        position: x.p_0,
                        vertex_color: glam::vec4(0.0, 1.0, 0.0, 1.0),
                    },
                    MeshVertex3 {
                        position: x.p_1,
                        vertex_color: glam::vec4(0.0, 1.0, 0.0, 1.0),
                    },
                ]
            })
            .collect::<Vec<MeshVertex3>>();
        let mut v2 = lines[4..8]
            .iter()
            .flat_map(|x| {
                vec![
                    MeshVertex3 {
                        position: x.p_0,
                        vertex_color: glam::vec4(1.0, 0.0, 0.0, 1.0),
                    },
                    MeshVertex3 {
                        position: x.p_1,
                        vertex_color: glam::vec4(1.0, 0.0, 0.0, 1.0),
                    },
                ]
            })
            .collect::<Vec<MeshVertex3>>();

        let mut v3 = lines[8..]
            .iter()
            .flat_map(|x| {
                vec![
                    MeshVertex3 {
                        position: x.p_0,
                        vertex_color: glam::vec4(0.0, 1.0, 0.0, 1.0),
                    },
                    MeshVertex3 {
                        position: x.p_1,
                        vertex_color: glam::vec4(1.0, 0.0, 0.0, 1.0),
                    },
                ]
            })
            .collect::<Vec<MeshVertex3>>();

        let mut vertex: Vec<MeshVertex3> = vec![];
        vertex.append(&mut v1);
        vertex.append(&mut v2);
        vertex.append(&mut v3);

        let vertex_count = vertex.len();
        let vertex_buffer_handle =
            engine.create_vertex_buffer(&vertex, Some(format!("rs.VertexBuffer")));
        let constants_handle = engine.create_constants_buffer(
            &vec![constants::Constants::default()],
            Some(format!("rs.Constants")),
        );
        (
            DrawObject::new(
                0,
                vec![*vertex_buffer_handle],
                vertex_count as u32,
                EPipelineType::Builtin(EBuiltinPipelineType::Primitive),
                None,
                None,
                vec![
                    vec![EBindingResource::Constants(
                        *player_viewport.global_constants_handle,
                    )],
                    vec![EBindingResource::Constants(*constants_handle)],
                ],
            ),
            constants_handle,
        )
    }

    pub fn tick(
        &mut self,
        time: f32,
        engine: &mut Engine,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let _ = rigid_body_set;
        let _ = collider_set;
        let _ = time;
        let _ = engine;
        let Some(run_time) = &mut self.run_time else {
            return;
        };
        let mut player_viewport = run_time.player_viewport.borrow_mut();
        let camera = &mut player_viewport.camera;
        let final_transformation = run_time.final_transformation;
        camera.set_world_location(final_transformation.to_scale_rotation_translation().2);
        camera.set_forward_vector(
            final_transformation.transform_vector3(Camera::default_forward_vector()),
        );
        run_time.constants.model = final_transformation;
        engine.update_buffer(
            run_time.constants_handle.clone(),
            rs_foundation::cast_any_as_u8_slice(&run_time.constants),
        );
    }

    pub fn get_player_viewport(&self) -> Option<SingleThreadMutType<PlayerViewport>> {
        if !self.is_enable {
            return None;
        }
        match &self.run_time {
            Some(run_time) => Some(run_time.player_viewport.clone()),
            None => None,
        }
    }

    pub fn get_draw_objects(&self) -> Vec<&crate::drawable::EDrawObjectType> {
        self.run_time
            .as_ref()
            .map(|x| vec![&x.draw_object])
            .unwrap_or(vec![])
    }

    pub fn on_post_update_transformation(
        &mut self,
        level_physics: Option<&mut crate::content::level::Physics>,
    ) {
        let _ = level_physics;
    }

    pub fn initialize_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        let _ = collider_set;
        let _ = rigid_body_set;
    }
}
