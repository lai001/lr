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
    _ai_metadata: &'a mut russimp_sys::aiMetadata,
    pub values: Vec<EMetadataType>,
    pub keys: Vec<String>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Metadata<'a> {
    pub fn borrow_from(ai_metadata: &'a mut russimp_sys::aiMetadata) -> Metadata<'a> {
        const AIMETADATATYPE_AI_BOOL: i32 = aiMetadataType_AI_BOOL;
        const AIMETADATATYPE_AI_INT32: i32 = aiMetadataType_AI_INT32;
        const AIMETADATATYPE_AI_UINT64: i32 = aiMetadataType_AI_UINT64;
        const AIMETADATATYPE_AI_FLOAT: i32 = aiMetadataType_AI_FLOAT;
        const AIMETADATATYPE_AI_DOUBLE: i32 = aiMetadataType_AI_DOUBLE;
        const AIMETADATATYPE_AI_AISTRING: i32 = aiMetadataType_AI_AISTRING;
        const AIMETADATATYPE_AI_AIVECTOR3D: i32 = aiMetadataType_AI_AIVECTOR3D;
        const AIMETADATATYPE_AI_AIMETADATA: i32 = aiMetadataType_AI_AIMETADATA;
        const AIMETADATATYPE_AI_INT64: i32 = aiMetadataType_AI_INT64;
        const AIMETADATATYPE_AI_UINT32: i32 = aiMetadataType_AI_UINT32;

        let keys = unsafe {
            std::slice::from_raw_parts(ai_metadata.mKeys, ai_metadata.mNumProperties as _)
        };
        let keys: Vec<String> = keys.iter().map(|x| x.to_string()).collect();

        let mut values = Vec::new();
        let ai_metadata_entries = unsafe {
            std::slice::from_raw_parts(ai_metadata.mValues, ai_metadata.mNumProperties as _)
        };
        for ai_metadata_entry in ai_metadata_entries {
            let mut metadata_type: Option<EMetadataType> = None;
            let data = ai_metadata_entry.mData;
            match ai_metadata_entry.mType {
                AIMETADATATYPE_AI_BOOL => {
                    let value = data as *mut bool;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Bool(*value));
                    }
                }
                AIMETADATATYPE_AI_INT32 => {
                    let value = data as *mut i32;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Int32(*value));
                    }
                }
                AIMETADATATYPE_AI_UINT64 => {
                    let value = data as *mut u64;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Uint64(*value));
                    }
                }
                AIMETADATATYPE_AI_FLOAT => {
                    let value = data as *mut f32;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Float(*value));
                    }
                }
                AIMETADATATYPE_AI_DOUBLE => {
                    let value = data as *mut f64;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Double(*value));
                    }
                }
                AIMETADATATYPE_AI_AISTRING => {
                    let value = data as *mut aiString;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::String(value.to_string()));
                    }
                }
                AIMETADATATYPE_AI_AIVECTOR3D => {
                    let value = data as *mut aiVector3D;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Vector3D(value.to_vec3()));
                    }
                }
                AIMETADATATYPE_AI_INT64 => {
                    let value = data as *mut i64;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Int64(*value));
                    }
                }
                AIMETADATATYPE_AI_UINT32 => {
                    let value = data as *mut u32;
                    if let Some(value) = unsafe { value.as_ref() } {
                        metadata_type = Some(EMetadataType::Uint32(*value));
                    }
                }
                AIMETADATATYPE_AI_AIMETADATA => {
                    todo!()
                }
                _ => {}
            }
            if let Some(metadata_type) = metadata_type {
                values.push(metadata_type);
            }
        }
        Metadata {
            _ai_metadata: ai_metadata,
            keys,
            values,
            marker: PhantomData,
        }
    }
}
