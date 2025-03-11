use crate::camera::Camera;
use crate::components::component::Component;
use crate::content::content_file_type::EContentFileType;
use crate::directional_light::DirectionalLight;
use crate::drawable::{EDrawObjectType, PBRBindingResources};
use crate::engine::{Engine, VirtualPassHandle};
use crate::handle::TextureHandle;
use crate::input_mode::EInputMode;
use crate::misc::{Mat4Extension, FORWARD_VECTOR, UP_VECTOR};
use crate::physics_debug_render::{PhysicsDebugRender, RenderRigidBodiesBundle};
use crate::resource_manager::ResourceManager;
use crate::{build_built_in_resouce_url, BUILT_IN_RESOURCE};
use glam::Vec4Swizzles;
use rapier3d::prelude::*;
use rs_artifact::material::GroupBinding;
use rs_foundation::new::{MultipleThreadMutType, SingleThreadMutType};
use rs_render::antialias_type::{FXAAInfo, MSAAInfo};
use rs_render::command::{
    BufferCreateInfo, CreateBuffer, DrawObject, EBindingResource, ERenderTargetType,
    MultipleResolutionMeshPass, RenderCommand, ShadowMapping, TextureDescriptorCreateInfo,
    UpdateBuffer, VirtualPassSet,
};
use rs_render::constants::Constants;
use rs_render::global_uniform;
use rs_render::renderer::{EBuiltinPipelineType, EPipelineType, MaterialPipelineType};
use rs_render::vertex_data_type::mesh_vertex::MeshVertex3;
use rs_render::virtual_texture_source::TVirtualTextureSource;
use rs_render::{antialias_type::EAntialiasType, scene_viewport::SceneViewport};
use rs_render_types::MaterialOptions;
use std::collections::HashMap;
use std::ops::Deref;

bitflags::bitflags! {
    #[derive(PartialEq, Debug, Copy, Clone, Hash, Eq)]
    pub struct DebugFlags: u8 {
        const Line = 1;
        const Physics = 1 << 1 | DebugFlags::Line.bits();
    }
}

pub struct PlayerViewport {
    // pub window_id: isize,
    render_target_type: ERenderTargetType,
    pub scene_viewport: SceneViewport,
    pub width: u32,
    pub height: u32,
    pub global_sampler_handle: crate::handle::SamplerHandle,
    pub global_constants: rs_render::global_uniform::Constants,
    pub global_constants_handle: crate::handle::BufferHandle,
    pub point_lights_constants: rs_render::constants::PointLights,
    pub point_lights_constants_handle: crate::handle::BufferHandle,
    pub spot_lights_constants: rs_render::constants::SpotLights,
    pub spot_lights_constants_handle: crate::handle::BufferHandle,
    pub virtual_pass_handle: Option<VirtualPassHandle>,
    pub shadow_depth_texture_handle: Option<TextureHandle>,
    grid_draw_object: Option<DrawObject>,
    pub draw_objects: Vec<DrawObject>,
    pub particle_draw_objects: Vec<DrawObject>,
    pub camera: Camera,
    virtual_texture_source_infos: SingleThreadMutType<
        HashMap<url::Url, MultipleThreadMutType<Box<dyn TVirtualTextureSource>>>,
    >,
    pub debug_draw_objects: Vec<DrawObject>,
    physics_debug_render: Option<PhysicsDebugRender>,
    debug_flags: DebugFlags,
    _input_mode: EInputMode,
    _camera_movement_speed: f32,
    _camera_motion_speed: f32,
    pub is_use_default_input_process: bool,
    pub is_grid_visible: bool,
    settings: rs_core_minimal::settings::Settings,
    cluster_light: Option<crate::cluster_light::ClusterLight>,
    pub h_z_texture_handle: Option<TextureHandle>,
    name: String,
    _unique_id: uuid::Uuid,
}

impl PlayerViewport {
    fn new(
        render_target_type: ERenderTargetType,
        width: u32,
        height: u32,
        engine: &mut Engine,
        input_mode: EInputMode,
        is_create_grid: bool,
    ) -> PlayerViewport {
        let scene_viewport = SceneViewport::new();

        let global_constants_handle = engine.get_resource_manager().next_buffer();
        let global_constants = global_uniform::Constants::default();
        let command = RenderCommand::CreateBuffer(CreateBuffer {
            handle: *global_constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some("Global.Constants".to_string()),
                contents: rs_foundation::cast_to_raw_buffer(&vec![global_constants]).to_vec(),
                usage: wgpu::BufferUsages::all(),
            },
        });
        engine.get_render_thread_mode_mut().send_command(command);

