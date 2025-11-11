pub mod acceleration_bake;
pub mod antialias_type;
pub mod bake_info;
pub mod base_compute_pipeline;
pub mod base_compute_pipeline_pool;
pub mod base_render_pipeline;
pub mod base_render_pipeline_pool;
pub mod bind_group_layout_entry_hook;
pub mod command;
pub mod compute_pipeline;
pub mod constants;
pub mod cube_map;
pub mod default_textures;
pub mod depth_texture;
pub mod egui_render;
pub mod error;
pub mod ffi;
pub mod frame_buffer;
pub mod global_shaders;
pub mod global_uniform;
pub mod gpu_buffer;
pub(crate) mod gpu_vertex_buffer;
pub mod ibl_readback;
pub mod light_culling;
pub mod misc;
pub mod multi_res_mesh;
pub mod multiple_resolution_meshs_pass;
pub mod prebake_ibl;
pub mod reflection;
pub mod render_pipeline;
#[cfg(feature = "renderdoc")]
pub mod renderdoc;
pub mod renderer;
pub mod sampler_cache;
pub mod scene_viewport;
pub mod sdf2d_generator;
pub mod shader_library;
pub mod shadow_pass;
pub mod texture_loader;
pub mod vertex_data_type;
pub mod view_mode;
pub mod virtual_texture_pass;
pub mod virtual_texture_source;

use rs_render_core::buffer_dimensions;
use rs_render_core::texture_readback;

pub fn get_buildin_shader_dir() -> std::path::PathBuf {
    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();
    engine_root_dir.join("rs_render/shaders")
}

pub(crate) fn get_old_buildin_shader_dir() -> std::path::PathBuf {
    let engine_root_dir = rs_core_minimal::file_manager::get_engine_root_dir();
    engine_root_dir.join("rs_computer_graphics/src/shader")
}

trait TypeLayoutInfoClone {
    fn clone(&self) -> type_layout::TypeLayoutInfo;
}

impl TypeLayoutInfoClone for type_layout::TypeLayoutInfo {
    fn clone(&self) -> type_layout::TypeLayoutInfo {
        type_layout::TypeLayoutInfo {
            name: self.name.clone(),
            size: self.size,
            alignment: self.alignment,
            fields: {
                let mut fields: Vec<type_layout::Field> = vec![];
                for field in self.fields.iter() {
                    let field_clone = match field {
                        type_layout::Field::Field { name, ty, size } => type_layout::Field::Field {
                            name: name.clone(),
                            ty: ty.clone(),
                            size: *size,
                        },
                        type_layout::Field::Padding { size } => {
                            type_layout::Field::Padding { size: *size }
                        }
                    };
                    fields.push(field_clone);
                }
                fields
            },
        }
    }
}

#[derive(Debug)]
pub enum VertexBufferType {
    Interleaved(Vec<type_layout::TypeLayoutInfo>),
    Noninterleaved,
}

impl Clone for VertexBufferType {
    fn clone(&self) -> Self {
        match self {
            Self::Interleaved(arg0) => Self::Interleaved(arg0.iter().map(|x| x.clone()).collect()),
            Self::Noninterleaved => Self::Noninterleaved,
        }
    }
}

impl std::hash::Hash for VertexBufferType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl PartialEq for VertexBufferType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Interleaved(l0), Self::Interleaved(r0)) => {
                if l0.len() != r0.len() {
                    return false;
                }
                for (left, right) in std::iter::zip(l0, r0) {
                    if left.alignment != right.alignment {
                        return false;
                    }
                    if left.name != right.name {
                        return false;
                    }
                    if left.size != right.size {
                        return false;
                    }
                    if left.fields.len() != right.fields.len() {
                        return false;
                    }
                    for (left, right) in std::iter::zip(&left.fields, &right.fields) {
                        match (left, right) {
                            (
                                type_layout::Field::Padding { size },
                                type_layout::Field::Padding { size: size1 },
                            ) => {
                                if size != size1 {
                                    return false;
                                }
                            }
                            (
                                type_layout::Field::Field { name, ty, size },
                                type_layout::Field::Field {
                                    name: name1,
                                    ty: ty1,
                                    size: size1,
                                },
                            ) => {
                                if name != name1 || ty != ty1 || size != size1 {
                                    return false;
                                }
                            }
                            _ => return false,
                        }
                    }
                }
                return true;
            }
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
