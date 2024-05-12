use crate::{shader_library::ShaderLibrary, VertexBufferType};
use naga::*;
use std::collections::HashMap;
use wgpu::*;

#[derive(Debug, Clone)]
pub enum EPipelineType {
    Render(EntryPoint, EntryPoint),
    Compute(EntryPoint),
}

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
            VertexBufferType::Interleaved(verifications) => {
                assert_eq!(verifications.len(), self.vertex_attributes.len());
                verifications
                    .iter()
                    .zip(self.vertex_attributes.iter())
                    .map(|(verification, vertex_attribute)| VertexBufferLayout {
                        array_stride: verification.size as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &vertex_attribute,
                    })
                    .collect()
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
    vertex_attributes: Vec<wgpu::VertexAttribute>,
    array_stride: u64,
    pipeline_type: EPipelineType,
    bind_group_layout_entrys: Vec<Vec<BindGroupLayoutEntry>>,
}

impl Reflection {
    pub fn new(shader_code: &str, is_enable_validation: bool) -> crate::error::Result<Reflection> {
        let module = front::wgsl::parse_str(&shader_code)
            .map_err(|err| crate::error::Error::ShaderReflection(err, None))?;

        if is_enable_validation {
            ShaderLibrary::validate_shader_module(&module)?
        }

        let render_entry_points = Self::extract_render_entry_point(&module);
        let cs_entry_point = Self::extract_compute_entry_point(&module);
        let pipeline_type: EPipelineType;

        if let Some(render_entry_points) = render_entry_points {
            pipeline_type = EPipelineType::Render(render_entry_points.0, render_entry_points.1);
        } else if let Some(cs_entry_point) = cs_entry_point {
            pipeline_type = EPipelineType::Compute(cs_entry_point);
        } else {
            return Err(crate::error::Error::ShaderNotSupported(None));
        }

        let (vertex_attributes, array_stride) = Self::extract_vertex_attributes(&module);
        let bind_group_layout_entrys =
            Self::extract_bind_group_layout_entrys(&module, &pipeline_type);

        let reflection = Reflection {
            module,
            vertex_attributes,
            array_stride,
            pipeline_type,
            bind_group_layout_entrys,
        };

        Ok(reflection)
    }

    pub fn make_vertex_buffer_layout_builder(
        &self,
        vertex_buffer_type: VertexBufferType,
    ) -> VertexBufferLayoutBuilder {
        match &vertex_buffer_type {
            VertexBufferType::Interleaved(verifications) => {
                let mut vertex_attributes_layout: Vec<Vec<VertexAttribute>> =
                    Vec::with_capacity(verifications.len());
                let mut index: usize = 0;
                let mut _vertex_attributes = self.vertex_attributes.clone();

                for verification in verifications {
                    let mut vertex_attributes: Vec<VertexAttribute> =
                        Vec::with_capacity(verification.fields.len());
                    let mut current_offset: u64 = 0;
                    let mut offsets = Vec::<u64>::new();
                    for field in verification.fields.iter() {
                        match field {
                            type_layout::Field::Field { size, .. } => {
                                let attr = _vertex_attributes.get_mut(index).unwrap();
                                attr.offset = current_offset;
                                vertex_attributes.push(*attr);
                                offsets.push(current_offset);
                                current_offset += *size as u64;
                                index += 1;
                            }
                            type_layout::Field::Padding { size } => {
                                current_offset += *size as u64;
                            }
                        }
                    }
                    debug_assert_eq!(vertex_attributes.len(), offsets.len());
                    vertex_attributes_layout.push(vertex_attributes);
                }
                let builder =
                    VertexBufferLayoutBuilder::new(vertex_attributes_layout, vertex_buffer_type);
                builder
            }
            VertexBufferType::Noninterleaved => {
                let mut noninterleaved_vertex_attributes = Vec::<Vec<wgpu::VertexAttribute>>::new();
                for mut vertex_attribute in self.vertex_attributes.to_vec() {
                    vertex_attribute.offset = 0;
                    noninterleaved_vertex_attributes.push(vec![vertex_attribute]);
                }
                let builder = VertexBufferLayoutBuilder::new(
                    noninterleaved_vertex_attributes,
                    vertex_buffer_type,
                );
                builder
            }
        }
    }

