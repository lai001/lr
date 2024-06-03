use crate::debug_log;
use proc_macro::TokenStream;
use quote::quote;

fn get_dimension(vector_size: &naga::VectorSize) -> usize {
    match vector_size {
        naga::VectorSize::Bi => 2,
        naga::VectorSize::Tri => 3,
        naga::VectorSize::Quad => 4,
    }
}

fn cal_align_of(row: isize, width: isize) -> isize {
    rs_foundation::next_highest_power_of_two(row * width)
}

fn fill(current_offset: usize, align: usize) -> usize {
    (rs_foundation::alignment(current_offset as isize, align as isize) as usize) - current_offset
}

// trait Alignment {
//     fn align_of(&self) -> usize;
//     fn size_of(&self) -> usize;
// }

trait WgslAlignment {
    fn align_of(&self, module: &naga::Module) -> usize;
    fn size_of(&self, module: &naga::Module) -> usize;
}

impl WgslAlignment for naga::Scalar {
    fn align_of(&self, module: &naga::Module) -> usize {
        self.size_of(module)
    }

    fn size_of(&self, _: &naga::Module) -> usize {
        match self.kind {
            naga::ScalarKind::Sint => self.width as usize,
            naga::ScalarKind::Uint => self.width as usize,
            naga::ScalarKind::Float => self.width as usize,
            naga::ScalarKind::Bool => todo!(),
            naga::ScalarKind::AbstractInt => todo!(),
            naga::ScalarKind::AbstractFloat => todo!(),
        }
    }
}

impl WgslAlignment for naga::TypeInner {
    fn align_of(&self, module: &naga::Module) -> usize {
        match self {
            naga::TypeInner::Scalar(scalar) => scalar.align_of(module),
            naga::TypeInner::Vector { size, scalar } => {
                cal_align_of(get_dimension(size) as isize, scalar.width as isize) as usize
            }
            naga::TypeInner::Matrix { rows, scalar, .. } => {
                cal_align_of(get_dimension(rows) as isize, scalar.width as isize) as usize
            }
            naga::TypeInner::Atomic(_) => todo!(),
            naga::TypeInner::Pointer { .. } => todo!(),
            naga::TypeInner::ValuePointer { .. } => todo!(),
            naga::TypeInner::Array { stride, .. } => *stride as usize,
            naga::TypeInner::Struct { members, .. } => {
                let mut struct_align: usize = 0;
                for member in members {
                    let type_inner = &module.types.get_handle(member.ty).unwrap().inner;
                    struct_align = struct_align.max(type_inner.align_of(module));
                }
                struct_align
            }
            naga::TypeInner::Image { .. } => todo!(),
            naga::TypeInner::Sampler { .. } => todo!(),
            naga::TypeInner::AccelerationStructure => todo!(),
            naga::TypeInner::RayQuery => todo!(),
            naga::TypeInner::BindingArray { .. } => todo!(),
        }
    }

    fn size_of(&self, module: &naga::Module) -> usize {
        match self {
            naga::TypeInner::Scalar(scalar) => scalar.size_of(module),
            naga::TypeInner::Vector { size, scalar } => {
                scalar.size_of(module) * get_dimension(size)
            }
            naga::TypeInner::Matrix { columns, .. } => {
                get_dimension(columns) * self.align_of(module)
            }
            naga::TypeInner::Atomic(_) => todo!(),
            naga::TypeInner::Pointer { .. } => todo!(),
            naga::TypeInner::ValuePointer { .. } => todo!(),
            naga::TypeInner::Array { size, stride, .. } => match size {
                naga::ArraySize::Constant(len) => len.get() as usize * *stride as usize,
                naga::ArraySize::Dynamic => todo!(),
            },
            naga::TypeInner::Struct { members, .. } => {
                if members.is_empty() {
                    0
                } else {
                    let self_align = self.align_of(module);
                    let mut offset: usize = 0;
                    for member in members {
                        let type_inner = &module.types.get_handle(member.ty).unwrap().inner;
                        let member_align = type_inner.align_of(module);
                        let member_size = type_inner.size_of(module);
                        offset = offset + member_size + fill(offset, member_align);
                    }
                    offset + fill(offset, self_align)
                }
            }
            naga::TypeInner::Image { .. } => todo!(),
            naga::TypeInner::Sampler { .. } => todo!(),
            naga::TypeInner::AccelerationStructure => todo!(),
            naga::TypeInner::RayQuery => todo!(),
            naga::TypeInner::BindingArray { .. } => todo!(),
        }
    }
}

fn tab_str(level: u32) -> String {
    " ".repeat(4 * level as usize).to_string()
}

fn dump(module: &naga::Module, ty: &naga::Type, p_offset: usize, level: u32) -> (String, usize) {
    let name = ty.name.clone().unwrap_or("".to_string());
    match &ty.inner {
        naga::TypeInner::Struct { members, .. } => {
            if members.is_empty() {
                return (String::from(""), 0);
            } else {
                let self_align = ty.inner.align_of(module);
                let self_size = ty.inner.size_of(module);
                let mut offset: usize = 0;

                let mut message = tab_str(level)
                    + &String::from(&name)
                    + ": "
                    + &format!("align({})  size({})", self_align, self_size)
                    + "\n";
                for item in members {
                    let member_name = item.name.as_ref().cloned().unwrap_or("".to_string());
                    let ty = module.types.get_handle(item.ty).unwrap();
                    let type_inner = &ty.inner;
                    let member_align = type_inner.align_of(module);
                    let member_size = type_inner.size_of(module);
                    let fill_size = fill(offset, member_align);
                    if fill_size > 0 {
                        message = message + &tab_str(level + 1) + &format!("name(implicit struct size padding) offset({} + {})                  size({})\n", p_offset, offset, fill_size);
                    }

                    if let naga::TypeInner::Struct { .. } = &type_inner {
                        let (mes, _) = dump(module, ty, offset, level + 1);
                        message = message + &mes;
                    } else {
                        message = message
                            + &tab_str(level + 1)
                            + &format!(
                                "name({}) offset({} + {})     align({})    size({})\n",
                                member_name, p_offset, offset, member_align, member_size
                            );
                    }

                    offset = offset + member_size + fill_size;
                }
                let fill_size = fill(offset, self_align);
                if fill_size > 0 {
                    message = message
                        + &tab_str(level + 1)
                        + &format!(
                            "name(implicit struct size padding) offset({} + {})                  size({})\n",
                            p_offset, offset, fill_size
                        );
                }
                (message, offset + fill(offset, self_align))
            }
        }
        _ => {
            panic!()
        }
    }
}

pub fn shader_uniform_macro_impl(input: TokenStream) -> TokenStream {
    let module = naga::front::wgsl::parse_str(&input.to_string());

    match module {
        Ok(module) => {
            let mut type_structs = module.types.iter().filter(|(_, ty)| match &ty.inner {
                naga::TypeInner::Struct { .. } => true,
                _ => false,
            });
            let mut type_structs_count = 0;
            while let Some(type_struct) = type_structs.next() {
                type_structs_count += 1;
                assert_eq!(type_structs_count, 1);

                let (message, _) = dump(&module, type_struct.1, 0, 0);
                debug_log("shader_uniform", &format!("{}", message));
            }
        }
        Err(err) => {
            debug_log("shader_uniform", &format!("{:?}", err));
        }
    }
    let output_stream = quote! {};

    output_stream.into()
}
