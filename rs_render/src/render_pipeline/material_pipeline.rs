use crate::{
    base_render_pipeline::{BaseRenderPipeline, ColorAttachment},
    base_render_pipeline_pool::{BaseRenderPipelineBuilder, BaseRenderPipelinePool},
    command::{MaterialRenderPipelineHandle, Viewport},
    gpu_vertex_buffer::GpuVertexBufferImp,
    shader_library::ShaderLibrary,
    vertex_data_type::mesh_vertex::{MeshVertex0, MeshVertex1, MeshVertex2, MeshVertex5},
    view_mode::EViewModeType,
    VertexBufferType,
};
use rs_render_types::MaterialOptions;
use std::{collections::HashMap, sync::Arc};
use type_layout::TypeLayout;
use wgpu::*;

pub struct MaterialRenderPipeline {
    pub base_render_pipeline: Arc<BaseRenderPipeline>,
    builder: BaseRenderPipelineBuilder,
}

impl MaterialRenderPipeline {
    pub fn new(
        material_render_pipeline_handle: MaterialRenderPipelineHandle,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> crate::error::Result<MaterialRenderPipeline> {
        Self::skin(
            material_render_pipeline_handle,
            device,
            shader_library,
            texture_format,
            pool,
        )
    }

    fn new_internal(
        material_render_pipeline_handle: MaterialRenderPipelineHandle,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
        is_skin: bool,
    ) -> crate::error::Result<MaterialRenderPipeline> {
        let shader_name = ShaderLibrary::get_material_shader_name(
            material_render_pipeline_handle,
            if is_skin {
                &MaterialOptions { is_skin: true }
            } else {
                &MaterialOptions { is_skin: false }
            },
        );

        let mut builder = BaseRenderPipelineBuilder::default();
        builder.targets = vec![Some(ColorTargetState {
            format: texture_format.clone(),
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        })];
        builder.shader_name = shader_name;
        builder.depth_stencil = Some(DepthStencilState {
            depth_compare: CompareFunction::Less,
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        });
        if is_skin {
            builder.vertex_buffer_type = Some(VertexBufferType::Interleaved(vec![
                MeshVertex5::type_layout(),
                MeshVertex0::type_layout(),
                MeshVertex1::type_layout(),
                MeshVertex2::type_layout(),
            ]));
        } else {
            builder.vertex_buffer_type = Some(VertexBufferType::Interleaved(vec![
                MeshVertex5::type_layout(),
                MeshVertex0::type_layout(),
                MeshVertex1::type_layout(),
            ]));
        }

        builder.primitive = Some(PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            cull_mode: None,
            polygon_mode: PolygonMode::Fill,
            ..Default::default()
        });

        let base_render_pipeline = pool.get(device, shader_library, &builder);

        Ok(MaterialRenderPipeline {
            base_render_pipeline,
            builder,
        })
    }

    pub fn skin(
        material_render_pipeline_handle: MaterialRenderPipelineHandle,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> crate::error::Result<MaterialRenderPipeline> {
        Self::new_internal(
            material_render_pipeline_handle,
            device,
            shader_library,
            texture_format,
            pool,
            true,
        )
    }

    pub fn static_mesh(
        material_render_pipeline_handle: MaterialRenderPipelineHandle,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> crate::error::Result<MaterialRenderPipeline> {
        Self::new_internal(
            material_render_pipeline_handle,
            device,
            shader_library,
            texture_format,
            pool,
            false,
        )
    }

    pub fn draw(
        &self,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        depth_view: &TextureView,
        mesh_buffers: &[GpuVertexBufferImp],
        binding_resource: Vec<Vec<BindingResource<'_>>>,
        scissor_rect: Option<glam::UVec4>,
        viewport: Option<Viewport>,
    ) {
        self.base_render_pipeline.draw_resources(
            device,
            queue,
            binding_resource,
            mesh_buffers,
            &[ColorAttachment {
                color_ops: None,
                view: output_view,
                resolve_target: None,
            }],
            None,
            None,
            Some(depth_view),
            scissor_rect,
            viewport,
        );
    }
    pub fn set_view_mode(
        &mut self,
        view_mode: EViewModeType,
        device: &Device,
        shader_library: &ShaderLibrary,
        pool: &mut BaseRenderPipelinePool,
    ) {
        match view_mode {
            EViewModeType::Wireframe => {
                self.builder.primitive = Some(PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Line,
                    ..Default::default()
                });
            }
            EViewModeType::Lit => {
                self.builder.primitive = Some(PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    cull_mode: None,
                    polygon_mode: PolygonMode::Fill,
                    ..Default::default()
                });
            }
            EViewModeType::Unlit => todo!(),
        }

        self.base_render_pipeline = pool.get(device, shader_library, &self.builder);
    }
}

pub struct VariantMaterialRenderPipeline {
    pipelines: HashMap<MaterialOptions, MaterialRenderPipeline>,
}

impl VariantMaterialRenderPipeline {
    pub fn new(
        handle: MaterialRenderPipelineHandle,
        options: Vec<MaterialOptions>,
        device: &Device,
        shader_library: &ShaderLibrary,
        texture_format: &TextureFormat,
        pool: &mut BaseRenderPipelinePool,
    ) -> VariantMaterialRenderPipeline {
        let mut variant_material_render_pipeline = VariantMaterialRenderPipeline {
            pipelines: HashMap::new(),
        };
        for option in options {
            let pipeline = if option.is_skin {
                MaterialRenderPipeline::skin(handle, device, shader_library, texture_format, pool)
            } else {
                MaterialRenderPipeline::static_mesh(
                    handle,
                    device,
                    shader_library,
                    texture_format,
                    pool,
                )
            };
            match pipeline {
                Ok(pipeline) => {
                    let old_value = variant_material_render_pipeline
                        .pipelines
                        .insert(option, pipeline);
                    debug_assert!(old_value.is_none());
                }
                Err(err) => {
                    log::warn!("{}", err);
                }
            }
        }
        variant_material_render_pipeline
    }

    pub fn get(&self, options: &MaterialOptions) -> Option<&MaterialRenderPipeline> {
        self.pipelines.get(options)
    }

    pub fn get_mut(&mut self, options: &MaterialOptions) -> Option<&mut MaterialRenderPipeline> {
        self.pipelines.get_mut(options)
    }

    pub fn set_view_mode(
        &mut self,
        view_mode: EViewModeType,
        device: &Device,
        shader_library: &ShaderLibrary,
        pool: &mut BaseRenderPipelinePool,
    ) {
        for (_, pipeline) in self.pipelines.iter_mut() {
            pipeline.set_view_mode(view_mode, device, shader_library, pool);
        }
    }
}