    fn extract_render_entry_point(module: &naga::Module) -> Option<(EntryPoint, EntryPoint)> {
        let mut vs_entry_point: Option<EntryPoint> = None;
        let mut fs_entry_point: Option<EntryPoint> = None;
        for entry_point in module.entry_points.iter() {
            match entry_point.stage {
                ShaderStage::Vertex => {
                    vs_entry_point = Some(entry_point.clone());
                }
                ShaderStage::Fragment => {
                    fs_entry_point = Some(entry_point.clone());
                }
                ShaderStage::Compute => {}
            }
        }
        if let (Some(vs_entry_point), Some(fs_entry_point)) = (vs_entry_point, fs_entry_point) {
            return Some((vs_entry_point, fs_entry_point));
        } else {
            return None;
        }
    }

    fn extract_compute_entry_point(module: &naga::Module) -> Option<EntryPoint> {
        for entry_point in module.entry_points.iter() {
            match entry_point.stage {
                ShaderStage::Vertex => {}
                ShaderStage::Fragment => {}
                ShaderStage::Compute => {
                    return Some(entry_point.clone());
                }
            }
        }
        None
    }

    fn extract_vertex_attributes(module: &naga::Module) -> (Vec<wgpu::VertexAttribute>, u64) {
        let mut attributes = Vec::new();
        let Some(entry_point) = module
            .entry_points
            .iter()
            .find(|x| x.stage == naga::ShaderStage::Vertex)
        else {
            return (vec![], 0);
        };

        let mut offset: u64 = 0;
        for arg in entry_point.function.arguments.iter() {
            let arg_type = module.types.get_handle(arg.ty).unwrap();
            // let arg_size = arg_type.inner.size(&module.constants);

            match &arg_type.inner {
                naga::TypeInner::Scalar(scaler) => {
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
                    match scaler.kind {
                        naga::ScalarKind::Sint => {
                            attribute.format = wgpu::VertexFormat::Sint32;
                        }
                        naga::ScalarKind::Uint => {
                            attribute.format = wgpu::VertexFormat::Uint32;
                        }
                        naga::ScalarKind::Float => {
                            attribute.format = wgpu::VertexFormat::Float32;
                        }
                        _ => todo!(),
                    }
                    offset += attribute.format.size();
                    attributes.push(attribute);
                }
                naga::TypeInner::Vector { size, scalar } => {
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
                    match scalar.kind {
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
                        _ => todo!(),
                    }
                    offset += attribute.format.size();
                    attributes.push(attribute);
                }
                naga::TypeInner::Struct { members, .. } => {
                    for member in members {
                        let Some(binding) = &member.binding else {
                            continue;
                        };
                        let naga::Binding::Location { location, .. } = binding else {
                            continue;
                        };
                        let Ok(arg_type) = module.types.get_handle(member.ty) else {
                            continue;
                        };

                        let TypeInner::Vector { size, scalar } = arg_type.inner else {
                            continue;
                        };
                        let mut attribute = wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32,
                            offset,
                            shader_location: *location,
                        };
                        match scalar.kind {
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
                            naga::ScalarKind::Sint => match size {
                                naga::VectorSize::Bi => {
                                    attribute.format = wgpu::VertexFormat::Sint32x2;
                                }
                                naga::VectorSize::Tri => {
                                    attribute.format = wgpu::VertexFormat::Sint32x3;
                                }
                                naga::VectorSize::Quad => {
                                    attribute.format = wgpu::VertexFormat::Sint32x4;
                                }
                            },
                            _ => todo!(),
                        }
                        offset += attribute.format.size();
                        attributes.push(attribute);
                    }
                }
                _ => {}
            }
        }

