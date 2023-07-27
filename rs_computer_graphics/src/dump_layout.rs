enum EDataType {
    Int32,
    Uint32,
    Float32,
    Float16,
    Vec2int32,
    Vec3f32,
    Mat4x4f32,
    Struct(WGStruct),
}

impl EDataType {
    fn align_of(&self) -> usize {
        match self {
            EDataType::Int32 => 4,
            EDataType::Uint32 => 4,
            EDataType::Float32 => 4,
            EDataType::Vec2int32 => 8,
            EDataType::Float16 => 2,
            EDataType::Struct(s) => s.align_of(),
            EDataType::Mat4x4f32 => 16,
            EDataType::Vec3f32 => 16,
        }
    }
    fn size_of(&self) -> usize {
        match self {
            EDataType::Int32 => 4,
            EDataType::Uint32 => 4,
            EDataType::Float32 => 4,
            EDataType::Vec2int32 => 8,
            EDataType::Float16 => 2,
            EDataType::Struct(s) => s.size_of(),
            EDataType::Mat4x4f32 => 64,
            EDataType::Vec3f32 => 12,
        }
    }
}

struct WGStruct {
    pub name: String,
    pub members: Vec<WGStructMember>,
}

struct WGStructMember {
    pub name: String,
    pub data_type: EDataType,
}

impl WGStruct {
    fn align_of(&self) -> usize {
        let mut struct_align: usize = 0;
        for item in &self.members {
            struct_align = struct_align.max(item.data_type.align_of());
        }
        struct_align
    }

    fn size_of(&self) -> usize {
        if self.members.is_empty() {
            0
        } else {
            let self_align = self.align_of();
            let mut offset: usize = 0;
            for item in &self.members {
                let member_align = item.data_type.align_of();
                let member_size = item.data_type.size_of();
                offset = offset + member_size + Self::fill(offset, member_align);
            }
            offset + Self::fill(offset, self_align)
        }
    }

    fn fill(current_offset: usize, align: usize) -> usize {
        (crate::util::alignment(current_offset as isize, align as isize) as usize) - current_offset
    }

    fn dump(&self, p_offset: usize, level: u32) -> (String, usize) {
        if self.members.is_empty() {
            return (String::from(""), 0);
        } else {
            let self_align = self.align_of();
            let self_size = self.size_of();
            let mut offset: usize = 0;

            let mut message = Self::tab_str(level)
                + &String::from(&self.name)
                + ": "
                + &format!("align({})  size({})", self_align, self_size)
                + "\n";
            for item in &self.members {
                let member_name = &item.name;
                let member_align = item.data_type.align_of();
                let member_size = item.data_type.size_of();
                let fill_size = Self::fill(offset, member_align);
                if fill_size > 0 {
                    message = message + &Self::tab_str(level + 1) + &format!("name(implicit struct size padding) offset({} + {})                  size({})\n", p_offset, offset, fill_size);
                }

                if let EDataType::Struct(s) = &item.data_type {
                    let (mes, size) = s.dump(offset, level + 1);
                    message = message + &mes;
                } else {
                    message = message
                        + &Self::tab_str(level + 1)
                        + &format!(
                            "name({}) offset({} + {})     align({})    size({})\n",
                            member_name, p_offset, offset, member_align, member_size
                        );
                }

                offset = offset + member_size + fill_size;
            }
            let fill_size = Self::fill(offset, self_align);
            if fill_size > 0 {
                message = message
                    + &Self::tab_str(level + 1)
                    + &format!(
                        "name(implicit struct size padding) offset({} + {})                  size({})\n",
                        p_offset, offset, fill_size
                    );
            }
            (message, offset + Self::fill(offset, self_align))
        }
    }

    fn tab_str(level: u32) -> String {
        let mut str = String::from("");
        for _ in 0..level {
            str = str + "    ";
        }
        str
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_case() {
        let spot_light = WGStruct {
            name: "SpotLight".to_string(),
            members: vec![
                WGStructMember {
                    name: "position".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "direction".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "ambient".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "diffuse".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "specular".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "cut_off".to_string(),
                    data_type: EDataType::Float32,
                },
                WGStructMember {
                    name: "outer_cut_off".to_string(),
                    data_type: EDataType::Float32,
                },
                WGStructMember {
                    name: "constant".to_string(),
                    data_type: EDataType::Float32,
                },
                WGStructMember {
                    name: "linear".to_string(),
                    data_type: EDataType::Float32,
                },
                WGStructMember {
                    name: "quadratic".to_string(),
                    data_type: EDataType::Float32,
                },
            ],
        };

        let point_light = WGStruct {
            name: "PointLight".to_string(),
            members: vec![
                WGStructMember {
                    name: "position".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "ambient".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "diffuse".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "specular".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "constant".to_string(),
                    data_type: EDataType::Float32,
                },
                WGStructMember {
                    name: "linear".to_string(),
                    data_type: EDataType::Float32,
                },
                WGStructMember {
                    name: "quadratic".to_string(),
                    data_type: EDataType::Float32,
                },
            ],
        };

        let directional_light = WGStruct {
            name: "DirectionalLight".to_string(),
            members: vec![
                WGStructMember {
                    name: "direction".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "ambient".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "diffuse".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "specular".to_string(),
                    data_type: EDataType::Vec3f32,
                },
            ],
        };

        let constants = WGStruct {
            name: "Constants".to_string(),
            members: vec![
                WGStructMember {
                    name: "directional_light".to_string(),
                    data_type: EDataType::Struct(directional_light),
                },
                WGStructMember {
                    name: "point_light".to_string(),
                    data_type: EDataType::Struct(point_light),
                },
                WGStructMember {
                    name: "spot_light".to_string(),
                    data_type: EDataType::Struct(spot_light),
                },
                WGStructMember {
                    name: "model".to_string(),
                    data_type: EDataType::Mat4x4f32,
                },
                WGStructMember {
                    name: "view".to_string(),
                    data_type: EDataType::Mat4x4f32,
                },
                WGStructMember {
                    name: "projection".to_string(),
                    data_type: EDataType::Mat4x4f32,
                },
                WGStructMember {
                    name: "view_position".to_string(),
                    data_type: EDataType::Vec3f32,
                },
                WGStructMember {
                    name: "roughness_factor".to_string(),
                    data_type: EDataType::Float32,
                },
                WGStructMember {
                    name: "metalness_factor".to_string(),
                    data_type: EDataType::Float32,
                },
            ],
        };

        let (message, size) = constants.dump(0, 0);
        println!("{}\n{}", message, size);
    }
}
