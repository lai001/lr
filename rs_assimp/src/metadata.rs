use crate::convert::{ConvertToString, ConvertToVec3};
use russimp_sys::*;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum EMetadataType {
    Bool(bool),
    Int32(i32),
    Uint64(u64),
    Float(f32),
    Double(f64),
    String(String),
    Vector3D(glam::Vec3),
    Int64(i64),
    Uint32(u32),
}

pub struct Metadata<'a> {
    c: &'a mut russimp_sys::aiMetadata,
    pub values: Vec<EMetadataType>,
    pub keys: Vec<String>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Metadata<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiMetadata) -> Metadata<'a> {
        let keys = unsafe { std::slice::from_raw_parts(c.mKeys, c.mNumProperties as _) };
        let keys: Vec<String> = keys.iter().map(|x| x.to_string()).collect();

        let mut values = Vec::new();
        let ai_metadata_entries =
            unsafe { std::slice::from_raw_parts(c.mValues, c.mNumProperties as _) };
        for ai_metadata_entry in ai_metadata_entries {
            let mut metadata_type: Option<EMetadataType> = None;
            let data = ai_metadata_entry.mData;
            match ai_metadata_entry.mType {
                aiMetadataType_AI_BOOL => {
                    let value = data as *mut bool;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Bool(*value));
                    }
                }
                aiMetadataType_AI_INT32 => {
                    let value = data as *mut i32;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Int32(*value));
                    }
                }
                aiMetadataType_AI_UINT64 => {
                    let value = data as *mut u64;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Uint64(*value));
                    }
                }
                aiMetadataType_AI_FLOAT => {
                    let value = data as *mut f32;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Float(*value));
                    }
                }
                aiMetadataType_AI_DOUBLE => {
                    let value = data as *mut f64;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Double(*value));
                    }
                }
                aiMetadataType_AI_AISTRING => {
                    let value = data as *mut aiString;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::String(value.to_string()));
                    }
                }
                aiMetadataType_AI_AIVECTOR3D => {
                    let value = data as *mut aiVector3D;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Vector3D(value.to_vec3()));
                    }
                }
                aiMetadataType_AI_INT64 => {
                    let value = data as *mut i64;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Int64(*value));
                    }
                }
                aiMetadataType_AI_UINT32 => {
                    let value = data as *mut u32;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Uint32(*value));
                    }
                }
                aiMetadataType_AI_AIMETADATA => {
                    todo!()
                }
                _ => {}
            }
            if let Some(metadata_type) = metadata_type {
                values.push(metadata_type);
            }
        }
        Metadata {
            c,
            keys,
            values,
            marker: PhantomData,
        }
    }
}