        let array_stride: u64 = attributes.iter().fold(0, |acc, &x| acc + x.format.size());
        (attributes, array_stride)
    }

    fn create_bind_group_layout_entry(
        binding: u32,
        space: AddressSpace,
        arg_type: &Type,
        pipeline_type: &EPipelineType,
    ) -> Option<BindGroupLayoutEntry> {
        match space {
            AddressSpace::Storage { access } => {
                let bind_group_layout_entry = BindGroupLayoutEntry {
                    binding,
                    visibility: match pipeline_type {
                        EPipelineType::Render(_, _) => {
                            ShaderStages::VERTEX | ShaderStages::FRAGMENT
                        }
                        EPipelineType::Compute(_) => ShaderStages::COMPUTE,
                    },
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage {
                            read_only: !access.contains(StorageAccess::STORE),
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                };
                return Some(bind_group_layout_entry);
            }
            AddressSpace::Uniform => {
                let bind_group_layout_entry = BindGroupLayoutEntry {
                    binding,
                    visibility: match pipeline_type {
                        EPipelineType::Render(_, _) => {
                            ShaderStages::VERTEX | ShaderStages::FRAGMENT
                        }
                        EPipelineType::Compute(_) => ShaderStages::COMPUTE,
                    },
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                };
                return Some(bind_group_layout_entry);
            }
            AddressSpace::Handle => match &arg_type.inner {
                TypeInner::Image {
                    dim,
                    arrayed,
                    class,
                } => {
                    let binding_type: BindingType = match class {
                        ImageClass::Sampled { kind, multi } => BindingType::Texture {
                            sample_type: match kind {
                                ScalarKind::Sint => TextureSampleType::Sint,
                                ScalarKind::Uint => TextureSampleType::Uint,
                                ScalarKind::Float => TextureSampleType::Float { filterable: true },
                                _ => todo!(),
                            },
                            view_dimension: Self::image_dimension2texture_dimension(*dim, *arrayed),
                            multisampled: *multi,
                        },
                        ImageClass::Depth { multi } => todo!(),
                        ImageClass::Storage { format, access } => BindingType::StorageTexture {
                            access: Self::storage_access2storage_texture_access(access),
                            format: Self::storage_format2texture_format(format),
                            view_dimension: Self::image_dimension2texture_dimension(*dim, *arrayed),
                        },
                    };

                    let bind_group_layout_entry = BindGroupLayoutEntry {
                        binding,
                        visibility: match pipeline_type {
                            EPipelineType::Render(_, _) => {
                                ShaderStages::VERTEX | ShaderStages::FRAGMENT
                            }
                            EPipelineType::Compute(_) => ShaderStages::COMPUTE,
                        },
                        ty: binding_type,
                        count: None,
                    };
                    return Some(bind_group_layout_entry);
                }
                TypeInner::Sampler { comparison } => {
                    let sampler_binding_type: SamplerBindingType;
                    if *comparison {
                        sampler_binding_type = SamplerBindingType::Comparison;
                    } else {
                        sampler_binding_type = SamplerBindingType::Filtering;
                    }
                    let bind_group_layout_entry = BindGroupLayoutEntry {
                        binding,
                        visibility: match pipeline_type {
                            EPipelineType::Render(_, _) => {
                                ShaderStages::VERTEX | ShaderStages::FRAGMENT
                            }
                            EPipelineType::Compute(_) => ShaderStages::COMPUTE,
                        },
                        ty: BindingType::Sampler(sampler_binding_type),
                        count: None,
                    };
                    return Some(bind_group_layout_entry);
                }
                _ => {}
            },
            _ => {}
        }
        None
    }

    fn extract_bind_group_layout_entrys(
        module: &naga::Module,
        pipeline_type: &EPipelineType,
    ) -> Vec<Vec<BindGroupLayoutEntry>> {
        let mut bind_group_layout_entrys_map: HashMap<u32, Vec<BindGroupLayoutEntry>> =
            HashMap::new();

        for (_, global_variable) in module.global_variables.iter() {
            // log::trace!("{:?}", global_variable);
            let arg_type = module.types.get_handle(global_variable.ty).unwrap();
            let space = global_variable.space;
            let binding = &global_variable.binding;
            let group = binding.clone().unwrap().group;
            let Some(bind_group_layout_entry) = Self::create_bind_group_layout_entry(
                binding.clone().unwrap().binding,
                space,
                arg_type,
                pipeline_type,
            ) else {
                continue;
            };

            if !bind_group_layout_entrys_map.contains_key(&group) {
                bind_group_layout_entrys_map.insert(group, Vec::new());
            }
            let Some(bind_group_layout_entrys) = bind_group_layout_entrys_map.get_mut(&group)
            else {
                continue;
            };
            bind_group_layout_entrys.push(bind_group_layout_entry);
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

    fn image_dimension2texture_dimension(
        image_dimension: ImageDimension,
        arrayed: bool,
    ) -> TextureViewDimension {
        if arrayed {
            match image_dimension {
                ImageDimension::D1 => TextureViewDimension::D1,
                ImageDimension::D2 => TextureViewDimension::D2Array,
                ImageDimension::D3 => TextureViewDimension::D3,
                ImageDimension::Cube => TextureViewDimension::CubeArray,
            }
        } else {
            match image_dimension {
                ImageDimension::D1 => TextureViewDimension::D1,
                ImageDimension::D2 => TextureViewDimension::D2,
                ImageDimension::D3 => TextureViewDimension::D3,
                ImageDimension::Cube => TextureViewDimension::Cube,
            }
        }
    }

    fn storage_access2storage_texture_access(access: &StorageAccess) -> StorageTextureAccess {
        if access.contains(StorageAccess::LOAD) && access.contains(StorageAccess::STORE) {
            StorageTextureAccess::ReadWrite
        } else if access.contains(StorageAccess::LOAD) {
            StorageTextureAccess::ReadOnly
        } else {
            StorageTextureAccess::WriteOnly
        }
    }

    fn storage_format2texture_format(storage_format: &StorageFormat) -> TextureFormat {
        match storage_format {
            StorageFormat::R8Unorm => TextureFormat::R8Unorm,
            StorageFormat::R8Snorm => TextureFormat::R8Snorm,
            StorageFormat::R8Uint => TextureFormat::R8Uint,
            StorageFormat::R8Sint => TextureFormat::R8Sint,
            StorageFormat::R16Uint => TextureFormat::R16Uint,
            StorageFormat::R16Sint => TextureFormat::R16Sint,
            StorageFormat::R16Float => TextureFormat::R16Float,
            StorageFormat::Rg8Unorm => TextureFormat::Rg8Unorm,
            StorageFormat::Rg8Snorm => TextureFormat::Rg8Snorm,
            StorageFormat::Rg8Uint => TextureFormat::Rg8Uint,
            StorageFormat::Rg8Sint => TextureFormat::Rg8Sint,
            StorageFormat::R32Uint => TextureFormat::R32Uint,
            StorageFormat::R32Sint => TextureFormat::R32Sint,
            StorageFormat::R32Float => TextureFormat::R32Float,
            StorageFormat::Rg16Uint => TextureFormat::Rg16Uint,
            StorageFormat::Rg16Sint => TextureFormat::Rg16Sint,
            StorageFormat::Rg16Float => TextureFormat::Rg16Float,
            StorageFormat::Rgba8Unorm => TextureFormat::Rgba8Unorm,
            StorageFormat::Rgba8Snorm => TextureFormat::Rgba8Snorm,
            StorageFormat::Rgba8Uint => TextureFormat::Rgba8Uint,
            StorageFormat::Rgba8Sint => TextureFormat::Rgba8Sint,
            StorageFormat::Rgb10a2Unorm => TextureFormat::Rgb10a2Unorm,
            StorageFormat::Rg11b10Float => TextureFormat::Rg11b10Float,
            StorageFormat::Rg32Uint => TextureFormat::Rg32Uint,
            StorageFormat::Rg32Sint => TextureFormat::Rg32Sint,
            StorageFormat::Rg32Float => TextureFormat::Rg32Float,
            StorageFormat::Rgba16Uint => TextureFormat::Rgba16Uint,
            StorageFormat::Rgba16Sint => TextureFormat::Rgba16Sint,
            StorageFormat::Rgba16Float => TextureFormat::Rgba16Float,
            StorageFormat::Rgba32Uint => TextureFormat::Rgba32Uint,
            StorageFormat::Rgba32Sint => TextureFormat::Rgba32Sint,
            StorageFormat::Rgba32Float => TextureFormat::Rgba32Float,
            StorageFormat::R16Unorm => TextureFormat::R16Unorm,
            StorageFormat::R16Snorm => TextureFormat::R16Snorm,
            StorageFormat::Rg16Unorm => TextureFormat::Rg16Unorm,
            StorageFormat::Rg16Snorm => TextureFormat::Rg16Snorm,
            StorageFormat::Rgba16Unorm => TextureFormat::Rgba16Unorm,
            StorageFormat::Rgba16Snorm => TextureFormat::Rgba16Snorm,
            StorageFormat::Bgra8Unorm => TextureFormat::Bgra8Unorm,
            StorageFormat::Rgb10a2Uint => TextureFormat::Rgb10a2Uint,
        }
    }

    pub fn get_array_stride(&self) -> u64 {
        self.array_stride
    }

    pub fn get_module(&self) -> &Module {
        &self.module
    }

    pub fn get_bind_group_layout_entrys(&self) -> &[Vec<BindGroupLayoutEntry>] {
        self.bind_group_layout_entrys.as_ref()
    }

    pub fn get_pipeline_type(&self) -> &EPipelineType {
        &self.pipeline_type
    }
}
