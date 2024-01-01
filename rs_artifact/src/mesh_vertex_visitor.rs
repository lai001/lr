use crate::type_expected::{GlamVec2Expected, GlamVec3Expected, GlamVec4Expected};
use serde::{
    de::{self, SeqAccess, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::fmt;

pub const VERTEX_COLOR_FIELD: &str = "vertex_color";
pub const POSITION_FIELD: &str = "position";
pub const NORMAL_FIELD: &str = "normal";
pub const TANGENT_FIELD: &str = "tangent";
pub const BITANGENT_FIELD: &str = "bitangent";
pub const TEX_COORD_FIELD: &str = "tex_coord";

pub const FIELDS: &'static [&'static str] = &[
    VERTEX_COLOR_FIELD,
    POSITION_FIELD,
    NORMAL_FIELD,
    TANGENT_FIELD,
    BITANGENT_FIELD,
    TEX_COORD_FIELD,
];

pub const STRUCT_NAME: &str = "MeshVertex";

enum Field {
    VertexColor,
    Position,
    Normal,
    Tangent,
    Bitangent,
    TexCoord,
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(FieldVisitor)
    }
}

struct FieldVisitor;

impl<'de> Visitor<'de> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&format!("one of {:?}", FIELDS))
    }

    fn visit_str<E>(self, value: &str) -> Result<Field, E>
    where
        E: de::Error,
    {
        match value {
            VERTEX_COLOR_FIELD => Ok(Field::VertexColor),
            POSITION_FIELD => Ok(Field::Position),
            NORMAL_FIELD => Ok(Field::Normal),
            TANGENT_FIELD => Ok(Field::Tangent),
            BITANGENT_FIELD => Ok(Field::Bitangent),
            TEX_COORD_FIELD => Ok(Field::TexCoord),
            _ => Err(de::Error::unknown_field(value, FIELDS)),
        }
    }
}

pub struct MeshVertexVisitor;
impl<'de> Visitor<'de> for MeshVertexVisitor {
    type Value = crate::mesh_vertex::MeshVertex;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(STRUCT_NAME)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut vertex_color: Option<glam::Vec4> = None;
        let mut position: Option<glam::Vec3> = None;
        let mut normal: Option<glam::Vec3> = None;
        let mut tangent: Option<glam::Vec3> = None;
        let mut bitangent: Option<glam::Vec3> = None;
        let mut tex_coord: Option<glam::Vec2> = None;

        while let Some(key) = map.next_key()? {
            match key {
                Field::VertexColor => {
                    if vertex_color.is_some() {
                        return Err(de::Error::duplicate_field(VERTEX_COLOR_FIELD));
                    }
                    if let Ok(next_value) = map.next_value() {
                        vertex_color = Some(glam::Vec4::from_array(next_value));
                    } else {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Seq,
                            &GlamVec4Expected {},
                        ));
                    }
                }
                Field::Position => {
                    if position.is_some() {
                        return Err(de::Error::duplicate_field(POSITION_FIELD));
                    }
                    if let Ok(next_value) = map.next_value() {
                        position = Some(glam::Vec3::from_array(next_value));
                    } else {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Seq,
                            &GlamVec3Expected {},
                        ));
                    }
                }
                Field::Normal => {
                    if normal.is_some() {
                        return Err(de::Error::duplicate_field(NORMAL_FIELD));
                    }
                    if let Ok(next_value) = map.next_value() {
                        normal = Some(glam::Vec3::from_array(next_value));
                    } else {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Seq,
                            &GlamVec3Expected {},
                        ));
                    }
                }
                Field::Tangent => {
                    if tangent.is_some() {
                        return Err(de::Error::duplicate_field(TANGENT_FIELD));
                    }
                    if let Ok(next_value) = map.next_value() {
                        tangent = Some(glam::Vec3::from_array(next_value));
                    } else {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Seq,
                            &GlamVec3Expected {},
                        ));
                    }
                }
                Field::Bitangent => {
                    if bitangent.is_some() {
                        return Err(de::Error::duplicate_field(BITANGENT_FIELD));
                    }
                    if let Ok(next_value) = map.next_value() {
                        bitangent = Some(glam::Vec3::from_array(next_value));
                    } else {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Seq,
                            &GlamVec3Expected {},
                        ));
                    }
                }
                Field::TexCoord => {
                    if tex_coord.is_some() {
                        return Err(de::Error::duplicate_field(TEX_COORD_FIELD));
                    }
                    if let Ok(next_value) = map.next_value() {
                        tex_coord = Some(glam::Vec2::from_array(next_value));
                    } else {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Seq,
                            &GlamVec2Expected {},
                        ));
                    }
                }
            }
        }

        let vertex_color =
            vertex_color.ok_or_else(|| de::Error::missing_field(VERTEX_COLOR_FIELD))?;
        let position = position.ok_or_else(|| de::Error::missing_field(POSITION_FIELD))?;
        let normal = normal.ok_or_else(|| de::Error::missing_field(NORMAL_FIELD))?;
        let tangent = tangent.ok_or_else(|| de::Error::missing_field(TANGENT_FIELD))?;
        let bitangent = bitangent.ok_or_else(|| de::Error::missing_field(BITANGENT_FIELD))?;
        let tex_coord = tex_coord.ok_or_else(|| de::Error::missing_field(TEX_COORD_FIELD))?;
        Ok(Self::Value {
            vertex_color,
            position,
            normal,
            tangent,
            bitangent,
            tex_coord,
        })
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let vertex_color = seq
            .next_element::<[f32; 4]>()?
            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
        let position = seq
            .next_element::<[f32; 3]>()?
            .ok_or_else(|| de::Error::invalid_length(1, &self))?;
        let normal = seq
            .next_element::<[f32; 3]>()?
            .ok_or_else(|| de::Error::invalid_length(2, &self))?;
        let tangent = seq
            .next_element::<[f32; 3]>()?
            .ok_or_else(|| de::Error::invalid_length(3, &self))?;
        let bitangent = seq
            .next_element::<[f32; 3]>()?
            .ok_or_else(|| de::Error::invalid_length(4, &self))?;
        let tex_coord = seq
            .next_element::<[f32; 2]>()?
            .ok_or_else(|| de::Error::invalid_length(5, &self))?;
        Ok(Self::Value {
            vertex_color: vertex_color.into(),
            position: position.into(),
            normal: normal.into(),
            tangent: tangent.into(),
            bitangent: bitangent.into(),
            tex_coord: tex_coord.into(),
        })
    }
}
