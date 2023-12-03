use super::base_render_pipeline::{BaseRenderPipeline, VertexBufferType};
use crate::bind_group_layout_entry_hook::EBindGroupLayoutEntryHookType;
use crate::brigde_data::gpu_vertex_buffer::GpuVertexBufferImp;
use crate::brigde_data::mesh_vertex::MeshVertex;
use crate::light::{DirectionalLight, PointLight, SpotLight};
use crate::util;
use std::collections::HashMap;
use type_layout::TypeLayout;
use wgpu::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Constants {
    pub directional_light: DirectionalLight,
    pub point_light: PointLight,
    pub spot_light: SpotLight,
    pub model: glam::Mat4,
    pub view: glam::Mat4,
    pub projection: glam::Mat4,
    pub view_position: glam::Vec3,
    pub roughness_factor: f32,
    pub metalness_factor: f32,
    pub base_layer_ior: f32,
    pub clear_coat: f32,                
    pub clear_coat_roughness: f32,   
    // _padding3: [u32; 3],
}

impl Constants {
    pub fn new(
        directional_light: DirectionalLight,
        point_light: PointLight,
        spot_light: SpotLight,
        model: glam::Mat4,
        view: glam::Mat4,
        projection: glam::Mat4,
        view_position: glam::Vec3,
        roughness_factor: f32,
        metalness_factor: f32,
    ) -> Constants {
        Constants {
            directional_light,
            point_light,
            spot_light,
            model,
            view,
            projection,
            view_position,
            roughness_factor,
            metalness_factor,
            base_layer_ior: 2.97,
            clear_coat: 0.0,
            clear_coat_roughness: 0.0,
            // _padding3: [0, 0, 0],
        }
    }
}

impl Constants {
    pub fn default() -> Constants {
        Constants {
            directional_light: DirectionalLight::default(),
            point_light: PointLight::default(),
            spot_light: SpotLight::default(),
            model: glam::Mat4::IDENTITY,
            view: glam::Mat4::IDENTITY,
            projection: glam::Mat4::IDENTITY,
            view_position: glam::Vec3::ZERO,
            roughness_factor: 0.0,
            metalness_factor: 0.0,
            base_layer_ior: 0.1,
            clear_coat: 0.0,
            clear_coat_roughness: 0.0,
            // _padding3: [0, 0, 0],
        }
    }
}

pub struct Material {
    pub albedo_texture_view: wgpu::TextureView,
    pub normal_texture_view: wgpu::TextureView,
    pub metallic_texture_view: wgpu::TextureView,
    pub roughness_texture_view: wgpu::TextureView,
    pub brdflut_texture_view: wgpu::TextureView,
    pub pre_filter_cube_map_texture_view: wgpu::TextureView,
    pub irradiance_texture_view: wgpu::TextureView,
}

pub struct NextPBRPipeline {
    base_render_pipeline: BaseRenderPipeline,
    base_color_sampler: Sampler,
    base_color_sampler_non_filtering: Sampler,
}

impl NextPBRPipeline {
    pub fn new(
        device: &Device,
        texture_format: &TextureFormat,
        is_noninterleaved: bool,
    ) -> NextPBRPipeline {
        let base_render_pipeline = BaseRenderPipeline::new(
            device,
            "pbr.wgsl",
            texture_format,
            Some(DepthStencilState {
                depth_compare: CompareFunction::Less,
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            None,
            None,
            Some(PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                ..Default::default()
            }),
            if is_noninterleaved {
                VertexBufferType::Noninterleaved
            } else {
                VertexBufferType::Interleaved(MeshVertex::type_layout())
            },
            None,
        );

        let base_color_sampler = device.create_sampler(&{
            let mut des = SamplerDescriptor::default();
            des.mipmap_filter = FilterMode::Linear;
            des
        });
        let base_color_sampler_non_filtering = device.create_sampler(&SamplerDescriptor::default());

        NextPBRPipeline {
            base_render_pipeline,
            base_color_sampler,
            base_color_sampler_non_filtering,
        }
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        constants: &Constants,
        mesh_buffers: &[GpuVertexBufferImp],
        material: &Material,
    ) {
        let uniform_buf = util::create_gpu_uniform_buffer_from(device, constants, None);

        self.base_render_pipeline.draw_resources2(
            device,
            queue,
            vec![
                vec![uniform_buf.as_entire_binding()],
                vec![
                    BindingResource::TextureView(&material.albedo_texture_view),
                    BindingResource::TextureView(&material.normal_texture_view),
                    BindingResource::TextureView(&material.metallic_texture_view),
                    BindingResource::TextureView(&material.roughness_texture_view),
                    BindingResource::TextureView(&material.brdflut_texture_view),
                    BindingResource::TextureView(&material.pre_filter_cube_map_texture_view),
                    BindingResource::TextureView(&material.irradiance_texture_view),
                ],
                vec![
                    BindingResource::Sampler(&self.base_color_sampler),
                    BindingResource::Sampler(&self.base_color_sampler_non_filtering),
                ],
            ],
            mesh_buffers,
            None,
            None,
            None,
            output_view,
            None,
            Some(depth_view),
        );
    }
}
