use crate::render_pipeline::base_render_pipeline::VertexBufferType;
use naga::*;
use std::collections::HashMap;
use wgpu::*;

pub struct VertexBufferLayoutBuilder {
    vertex_attributes: Vec<Vec<VertexAttribute>>,
    vertex_buffer_type: VertexBufferType,
}

impl VertexBufferLayoutBuilder {
    pub fn new(
        vertex_attributes: Vec<Vec<VertexAttribute>>,
        vertex_buffer_type: VertexBufferType,
    ) -> VertexBufferLayoutBuilder {
        VertexBufferLayoutBuilder {
            vertex_attributes,
            vertex_buffer_type,
        }
    }

    pub fn get_vertex_buffer_layout(&self) -> Vec<VertexBufferLayout> {
        match &self.vertex_buffer_type {
            VertexBufferType::Interleaved(verification) => {
                let vertex_buffer_layout = VertexBufferLayout {
                    array_stride: verification.size as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &self.vertex_attributes.first().unwrap(),
                };

                vec![vertex_buffer_layout]
            }
            VertexBufferType::Noninterleaved => self
                .vertex_attributes
                .iter()
                .map(|vertex_attribute| {
                    let vertex_buffer_layout = VertexBufferLayout {
                        array_stride: vertex_attribute.first().unwrap().format.size(),
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: vertex_attribute,
                    };
                    vertex_buffer_layout
                })
                .collect(),
        }
    }
}

pub struct Reflection {
    module: Module,
    interleaved_vertex_attributes: Vec<wgpu::VertexAttribute>,
    noninterleaved_vertex_attributes: Vec<Vec<wgpu::VertexAttribute>>,
    array_stride: u64,
    vs_entry_point: String,
    fs_entry_point: String,
    bind_group_layout_entrys: Vec<Vec<BindGroupLayoutEntry>>,
}

impl Reflection {
    pub fn new(shader_path: &str) -> Option<Reflection> {
        let module = Self::get_module_from_path(&shader_path);
        match module {
            Some(module) => {
                let (vertex_attributes, array_stride) = Self::extract_vertex_attributes(&module);
                let vs_entry_point = Self::extract_vertex_entry_point_name(&module);
                let fs_entry_point = Self::extract_fragment_entry_point_name(&module);
                let bind_group_layout_entrys = Self::extract_bind_group_layout_entrys(&module);

                let mut noninterleaved_vertex_attributes = Vec::<Vec<wgpu::VertexAttribute>>::new();
                for mut vertex_attribute in vertex_attributes.to_vec() {
                    vertex_attribute.offset = 0;
                    noninterleaved_vertex_attributes.push(vec![vertex_attribute]);
                }

                let reflection = Reflection {
                    module,
                    interleaved_vertex_attributes: vertex_attributes,
                    array_stride,
                    vs_entry_point,
                    fs_entry_point,
                    bind_group_layout_entrys,
                    noninterleaved_vertex_attributes,
                };

                Some(reflection)
            }
            None => None,
        }
    }

    pub fn get_module_from_path(shader_path: &str) -> Option<Module> {
        match std::fs::read_to_string(shader_path) {
            Ok(shader_source) => match front::wgsl::parse_str(&shader_source) {
                Ok(module) => Some(module),
                Err(error) => {
                    log::warn!("{:?}", error);
                    None
                }
            },
            Err(error) => {
                log::warn!("{:?}", error);
                None
            }
        }
    }

    pub fn make_vertex_buffer_layout_builder(
        &self,
        vertex_buffer_type: VertexBufferType,
    ) -> VertexBufferLayoutBuilder {
        match &vertex_buffer_type {
            VertexBufferType::Interleaved(verification) => {
                let mut offsets = Vec::<u64>::new();
                let mut current_offset: u64 = 0;
                let mut vertex_attributes = self.interleaved_vertex_attributes.clone();

                for field in verification.fields.iter() {
                    match field {
                        type_layout::Field::Field { size, .. } => {
                            offsets.push(current_offset);
                            current_offset += *size as u64;
                        }
                        type_layout::Field::Padding { size } => {
                            current_offset += *size as u64;
                        }
                    }
                }
                debug_assert_eq!(vertex_attributes.len(), offsets.len());
                for (vertex_attribute, offset) in vertex_attributes.iter_mut().zip(offsets) {
                    vertex_attribute.offset = offset;
                }
                let builder =
                    VertexBufferLayoutBuilder::new(vec![vertex_attributes], vertex_buffer_type);
                builder
            }
            VertexBufferType::Noninterleaved => {
                let builder = VertexBufferLayoutBuilder::new(
                    self.noninterleaved_vertex_attributes.clone(),
                    vertex_buffer_type,
                );
                builder
            }
        }
    }