        let point_lights_constants_handle = engine.get_resource_manager().next_buffer();
        let point_lights_constants = rs_render::constants::PointLights::default();
        let command = RenderCommand::CreateBuffer(CreateBuffer {
            handle: *point_lights_constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some("PointLights".to_string()),
                contents: rs_foundation::cast_any_as_u8_slice(&point_lights_constants).to_vec(),
                usage: wgpu::BufferUsages::all(),
            },
        });
        engine.get_render_thread_mode_mut().send_command(command);

        let spot_lights_constants_handle = engine.get_resource_manager().next_buffer();
        let spot_lights_constants = rs_render::constants::SpotLights::default();
        let command = RenderCommand::CreateBuffer(CreateBuffer {
            handle: *spot_lights_constants_handle,
            buffer_create_info: BufferCreateInfo {
                label: Some("SpotLights".to_string()),
                contents: rs_foundation::cast_any_as_u8_slice(&spot_lights_constants).to_vec(),
                usage: wgpu::BufferUsages::all(),
            },
        });
        engine.get_render_thread_mode_mut().send_command(command);

        let mut camera = Camera::default(width, height);
        camera.set_world_location(glam::vec3(0.0, 1.0, 0.0));
        let physics_debug_render = Some(PhysicsDebugRender::new());
        let grid_draw_object = if is_create_grid {
            #[cfg(feature = "editor")]
            {
                Some(engine.create_grid_draw_object(global_constants_handle.clone()))
            }
            #[cfg(not(feature = "editor"))]
            {
                None
            }
        } else {
            None
        };

        let resource_manager = ResourceManager::default();
        let shadow_depth_texture_handle =
            resource_manager.next_texture(resource_manager.get_unique_texture_url(
                &build_built_in_resouce_url("PlayerViewport.ShadowDepthTexture").unwrap(),
            ));
        engine
            .get_render_thread_mode_mut()
            .send_command(RenderCommand::CreateTexture(
                rs_render::command::CreateTexture {
                    handle: *shadow_depth_texture_handle,
                    texture_descriptor_create_info: TextureDescriptorCreateInfo {
                        label: Some(format!("PlayerViewport.ShadowDepthTexture")),
                        size: wgpu::Extent3d {
                            width: 1024,
                            height: 1024,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth32Float,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::COPY_SRC
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: None,
                    },
                    init_data: None,
                },
            ));

        let global_sampler_handle = resource_manager
            .get_builtin_resources()
            .global_sampler_handle
            .clone();
        let virtual_texture_source_infos = engine.get_virtual_texture_source_infos();
        let settings = engine.get_settings().clone();
        let unique_id = uuid::Uuid::new_v4();
        let h_z_texture_handle = if settings.render_setting.is_enable_multiple_resolution_mesh {
            Some(
                engine.create_texture(
                    &build_built_in_resouce_url(format!(
                        "PlayerViewport.MultipleResolutionMeshTexture.{}",
                        unique_id.simple()
                    ))
                    .unwrap(),
                    TextureDescriptorCreateInfo {
                        label: Some(format!("PlayerViewport.MultipleResolutionMeshTexture")),
                        size: wgpu::Extent3d {
                            width: 2560,
                            height: 1440,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Depth32Float,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::COPY_SRC
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: None,
                    },
                ),
            )
        } else {
            None
        };

        PlayerViewport {
            render_target_type,
            scene_viewport,
            width,
            height,
            global_sampler_handle,
            global_constants,
            global_constants_handle,
            virtual_pass_handle: None,
            shadow_depth_texture_handle: Some(shadow_depth_texture_handle),
            grid_draw_object,
            draw_objects: vec![],
            particle_draw_objects: vec![],
            camera,
            virtual_texture_source_infos,
            debug_draw_objects: vec![],
            physics_debug_render,
            debug_flags: DebugFlags::empty(),
            _input_mode: input_mode,
            _camera_movement_speed: 0.1,
            _camera_motion_speed: 0.1,
            is_use_default_input_process: true,
            is_grid_visible: true,
            point_lights_constants,
            point_lights_constants_handle,
            spot_lights_constants,
            spot_lights_constants_handle,
            cluster_light: None,
            settings,
            h_z_texture_handle,
            name: "PlayerViewport".to_string(),
            _unique_id: unique_id,
        }
    }

    pub fn from_window_surface(
        window_id: isize,
        width: u32,
        height: u32,
        engine: &mut Engine,
        input_mode: EInputMode,
        is_create_grid: bool,
    ) -> PlayerViewport {
        Self::new(
            ERenderTargetType::SurfaceTexture(window_id),
            width,
            height,
            engine,
            input_mode,
            is_create_grid,
        )
    }

    pub fn from_frame_buffer(
        color_texture_handle: crate::handle::TextureHandle,
        depth_texture_handle: crate::handle::TextureHandle,
        width: u32,
        height: u32,
        engine: &mut Engine,
        input_mode: EInputMode,
        is_create_grid: bool,
    ) -> PlayerViewport {
        Self::new(
            ERenderTargetType::FrameBuffer(rs_render::command::FrameBufferOptions {
                color: *color_texture_handle,
                depth: *depth_texture_handle,
            }),
            width,
            height,
            engine,
            input_mode,
            is_create_grid,
        )
    }

    pub fn enable_fxaa(&mut self, engine: &mut Engine) {
        let size = self
            .scene_viewport
            .viewport
            .as_ref()
            .map_or(glam::uvec2(self.width, self.height), |x| {
                x.rect.zw().floor().as_uvec2()
            });
        let texture_handle = engine.create_texture(
            &build_built_in_resouce_url("FXAATexture").unwrap(),
            TextureDescriptorCreateInfo {
                label: Some(format!("FXAATexture")),
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: None,
            },
        );

        self.scene_viewport.anti_type = EAntialiasType::FXAA(FXAAInfo {
            sampler: *self.global_sampler_handle,
            texture: *texture_handle,
        });
    }

    pub fn enable_msaa(&mut self, engine: &mut Engine) {
        let size = self
            .scene_viewport
            .viewport
            .as_ref()
            .map_or(glam::uvec2(self.width, self.height), |x| {
                x.rect.zw().floor().as_uvec2()
            });
        let texture_handle = engine.create_texture(
            &build_built_in_resouce_url("MSAATexture").unwrap(),
            TextureDescriptorCreateInfo {
                label: Some(format!("MSAATexture")),
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: None,
            },
        );

        let depth_texture_handle = engine.create_texture(
            &build_built_in_resouce_url("MSAADepthTexture").unwrap(),
            TextureDescriptorCreateInfo {
                label: Some(format!("MSAADepthTexture")),
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 4,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: None,
            },
        );

        self.scene_viewport.anti_type = EAntialiasType::MSAA(MSAAInfo {
            texture: *texture_handle,
            depth_texture: *depth_texture_handle,
        });
    }

    pub fn disable_antialias(&mut self) {
        self.scene_viewport.anti_type = EAntialiasType::None;
    }

    pub fn size_changed(&mut self, width: u32, height: u32, engine: &mut Engine) {
        self.width = width;
        self.height = height;
        self.camera.set_window_size(width, height);
        match self.scene_viewport.anti_type {
            EAntialiasType::None => {}
            EAntialiasType::FXAA(_) => {
                self.enable_fxaa(engine);
            }
            EAntialiasType::MSAA(_) => {
                self.enable_msaa(engine);
            }
        }
    }

    // pub fn enable_shadow(&mut self, engine: &mut Engine) {
    //     let shadow_depth_texture_handle = engine
    //         .get_resource_manager()
    //         .next_texture(build_built_in_resouce_url("ShadowDepthTexture").unwrap());
    //     engine
    //         .get_render_thread_mode_mut()
    //         .send_command(RenderCommand::CreateTexture(CreateTexture {
    //             handle: *shadow_depth_texture_handle,
    //             texture_descriptor_create_info: TextureDescriptorCreateInfo {
    //                 label: Some(format!("ShadowDepthTexture")),
    //                 size: wgpu::Extent3d {
    //                     width: 1024,
    //                     height: 1024,
    //                     depth_or_array_layers: 1,
    //                 },
    //                 mip_level_count: 1,
    //                 sample_count: 1,
    //                 dimension: wgpu::TextureDimension::D2,
    //                 format: wgpu::TextureFormat::Depth32Float,
    //                 usage: wgpu::TextureUsages::RENDER_ATTACHMENT
    //                     | wgpu::TextureUsages::COPY_SRC
    //                     | wgpu::TextureUsages::TEXTURE_BINDING,
    //                 view_formats: None,
    //             },
    //             init_data: None,
    //         }));
    //     self.shadow_depth_texture_handle = Some(shadow_depth_texture_handle);
    // }

    // fn enable_virtual_texture(
    //     &mut self,
    //     engine: &mut Engine,
    //     virtual_texture_setting: VirtualTextureSetting,
    // ) {
    //     let handle = VirtualPassHandle::new();
    //     engine
    //         .get_render_thread_mode_mut()
    //         .send_command(RenderCommand::CreateVirtualTexturePass(
    //             CreateVirtualTexturePass {
    //                 key: handle.key(),
    //                 surface_size: glam::uvec2(self.width, self.height),
    //                 settings: virtual_texture_setting,
    //             },
    //         ));
    //     self.virtual_pass_handle = Some(handle);
    // }

    pub fn update_global_constants(&mut self, engine: &mut Engine) {
        let view: glam::Mat4 = self.camera.get_view_matrix();
        let projection: glam::Mat4 = self.camera.get_projection_matrix();
        let world_location: glam::Vec3 = self.camera.get_world_location();
        self.global_constants.view = view;
        self.global_constants.projection = projection;
        self.global_constants.view_projection =
            self.global_constants.projection * self.global_constants.view;
        self.global_constants.view_position = world_location;
        self.global_constants.camera_frustum =
            self.camera.get_render_frustum_apply_tramsformation();

        let command = RenderCommand::UpdateBuffer(UpdateBuffer {
            handle: *self.global_constants_handle,
            data: rs_foundation::cast_to_raw_buffer(&vec![self.global_constants]).to_vec(),
        });
        engine.get_render_thread_mode_mut().send_command(command);

        let command = RenderCommand::UpdateBuffer(UpdateBuffer {
            handle: *self.point_lights_constants_handle,
            data: rs_foundation::cast_any_as_u8_slice(&self.point_lights_constants).to_vec(),
        });
        engine.get_render_thread_mode_mut().send_command(command);

        let command = RenderCommand::UpdateBuffer(UpdateBuffer {
            handle: *self.spot_lights_constants_handle,
            data: rs_foundation::cast_any_as_u8_slice(&self.spot_lights_constants).to_vec(),
        });
        engine.get_render_thread_mode_mut().send_command(command);
    }

    pub fn update_draw_object(&mut self, engine: &mut Engine, object: &mut EDrawObjectType) {
        match object {
            EDrawObjectType::Static(object) => {
                let resource_manager = engine.get_resource_manager();
                let settings = engine.get_settings();
                let default_textures = engine.get_default_textures();

                if let Some(texture_url) = object.diffuse_texture_url.as_ref() {
                    if let Some(_) =
                        ResourceManager::default().get_virtual_texture_by_url(texture_url)
                    {
                        let virtual_texture_source_infos =
                            self.virtual_texture_source_infos.borrow();
                        let source = virtual_texture_source_infos.get(texture_url).unwrap();
                        {
                            let source = source.lock().unwrap();
                            let max_mips = rs_core_minimal::misc::calculate_max_mips(
                                source.get_size().min_element(),
                            );
                            let max_lod = max_mips
                                - settings
                                    .render_setting
                                    .virtual_texture_setting
                                    .tile_size
                                    .ilog2()
                                - 1;
                            object.constants.diffuse_texture_max_lod = max_lod;
                            object.constants.diffuse_texture_size = source.get_size().as_vec2();
                        }
                        object.constants.is_virtual_diffuse_texture = 1;
                        object.diffuse_texture_resource =
                            EBindingResource::Texture(*default_textures.get_texture_handle());
                    } else if let Some(base_texture_handle) =
                        resource_manager.get_texture_by_url(texture_url)
                    {
                        object.constants.is_virtual_diffuse_texture = 0;
                        object.diffuse_texture_resource =
                            EBindingResource::Texture(*base_texture_handle);
                    }
                }

                engine.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
            }
            EDrawObjectType::Skin(object) => {
                let resource_manager = engine.get_resource_manager();
                let settings = engine.get_settings();
                let default_textures = engine.get_default_textures();

                if let Some(texture_url) = object.diffuse_texture_url.as_ref() {
                    if let Some(_) = resource_manager.get_virtual_texture_by_url(texture_url) {
                        let virtual_texture_source_infos =
                            self.virtual_texture_source_infos.borrow();
                        let source = virtual_texture_source_infos.get(texture_url).unwrap();
                        {
                            let source = source.lock().unwrap();
                            let max_mips = rs_core_minimal::misc::calculate_max_mips(
                                source.get_size().min_element(),
                            );
                            let max_lod = max_mips
                                - settings
                                    .render_setting
                                    .virtual_texture_setting
                                    .tile_size
                                    .ilog2()
                                - 1;
                            object.constants.diffuse_texture_max_lod = max_lod;
                            object.constants.diffuse_texture_size = source.get_size().as_vec2();
                        }
                        object.constants.is_virtual_diffuse_texture = 1;
                        object.diffuse_texture_resource =
                            EBindingResource::Texture(*default_textures.get_texture_handle());
                    } else if let Some(base_texture_handle) =
                        resource_manager.get_texture_by_url(texture_url)
                    {
                        object.constants.is_virtual_diffuse_texture = 0;
                        object.diffuse_texture_resource =
                            EBindingResource::Texture(*base_texture_handle);
                    }
                }

                engine.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
            }
            EDrawObjectType::SkinMaterial(object) => {
                let settings = engine.get_settings();

                let material_info = object.material.borrow().get_material_info().clone();
                let map_textures = &material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap()
                    .map_textures;
                for virtual_texture_url in &material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap()
                    .virtual_textures
                {
                    let virtual_texture_source_infos = self.virtual_texture_source_infos.borrow();
                    let source = virtual_texture_source_infos
                        .get(virtual_texture_url)
                        .unwrap();
                    {
                        let source = source.lock().unwrap();
                        let max_mips = rs_core_minimal::misc::calculate_max_mips(
                            source.get_size().min_element(),
                        );
                        let max_lod = max_mips
                            - settings
                                .render_setting
                                .virtual_texture_setting
                                .tile_size
                                .ilog2()
                            - 1;
                        object.virtual_texture_constants.virtual_texture_max_lod = max_lod;
                        object.virtual_texture_constants.virtual_texture_size =
                            source.get_size().as_vec2();
                    }
                }
                engine.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                engine.update_buffer(
                    object.skin_constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.skin_constants),
                );
                engine.update_buffer(
                    object.virtual_texture_constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.virtual_texture_constants),
                );

                let mut binding_resources: Vec<EBindingResource> =
                    Vec::with_capacity(map_textures.len());
                for map_texture in map_textures {
                    let resource_manager = engine.get_resource_manager();

                    if let Some(handle) =
                        resource_manager.get_texture_by_url(&map_texture.texture_url)
                    {
                        binding_resources.push(EBindingResource::Texture(*handle));
                    } else {
                        log::trace!("Can not find {}", map_texture.texture_url.to_string());
                    }
                }
                assert_eq!(binding_resources.len(), map_textures.len());
                object.user_textures_resources = binding_resources;
                let resource_manager = engine.get_resource_manager();

                let ibl_textures = resource_manager.get_ibl_textures();
                let Some((_, ibl_textures)) = ibl_textures.iter().find(|x| {
                    let url = x.0;
                    url.scheme() != BUILT_IN_RESOURCE
                }) else {
                    return;
                };

                let PBRBindingResources {
                    brdflut_texture_resource,
                    pre_filter_cube_map_texture_resource,
                    irradiance_texture_resource,
                    shadow_map_texture_resource,
                    point_lights_constants_resource,
                    material_parameters_collection_resources,
                    ..
                } = &mut object.pbr_binding_resources;

                *brdflut_texture_resource = EBindingResource::Texture(*ibl_textures.brdflut);
                *pre_filter_cube_map_texture_resource =
                    EBindingResource::Texture(*ibl_textures.pre_filter_cube_map);
                *irradiance_texture_resource = EBindingResource::Texture(*ibl_textures.irradiance);
                *shadow_map_texture_resource = EBindingResource::Texture(
                    *self
                        .shadow_depth_texture_handle
                        .clone()
                        .unwrap_or(engine.get_default_textures().get_depth_texture_handle()),
                );
                *point_lights_constants_resource =
                    EBindingResource::Constants(*self.point_lights_constants_handle);

                let material_info = material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap();
                material_parameters_collection_resources.clear();
                for material_paramenters_collection_binding in
                    &material_info.material_paramenters_collection_bindings
                {
                    let material_paramenters_collection_url =
                        &material_paramenters_collection_binding
                            .material_paramenters_collection_url;
                    let content = engine
                        .content_files
                        .get(material_paramenters_collection_url)
                        .unwrap();
                    let buffer_handle = match content {
                        EContentFileType::MaterialParamentersCollection(rc) => {
                            rc.borrow().get_buffer_handle().unwrap()
                        }
                        _ => {
                            panic!()
                        }
                    };
                    material_parameters_collection_resources.insert(
                        GroupBinding {
                            group: material_paramenters_collection_binding.group,
                            binding: material_paramenters_collection_binding.binding,
                        },
                        EBindingResource::Constants(*buffer_handle),
                    );
                }

                if let Some(cluster_light) = &self.cluster_light {
                    object.pbr_binding_resources.point_lights_constants_resource =
                        EBindingResource::Constants(*cluster_light.point_lights_handle);
                    object.pbr_binding_resources.cluster_light = Some(EBindingResource::Constants(
                        *cluster_light.cluster_light_handle,
                    ));
                    object.pbr_binding_resources.cluster_light_index = Some(
                        EBindingResource::Constants(*cluster_light.cluster_light_index_handle),
                    );
                }
            }
            EDrawObjectType::StaticMeshMaterial(object) => {
                let settings = engine.get_settings();

                let material_info = object.material.borrow().get_material_info().clone();
                let map_textures = &material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap()
                    .map_textures;
                for virtual_texture_url in &material_info
                    .get(&MaterialOptions { is_skin: true })
                    .unwrap()
                    .virtual_textures
                {
                    let virtual_texture_source_infos = self.virtual_texture_source_infos.borrow();
                    let source = virtual_texture_source_infos
                        .get(virtual_texture_url)
                        .unwrap();
                    {
                        let source = source.lock().unwrap();
                        let max_mips = rs_core_minimal::misc::calculate_max_mips(
                            source.get_size().min_element(),
                        );
                        let max_lod = max_mips
                            - settings
                                .render_setting
                                .virtual_texture_setting
                                .tile_size
                                .ilog2()
                            - 1;
                        object.virtual_texture_constants.virtual_texture_max_lod = max_lod;
                        object.virtual_texture_constants.virtual_texture_size =
                            source.get_size().as_vec2();
                    }
                }

                engine.update_buffer(
                    object.constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.constants),
                );
                engine.update_buffer(
                    object.virtual_texture_constants_buffer_handle.clone(),
                    rs_foundation::cast_any_as_u8_slice(&object.virtual_texture_constants),
                );

                let mut binding_resources: Vec<EBindingResource> =
                    Vec::with_capacity(map_textures.len());
                for map_texture in map_textures {
                    let resource_manager = engine.get_resource_manager();
                    if let Some(handle) =
                        resource_manager.get_texture_by_url(&map_texture.texture_url)
                    {
                        binding_resources.push(EBindingResource::Texture(*handle));
                    } else {
                        log::trace!("Can not find {}", map_texture.texture_url.to_string());
                    }
                }
                assert_eq!(binding_resources.len(), map_textures.len());
                object.user_textures_resources = binding_resources;

                let ibl_textures = {
                    let resource_manager = engine.get_resource_manager();
                    resource_manager.get_ibl_textures()
                };
                let Some((_, ibl_textures)) = ibl_textures.iter().find(|x| {
                    let url = x.0;
                    url.scheme() != BUILT_IN_RESOURCE
                }) else {
                    return;
                };

                let PBRBindingResources {
                    global_constants_resource,
                    brdflut_texture_resource,
                    pre_filter_cube_map_texture_resource,
                    irradiance_texture_resource,
                    shadow_map_texture_resource,
                    point_lights_constants_resource,
                    material_parameters_collection_resources,
                    ..
                } = &mut object.pbr_binding_resources;

                *global_constants_resource =
                    EBindingResource::Constants(*self.global_constants_handle);
                *brdflut_texture_resource = EBindingResource::Texture(*ibl_textures.brdflut);
                *pre_filter_cube_map_texture_resource =
                    EBindingResource::Texture(*ibl_textures.pre_filter_cube_map);
                *irradiance_texture_resource = EBindingResource::Texture(*ibl_textures.irradiance);
                *shadow_map_texture_resource = EBindingResource::Texture(
                    *self
                        .shadow_depth_texture_handle
                        .clone()
                        .unwrap_or(engine.get_default_textures().get_depth_texture_handle()),
                );
                *point_lights_constants_resource =
                    EBindingResource::Constants(*self.point_lights_constants_handle);

                let material_info = material_info
                    .get(&MaterialOptions { is_skin: false })
                    .unwrap();
                material_parameters_collection_resources.clear();
                for material_paramenters_collection_binding in
                    &material_info.material_paramenters_collection_bindings
                {
                    let material_paramenters_collection_url =
                        &material_paramenters_collection_binding
                            .material_paramenters_collection_url;
                    let content = engine
                        .content_files
                        .get(material_paramenters_collection_url)
                        .unwrap();
                    let buffer_handle = match content {
                        EContentFileType::MaterialParamentersCollection(rc) => {
                            rc.borrow().get_buffer_handle().unwrap()
                        }
                        _ => {
                            panic!()
                        }
                    };
                    material_parameters_collection_resources.insert(
                        GroupBinding {
                            group: material_paramenters_collection_binding.group,
                            binding: material_paramenters_collection_binding.binding,
                        },
                        EBindingResource::Constants(*buffer_handle),
                    );
                }

                if let Some(cluster_light) = &self.cluster_light {
                    object.pbr_binding_resources.point_lights_constants_resource =
                        EBindingResource::Constants(*cluster_light.point_lights_handle);
                    object.pbr_binding_resources.cluster_light = Some(EBindingResource::Constants(
                        *cluster_light.cluster_light_handle,
                    ));
                    object.pbr_binding_resources.cluster_light_index = Some(
                        EBindingResource::Constants(*cluster_light.cluster_light_index_handle),
                    );
                }
            }
            EDrawObjectType::Custom(_) => {}
        }
    }

    fn load_group_binding_to_resource(
        pbrbinding_resources: PBRBindingResources,
        material_info: &rs_artifact::material::MaterialInfo,
    ) -> Option<Vec<(GroupBinding, EBindingResource)>> {
        let mut group_binding_to_resource: Vec<(GroupBinding, EBindingResource)> = vec![];
        let PBRBindingResources {
            global_constants_resource,
            base_color_sampler_resource,
            physical_texture_resource,
            page_table_texture_resource,
            brdflut_texture_resource,
            pre_filter_cube_map_texture_resource,
            irradiance_texture_resource,
            shadow_map_texture_resource,
            constants_resource,
            virtual_texture_constants_resource,
            point_lights_constants_resource,
            spot_lights_constants_resource,
            material_parameters_collection_resources,
            cluster_light,
            cluster_light_index,
        } = pbrbinding_resources;
        if let Some(group_binding) = &material_info.global_constants_binding {
            group_binding_to_resource.push((*group_binding, global_constants_resource.clone()));
        }
        if let Some(group_binding) = &material_info.base_color_sampler_binding {
            group_binding_to_resource.push((*group_binding, base_color_sampler_resource));
        }
        if let Some(group_binding) = &material_info.physical_texture_binding {
            group_binding_to_resource.push((*group_binding, physical_texture_resource));
        }
        if let Some(group_binding) = &material_info.page_table_texture_binding {
            group_binding_to_resource.push((*group_binding, page_table_texture_resource));
        }
        if let Some(group_binding) = &material_info.brdflut_texture_binding {
            group_binding_to_resource.push((*group_binding, brdflut_texture_resource));
        }
        if let Some(group_binding) = &material_info.pre_filter_cube_map_texture_binding {
            group_binding_to_resource.push((*group_binding, pre_filter_cube_map_texture_resource));
        }
        if let Some(group_binding) = &material_info.irradiance_texture_binding {
            group_binding_to_resource.push((*group_binding, irradiance_texture_resource));
        }
        if let Some(group_binding) = &material_info.shadow_map_binding {
            group_binding_to_resource.push((*group_binding, shadow_map_texture_resource));
        }
        if let Some(group_binding) = &material_info.constants_binding {
            group_binding_to_resource.push((*group_binding, constants_resource.clone()));
        }
        if let Some(group_binding) = &material_info.virtual_texture_constants_binding {
            group_binding_to_resource.push((*group_binding, virtual_texture_constants_resource));
        }
        if let Some(group_binding) = &material_info.point_lights_binding {
            group_binding_to_resource.push((*group_binding, point_lights_constants_resource));
        }
        if let Some(group_binding) = &material_info.spot_lights_binding {
            group_binding_to_resource.push((*group_binding, spot_lights_constants_resource));
        }
        for (group_binding, material_parameters_collection_resource) in
            material_parameters_collection_resources
        {
            group_binding_to_resource
                .push((group_binding, material_parameters_collection_resource));
        }

        group_binding_to_resource.push((*&material_info.cluster_light_binding?, cluster_light?));

        group_binding_to_resource.push((
            *&material_info.cluster_light_index_binding?,
            cluster_light_index?,
        ));

        Some(group_binding_to_resource)
    }

    fn make_binding_resources(
        mut group_binding_to_resource: Vec<(GroupBinding, EBindingResource)>,
    ) -> Vec<Vec<EBindingResource>> {
        let mut binding_resources: Vec<Vec<EBindingResource>> = vec![];
        group_binding_to_resource.sort_unstable_by_key(|item| (item.0.group, item.0.binding));
        for (group_binding, binding_resource) in group_binding_to_resource {
            binding_resources.resize(group_binding.group + 1, vec![]);
            binding_resources[group_binding.group].push(binding_resource);
        }
        binding_resources
    }

    pub fn to_render_draw_object(
        draw_object: &EDrawObjectType,
        is_enable_shadow: bool,
    ) -> crate::error::Result<DrawObject> {
        match draw_object {
            EDrawObjectType::Static(static_objcet) => {
                let static_objcet = static_objcet.clone();
                let draw_object = DrawObject::new(
                    static_objcet.id,
                    static_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                    static_objcet.vertex_count,
                    EPipelineType::Builtin(EBuiltinPipelineType::StaticMeshPhong),
                    static_objcet.index_buffer.clone().map(|x| *x),
                    static_objcet.index_count,
                    vec![
                        vec![
                            static_objcet.global_constants_resource,
                            static_objcet.base_color_sampler_resource,
                            static_objcet.physical_texture_resource,
                            static_objcet.page_table_texture_resource,
                        ],
                        vec![
                            static_objcet.diffuse_texture_resource,
                            static_objcet.specular_texture_resource,
                        ],
                        vec![static_objcet.constants_resource],
                    ],
                );

                Ok(draw_object)
            }
            EDrawObjectType::Skin(skin_objcet) => {
                let skin_objcet = skin_objcet.clone();

                let draw_object = DrawObject::new(
                    skin_objcet.id,
                    skin_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                    skin_objcet.vertex_count,
                    EPipelineType::Builtin(EBuiltinPipelineType::SkinMeshPhong),
                    skin_objcet.index_buffer.clone().map(|x| *x),
                    skin_objcet.index_count,
                    vec![
                        vec![
                            skin_objcet.global_constants_resource,
                            skin_objcet.base_color_sampler_resource,
                            skin_objcet.physical_texture_resource,
                            skin_objcet.page_table_texture_resource,
                        ],
                        vec![
                            skin_objcet.diffuse_texture_resource,
                            skin_objcet.specular_texture_resource,
                        ],
                        vec![skin_objcet.constants_resource],
                    ],
                );
                Ok(draw_object)
            }
            EDrawObjectType::SkinMaterial(skin_objcet) => {
                let skin_objcet = skin_objcet.clone();
                let material = skin_objcet.material.borrow();
                let pipeline_handle = material
                    .get_pipeline_handle()
                    .ok_or(crate::error::Error::Other(None))?;
                let material_info = material
                    .get_material_info()
                    .get(&MaterialOptions { is_skin: true })
                    .ok_or(crate::error::Error::Other(None))?;

                let mut group_binding_to_resource = Self::load_group_binding_to_resource(
                    skin_objcet.pbr_binding_resources.clone(),
                    material_info,
                )
                .ok_or(crate::error::Error::NullReference(Some(
                    "Fail to load group binding to resource".to_string(),
                )))?;

                if let Some(group_binding) = &material_info.skin_constants_binding {
                    group_binding_to_resource
                        .push((*group_binding, skin_objcet.skin_constants_resource.clone()));
                }

                let mut binding_resources = Self::make_binding_resources(group_binding_to_resource);
                binding_resources[0].append(&mut skin_objcet.user_textures_resources.clone());
                let PBRBindingResources {
                    global_constants_resource,
                    constants_resource,
                    ..
                } = skin_objcet.pbr_binding_resources;

                let mut draw_object = DrawObject::new(
                    skin_objcet.id,
                    skin_objcet.vertex_buffers.iter().map(|x| **x).collect(),
                    skin_objcet.vertex_count,
                    EPipelineType::Material(MaterialPipelineType {
                        handle: *pipeline_handle,
                        options: MaterialOptions { is_skin: true },
                    }),
                    skin_objcet.index_buffer.clone().map(|x| *x),
                    skin_objcet.index_count,
                    binding_resources,
                );
                draw_object.debug_group_label = skin_objcet.debug_group_label;
                draw_object.virtual_pass_set = Some(VirtualPassSet {
                    vertex_buffers: vec![
                        *skin_objcet.vertex_buffers[0],
                        *skin_objcet.vertex_buffers[2],
                    ],
                    binding_resources: vec![
                        vec![global_constants_resource.clone()],
                        vec![
                            constants_resource.clone(),
                            skin_objcet.skin_constants_resource.clone(),
                        ],
                    ],
                });
                if is_enable_shadow {
                    draw_object.shadow_mapping = Some(ShadowMapping {
                        vertex_buffers: vec![
                            *skin_objcet.vertex_buffers[0],
                            *skin_objcet.vertex_buffers[2],
                        ],
                        binding_resources: vec![vec![
                            global_constants_resource.clone(),
                            constants_resource.clone(),
                            skin_objcet.skin_constants_resource.clone(),
                        ]],
                        is_skin: true,
                    });
                }
                Ok(draw_object)
            }
            EDrawObjectType::StaticMeshMaterial(static_mesh_draw_objcet) => {
                let static_mesh_draw_objcet = static_mesh_draw_objcet.clone();
                let material = static_mesh_draw_objcet.material.borrow();
                let pipeline_handle = material
                    .get_pipeline_handle()
                    .ok_or(crate::error::Error::Other(None))?;
                let material_info = material
                    .get_material_info()
                    .get(&MaterialOptions { is_skin: false })
                    .ok_or(crate::error::Error::Other(None))?;

                let group_binding_to_resource = Self::load_group_binding_to_resource(
                    static_mesh_draw_objcet.pbr_binding_resources.clone(),
                    material_info,
                )
                .ok_or(crate::error::Error::NullReference(Some(
                    "Fail to load group binding to resource".to_string(),
                )))?;

                let mut binding_resources = Self::make_binding_resources(group_binding_to_resource);
                binding_resources[0]
                    .append(&mut static_mesh_draw_objcet.user_textures_resources.clone());

                let PBRBindingResources {
                    global_constants_resource,
                    constants_resource,
                    ..
                } = static_mesh_draw_objcet.pbr_binding_resources;

                let mut draw_object = DrawObject::new(
                    static_mesh_draw_objcet.id,
                    static_mesh_draw_objcet
                        .vertex_buffers
                        .iter()
                        .map(|x| **x)
                        .collect(),
                    static_mesh_draw_objcet.vertex_count,
                    EPipelineType::Material(MaterialPipelineType {
                        handle: *pipeline_handle,
                        options: MaterialOptions { is_skin: false },
                    }),
                    static_mesh_draw_objcet.index_buffer.clone().map(|x| *x),
                    static_mesh_draw_objcet.index_count,
                    binding_resources,
                );
                draw_object.debug_group_label = static_mesh_draw_objcet.debug_group_label;
                draw_object.virtual_pass_set = Some(VirtualPassSet {
                    vertex_buffers: vec![*static_mesh_draw_objcet.vertex_buffers[0]],
                    binding_resources: vec![
                        vec![global_constants_resource.clone()],
                        vec![constants_resource.clone()],
                    ],
                });
                if is_enable_shadow {
                    draw_object.shadow_mapping = Some(ShadowMapping {
                        vertex_buffers: vec![*static_mesh_draw_objcet.vertex_buffers[0]],
                        binding_resources: vec![vec![
                            global_constants_resource.clone(),
                            constants_resource.clone(),
                        ]],
                        is_skin: false,
                    });
                }
                if let Some(handle) =
                    static_mesh_draw_objcet.multiple_resolution_mesh_pass_resource_handle
                {
                    draw_object.multiple_resolution_mesh_pass = Some(MultipleResolutionMeshPass {
                        resource_handle: *handle,
                        binding_resources: vec![vec![
                            global_constants_resource.clone(),
                            constants_resource.clone(),
                        ]],
                        transformation: static_mesh_draw_objcet.constants.model,
                    });
                }
                Ok(draw_object)
            }
            EDrawObjectType::Custom(custom_objcet) => Ok(custom_objcet.draw_object.clone()),
        }
    }

    pub fn push_to_draw_list(&mut self, draw_object: &EDrawObjectType) {
        match Self::to_render_draw_object(draw_object, self.shadow_depth_texture_handle.is_some()) {
            Ok(draw_object) => {
                self.draw_objects.push(draw_object);
            }
            Err(err) => {
                log::warn!("{}", err);
            }
        }
    }

    pub fn append_to_draw_list(&mut self, draw_objects: &[EDrawObjectType]) {
        let mut draw_objects = draw_objects
            .iter()
            .map(|x| {
                match Self::to_render_draw_object(x, self.shadow_depth_texture_handle.is_some()) {
                    Ok(ret) => Ok(ret),
                    Err(err) => {
                        // log::warn!("{}", err);
                        Err(err)
                    }
                }
            })
            .flatten()
            .collect();
        self.draw_objects.append(&mut draw_objects);
    }

    pub fn draw_debug_line(
        &mut self,
        engine: &mut Engine,
        start: glam::Vec3,
        end: glam::Vec3,
        color: glam::Vec4,
    ) {
        if !self.debug_flags.contains(DebugFlags::Line) {
            return;
        }
        let draw_object = Self::create_draw_debug_line(
            engine,
            self.global_constants_handle.clone(),
            start,
            end,
            color,
        );
        self.debug_draw_objects.push(draw_object);
    }

    pub fn create_draw_debug_line(
        engine: &mut Engine,
        global_constants_handle: crate::handle::BufferHandle,
        start: glam::Vec3,
        end: glam::Vec3,
        color: glam::Vec4,
    ) -> DrawObject {
        let contents = vec![
            MeshVertex3 {
                position: start,
                vertex_color: color,
            },
            MeshVertex3 {
                position: end,
                vertex_color: color,
            },
        ];
        let vertex_handle =
            engine.create_vertex_buffer(&contents, Some(String::from("DebugLine.Vertex")));
        let contents = Constants::default();
        let constants_handle = engine
            .create_constants_buffer(&vec![contents], Some(String::from("DebugLine.Constants")));
        let draw_object = DrawObject::new(
            0,
            vec![*vertex_handle],
            2,
            EPipelineType::Builtin(EBuiltinPipelineType::Primitive(None)),
            None,
            None,
            vec![
                vec![EBindingResource::Constants(*global_constants_handle)],
                vec![EBindingResource::Constants(*constants_handle)],
            ],
        );
        draw_object
    }

    pub fn draw_debug_lines(&mut self, engine: &mut Engine, bundles: &[RenderRigidBodiesBundle]) {
        if !self.debug_flags.contains(DebugFlags::Line) {
            return;
        }
        let contents: Vec<MeshVertex3> = bundles
            .iter()
            .flat_map(|x| {
                vec![
                    MeshVertex3 {
                        position: x.start,
                        vertex_color: x.color,
                    },
                    MeshVertex3 {
                        position: x.end,
                        vertex_color: x.color,
                    },
                ]
            })
            .collect();
        let vertex_count = contents.len();
        let vertex_handle =
            engine.create_vertex_buffer(&contents, Some(String::from("DebugLine.Vertex")));
        let contents = Constants::default();
        let constants_handle = engine
            .create_constants_buffer(&vec![contents], Some(String::from("DebugLine.Constants")));
        let draw_object = DrawObject::new(
            0,
            vec![*vertex_handle],
            vertex_count as u32,
            EPipelineType::Builtin(EBuiltinPipelineType::Primitive(None)),
            None,
            None,
            vec![
                vec![EBindingResource::Constants(*self.global_constants_handle)],
                vec![EBindingResource::Constants(*constants_handle)],
            ],
        );
        self.debug_draw_objects.push(draw_object);
    }

    pub fn physics_debug(
        &mut self,
        engine: &mut Engine,
        bodies: &RigidBodySet,
        colliders: &ColliderSet,
    ) {
        if !self.debug_flags.contains(DebugFlags::Physics) {
            return;
        }
        let Some(physics_debug_render) = &mut self.physics_debug_render else {
            return;
        };
        let mut bundles = vec![];
        let mut rigid_bodies_bundle = physics_debug_render.render_rigid_bodies(bodies);
        bundles.append(&mut rigid_bodies_bundle);
        let mut colliders_bundle = physics_debug_render.render_colliders(bodies, colliders);
        bundles.append(&mut colliders_bundle);
        self.draw_debug_lines(engine, &bundles);
    }

    pub fn set_debug_flags(&mut self, debug_flags: DebugFlags) {
        self.debug_flags = debug_flags;
    }

    #[cfg(not(target_os = "android"))]
    pub fn on_device_event(&mut self, device_event: &winit::event::DeviceEvent) {
        use crate::camera_input_event_handle::{
            CameraInputEventHandle, DefaultCameraInputEventHandle,
        };
        match device_event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                DefaultCameraInputEventHandle::mouse_motion_handle(
                    &mut self.camera,
                    *delta,
                    self._input_mode,
                    self._camera_motion_speed,
                );
            }
            _ => {}
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn on_window_input(&mut self, ty: crate::input_type::EInputType) {
        use crate::{
            camera_input_event_handle::{CameraInputEventHandle, DefaultCameraInputEventHandle},
            input_type::EInputType,
        };
        use winit::event::MouseScrollDelta;
        if !self.is_use_default_input_process {
            return;
        }
        match ty {
            EInputType::MouseWheel(delta) => {
                //
                match delta {
                    MouseScrollDelta::LineDelta(_, up) => {
                        self._camera_movement_speed += up * 0.005;
                        self._camera_movement_speed = self._camera_movement_speed.max(0.0);
                    }
                    MouseScrollDelta::PixelDelta(_) => todo!(),
                }
            }

            EInputType::MouseInput(_, _) => {}
            EInputType::KeyboardInput(virtual_key_code_states) => {
                for (virtual_key_code, element_state) in virtual_key_code_states {
                    DefaultCameraInputEventHandle::keyboard_input_handle(
                        &mut self.camera,
                        virtual_key_code,
                        element_state,
                        self._input_mode,
                        self._camera_movement_speed,
                    );
                }
            }
            _ => {}
        }
    }

    pub fn on_antialias_type_changed(
        &mut self,
        antialias_type: rs_core_minimal::settings::EAntialiasType,
        engine: &mut Engine,
    ) {
        match antialias_type {
            rs_core_minimal::settings::EAntialiasType::None => {
                self.disable_antialias();
            }
            rs_core_minimal::settings::EAntialiasType::FXAA => {
                self.enable_fxaa(engine);
            }
            rs_core_minimal::settings::EAntialiasType::MSAA => {
                self.enable_msaa(engine);
            }
        }
    }

    pub fn update_light(&mut self, light: &mut crate::directional_light::DirectionalLight) {
        self.global_constants.light_space_matrix = light.get_light_space_matrix();
    }

    fn update_light_with_view_projection(
        &mut self,
        view_matrix: glam::Mat4,
        projection_matrix: glam::Mat4,
    ) {
        self.global_constants.light_space_matrix = projection_matrix * view_matrix;
    }

    pub fn update_light_concentrate_scene(
        &mut self,
        offset_look_and_projection_matrix: (f32, glam::Vec3, glam::Mat4),
        directional_lights: Vec<SingleThreadMutType<DirectionalLight>>,
    ) {
        let (offset, center, projection_matrix) = offset_look_and_projection_matrix;
        for directional_light in directional_lights {
            let directional_light = directional_light.borrow();
            let look_to = directional_light
                .get_transformation()
                .transform_vector3(FORWARD_VECTOR);
            let eye = center - look_to * offset;
            let view_matrix = glam::Mat4::look_to_rh(
                eye,
                directional_light
                    .get_transformation()
                    .transform_vector3(FORWARD_VECTOR),
                UP_VECTOR,
            );
            self.update_light_with_view_projection(view_matrix, projection_matrix);
        }
    }

    pub fn update_point_lights(
        &mut self,
        engine: &mut Engine,
        lights: Vec<
            SingleThreadMutType<crate::components::point_light_component::PointLightComponent>,
        >,
    ) {
        rs_core_minimal::vec_ref!(lights_ref, lights);
        self.cluster_light = crate::cluster_light::ClusterLight::new(
            engine,
            &self.camera,
            lights_ref,
            self.settings
                .render_setting
                .is_enable_light_culling_acceleration,
        )
        .ok();

        self.point_lights_constants.available = lights.len() as u32;
        let num = lights.len().min(self.point_lights_constants.lights.len());
        for i in 0..num {
            let light = lights[i].borrow();
            let point_light_attributes = light.point_light;
            self.point_lights_constants.lights[i].ambient = point_light_attributes.ambient;
            self.point_lights_constants.lights[i].diffuse = point_light_attributes.diffuse;
            self.point_lights_constants.lights[i].specular = point_light_attributes.specular;
            self.point_lights_constants.lights[i].linear = point_light_attributes.linear;
            self.point_lights_constants.lights[i].constant = point_light_attributes.constant;
            self.point_lights_constants.lights[i].quadratic = point_light_attributes.quadratic;
            self.point_lights_constants.lights[i].position = light
                .get_final_transformation()
                .to_scale_rotation_translation()
                .2;
        }
    }

    pub fn update_spot_lights(
        &mut self,
        lights: Vec<
            SingleThreadMutType<crate::components::spot_light_component::SpotLightComponent>,
        >,
    ) {
        self.spot_lights_constants.available = lights.len() as u32;
        let num = lights.len().min(self.spot_lights_constants.lights.len());
        for i in 0..num {
            let light = lights[i].borrow();
            let spot_light_attributes = &light.spot_light;
            self.spot_lights_constants.lights[i].light.ambient =
                spot_light_attributes.light.ambient;
            self.spot_lights_constants.lights[i].light.diffuse =
                spot_light_attributes.light.diffuse;
            self.spot_lights_constants.lights[i].light.specular =
                spot_light_attributes.light.specular;
            self.spot_lights_constants.lights[i].light.linear = spot_light_attributes.light.linear;
            self.spot_lights_constants.lights[i].light.constant =
                spot_light_attributes.light.constant;
            self.spot_lights_constants.lights[i].light.quadratic =
                spot_light_attributes.light.quadratic;
            self.spot_lights_constants.lights[i].cut_off = spot_light_attributes.cut_off;
            self.spot_lights_constants.lights[i].outer_cut_off =
                spot_light_attributes.outer_cut_off;
            self.spot_lights_constants.lights[i].light.position = light
                .get_final_transformation()
                .to_scale_rotation_translation()
                .2;
            self.spot_lights_constants.lights[i].direction =
                light.get_final_transformation().get_forward_vector();
        }
    }

    pub fn set_debug_shading(&mut self, ty: global_uniform::EDebugShadingType) {
        self.global_constants.set_shading_type(ty);
    }

    pub fn set_input_mode(&mut self, input_mode: EInputMode) {
        self._input_mode = input_mode;
    }

    pub fn get_render_target_type(&self) -> &ERenderTargetType {
        &self.render_target_type
    }

    pub fn set_grid_visible(&mut self, is_visible: bool) {
        self.is_grid_visible = is_visible;
    }

    pub fn toggle_grid_visible(&mut self) {
        self.is_grid_visible = !self.is_grid_visible;
    }

    pub fn get_grid_draw_object(&self) -> Option<&DrawObject> {
        if self.is_grid_visible {
            self.grid_draw_object.as_ref()
        } else {
            None
        }
    }

    pub fn get_scene_light(&self) -> Option<&rs_render::command::SceneLight> {
        match &self.cluster_light {
            Some(cluster_light) => cluster_light.scene_points_lights.as_ref(),
            None => None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}