    pub fn extract_vertex_entry_point_name(module: &naga::Module) -> String {
        let mut name = String::new();
        for entry_point in module.entry_points.iter() {
            match entry_point.stage {
                ShaderStage::Vertex => {
                    name = entry_point.name.clone();
                    break;
                }
                ShaderStage::Fragment => {}
                ShaderStage::Compute => {}
            }
        }
        name
    }

    pub fn extract_fragment_entry_point_name(module: &naga::Module) -> String {
        let mut name = String::new();
        for entry_point in module.entry_points.iter() {
            match entry_point.stage {
                ShaderStage::Vertex => {}
                ShaderStage::Fragment => {
                    name = entry_point.name.clone();
                    break;
                }
                ShaderStage::Compute => {}
            }
        }
        name
    }

    pub fn extract_vertex_attributes(module: &naga::Module) -> (Vec<wgpu::VertexAttribute>, u64) {
        let mut attributes = Vec::new();
        for entry_point in module.entry_points.iter() {
            match entry_point.stage {
                naga::ShaderStage::Vertex => {
                    let mut offset: u64 = 0;
                    for arg in entry_point.function.arguments.iter() {
                        let arg_type = module.types.get_handle(arg.ty).unwrap();
                        // let arg_size = arg_type.inner.size(&module.constants);

                        match &arg_type.inner {
                            naga::TypeInner::Scalar { kind, .. } => {
                                let mut attribute = wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32,
                                    offset,
                                    shader_location: 0,
                                };
                                match arg.binding.clone().unwrap() {
                                    naga::Binding::BuiltIn(built_in) => match built_in {
                                        BuiltIn::VertexIndex => {
                                            log::trace!("{:?}", "Skip VertexIndex");
                                        }
                                        _ => {
                                            todo!()
                                        }
                                    },
                                    naga::Binding::Location { location, .. } => {
                                        attribute.shader_location = location;
                                    }
                                }
                                match kind {
                                    naga::ScalarKind::Sint => {
                                        attribute.format = wgpu::VertexFormat::Sint32;
                                    }
                                    naga::ScalarKind::Uint => {
                                        attribute.format = wgpu::VertexFormat::Uint32;
                                    }
                                    naga::ScalarKind::Float => {
                                        attribute.format = wgpu::VertexFormat::Float32;
                                    }
                                    naga::ScalarKind::Bool => todo!(),
                                }
                                offset += attribute.format.size();
                                attributes.push(attribute);
                            }
                            naga::TypeInner::Vector { size, kind, .. } => {
                                let mut attribute = wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32,
                                    offset,
                                    shader_location: 0,
                                };
                                match arg.binding.clone().unwrap() {
                                    naga::Binding::BuiltIn(_) => todo!(),
                                    naga::Binding::Location { location, .. } => {
                                        attribute.shader_location = location;
                                    }
                                }
                                match kind {
                                    naga::ScalarKind::Sint => todo!(),
                                    naga::ScalarKind::Uint => todo!(),
                                    naga::ScalarKind::Float => match size {
                                        naga::VectorSize::Bi => {
                                            attribute.format = wgpu::VertexFormat::Float32x2;
                                        }
                                        naga::VectorSize::Tri => {
                                            attribute.format = wgpu::VertexFormat::Float32x3;
                                        }
                                        naga::VectorSize::Quad => {
                                            attribute.format = wgpu::VertexFormat::Float32x4;
                                        }
                                    },
                                    naga::ScalarKind::Bool => todo!(),
                                }
                                offset += attribute.format.size();
                                attributes.push(attribute);
                            }
                            _ => {}
                        }
                    }
                }
                naga::ShaderStage::Fragment => {}
                naga::ShaderStage::Compute => {}
            }
        }
        let array_stride: u64 = attributes.iter().fold(0, |acc, &x| acc + x.format.size());
        (attributes, array_stride)
    }

    pub fn extract_bind_group_layout_entrys(
        module: &naga::Module,
    ) -> Vec<Vec<BindGroupLayoutEntry>> {
        let mut bind_group_layout_entrys_map: HashMap<u32, Vec<BindGroupLayoutEntry>> =
            HashMap::new();

        for (_, global_variable) in module.global_variables.iter() {
            // log::trace!("{:?}", global_variable);
            let arg_type = module.types.get_handle(global_variable.ty).unwrap();
            let space = global_variable.space;
            let binding = &global_variable.binding;

            match space {
                AddressSpace::Uniform => {
                    let bind_group_layout_entry = BindGroupLayoutEntry {
                        binding: binding.clone().unwrap().binding,
                        visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    };
                    match bind_group_layout_entrys_map.get_mut(&binding.clone().unwrap().group) {
                        Some(value) => {
                            value.push(bind_group_layout_entry);
                        }
                        None => {
                            let mut new_vec = Vec::new();
                            new_vec.push(bind_group_layout_entry);
                            bind_group_layout_entrys_map
                                .insert(binding.clone().unwrap().group, new_vec);
                        }
                    }
                }
                AddressSpace::Handle => match &arg_type.inner {
                    TypeInner::Image { dim, .. } => {
                        let bind_group_layout_entry = BindGroupLayoutEntry {
                            binding: binding.clone().unwrap().binding,
                            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: Self::image_dimension2texture_dimension(*dim),
                                multisampled: false,
                            },
                            count: None,
                        };

                        match bind_group_layout_entrys_map.get_mut(&binding.clone().unwrap().group)
                        {
                            Some(value) => {
                                value.push(bind_group_layout_entry);
                            }
                            None => {
                                let mut new_vec = Vec::new();
                                new_vec.push(bind_group_layout_entry);
                                bind_group_layout_entrys_map
                                    .insert(binding.clone().unwrap().group, new_vec);
                            }
                        }
                    }
                    TypeInner::Sampler { comparison } => {
                        let sampler_binding_type: SamplerBindingType;
                        if *comparison {
                            sampler_binding_type = SamplerBindingType::Comparison;
                        } else {
                            sampler_binding_type = SamplerBindingType::Filtering;
                        }
                        let bind_group_layout_entry = BindGroupLayoutEntry {
                            binding: binding.clone().unwrap().binding,
                            visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(sampler_binding_type),
                            count: None,
                        };
                        match bind_group_layout_entrys_map.get_mut(&binding.clone().unwrap().group)
                        {
                            Some(value) => {
                                value.push(bind_group_layout_entry);
                            }
                            None => {
                                let mut new_vec = Vec::new();
                                new_vec.push(bind_group_layout_entry);
                                bind_group_layout_entrys_map
                                    .insert(binding.clone().unwrap().group, new_vec);
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        let mut keys = bind_group_layout_entrys_map
            .keys()
            .map(|x| *x)
            .collect::<Vec<u32>>();
        keys.sort();

        let mut bind_group_layout_entrys: Vec<Vec<BindGroupLayoutEntry>> = Vec::new();

        for key in keys.iter() {
            let value = bind_group_layout_entrys_map.remove(&key).unwrap();
            bind_group_layout_entrys.push(value);
        }

        bind_group_layout_entrys
    }

    fn image_dimension2texture_dimension(image_dimension: ImageDimension) -> TextureViewDimension {
        match image_dimension {
            ImageDimension::D1 => TextureViewDimension::D1,
            ImageDimension::D2 => TextureViewDimension::D2,
            ImageDimension::D3 => TextureViewDimension::D3,
            ImageDimension::Cube => TextureViewDimension::Cube,
        }
    }

    pub fn get_array_stride(&self) -> u64 {
        self.array_stride
    }

    pub fn get_interleaved_vertex_attributes(&self) -> &[VertexAttribute] {
        self.interleaved_vertex_attributes.as_ref()
    }

    pub fn get_module(&self) -> &Module {
        &self.module
    }

    pub fn get_vs_entry_point(&self) -> &str {
        self.vs_entry_point.as_ref()
    }

    pub fn get_fs_entry_point(&self) -> &str {
        self.fs_entry_point.as_ref()
    }

    pub fn get_bind_group_layout_entrys(&self) -> &[Vec<BindGroupLayoutEntry>] {
        self.bind_group_layout_entrys.as_ref()
    }

    pub fn noninterleaved_vertex_attribute(&self, index: usize) -> &[VertexAttribute] {
        self.noninterleaved_vertex_attributes
            .get(index)
            .unwrap()
            .as_ref()
    }
}
