use crate::{convert::ConvertToString, texture_type::TextureType};
use russimp_sys::*;
use std::marker::PhantomData;
use strum::IntoEnumIterator;

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum EPropertyTypeInfo {
    FloatArray = aiPropertyTypeInfo_aiPTI_Float,
    DoubleArray = aiPropertyTypeInfo_aiPTI_Double,
    String = aiPropertyTypeInfo_aiPTI_String,
    IntegerArray = aiPropertyTypeInfo_aiPTI_Integer,
    Buffer = aiPropertyTypeInfo_aiPTI_Buffer,
    Force32Bit = aiPropertyTypeInfo__aiPTI_Force32Bit,
}

#[derive(Debug, Clone)]
pub enum EPropertyTypeValue {
    Buffer(Vec<u8>),
    IntegerArray(Vec<i32>),
    FloatArray(Vec<f32>),
    DoubleArray(Vec<f64>),
    String(String),
}

impl EPropertyTypeInfo {
    pub fn new(t: aiPropertyTypeInfo) -> Option<EPropertyTypeInfo> {
        const AIPROPERTYTYPEINFO_AIPTI_FLOAT: i32 = aiPropertyTypeInfo_aiPTI_Float;
        const AIPROPERTYTYPEINFO_AIPTI_DOUBLE: i32 = aiPropertyTypeInfo_aiPTI_Double;
        const AIPROPERTYTYPEINFO_AIPTI_STRING: i32 = aiPropertyTypeInfo_aiPTI_String;
        const AIPROPERTYTYPEINFO_AIPTI_INTEGER: i32 = aiPropertyTypeInfo_aiPTI_Integer;
        const AIPROPERTYTYPEINFO_AIPTI_BUFFER: i32 = aiPropertyTypeInfo_aiPTI_Buffer;
        match t {
            AIPROPERTYTYPEINFO_AIPTI_FLOAT => Some(EPropertyTypeInfo::FloatArray),
            AIPROPERTYTYPEINFO_AIPTI_DOUBLE => Some(EPropertyTypeInfo::DoubleArray),
            AIPROPERTYTYPEINFO_AIPTI_STRING => Some(EPropertyTypeInfo::String),
            AIPROPERTYTYPEINFO_AIPTI_INTEGER => Some(EPropertyTypeInfo::IntegerArray),
            AIPROPERTYTYPEINFO_AIPTI_BUFFER => Some(EPropertyTypeInfo::Buffer),
            _ => None,
        }
    }
}

pub struct MaterialProperty<'a> {
    _ai_material_property: &'a mut russimp_sys::aiMaterialProperty,
    pub key: String,
    _semantic: u32,
    _index: u32,
    _property_type_info: EPropertyTypeInfo,
    _data: Vec<i8>,
    pub value: EPropertyTypeValue,
    marker: PhantomData<&'a ()>,
}

impl<'a> MaterialProperty<'a> {
    pub fn new(
        ai_material_property: &'a mut aiMaterialProperty,
        ai_material: &russimp_sys::aiMaterial,
    ) -> MaterialProperty<'a> {
        let key = ai_material_property.mKey.to_string();
        let semantic = ai_material_property.mSemantic;
        let index = ai_material_property.mIndex;
        let property_type_info = EPropertyTypeInfo::new(ai_material_property.mType).unwrap();
        let data = unsafe {
            std::slice::from_raw_parts(
                ai_material_property.mData,
                ai_material_property.mDataLength as _,
            )
            .to_vec()
        };
        let value: EPropertyTypeValue;
        match property_type_info {
            EPropertyTypeInfo::FloatArray => {
                let mut out_vec: std::mem::MaybeUninit<f32> = std::mem::MaybeUninit::uninit();
                let mut p_max: std::mem::MaybeUninit<u32> = std::mem::MaybeUninit::uninit();
                unsafe {
                    let status = aiGetMaterialFloatArray(
                        ai_material,
                        ai_material_property.mKey.data.as_ptr(),
                        ai_material_property.mSemantic as _,
                        index,
                        out_vec.as_mut_ptr(),
                        p_max.as_mut_ptr(),
                    );
                    assert_eq!(aiReturn_aiReturn_SUCCESS, status);
                };
                let array = unsafe {
                    std::slice::from_raw_parts(out_vec.as_ptr(), p_max.assume_init() as _)
                };
                value = EPropertyTypeValue::FloatArray(array.to_vec());
            }
            EPropertyTypeInfo::DoubleArray => {
                todo!()
            }
            EPropertyTypeInfo::String => {
                let mut out_string: std::mem::MaybeUninit<aiString> =
                    std::mem::MaybeUninit::uninit();
                unsafe {
                    let status = aiGetMaterialString(
                        ai_material,
                        ai_material_property.mKey.data.as_ptr(),
                        ai_material_property.mSemantic as _,
                        index,
                        out_string.as_mut_ptr(),
                    );
                    assert_eq!(aiReturn_aiReturn_SUCCESS, status);
                };
                let out_string = unsafe { out_string.assume_init() };
                value = EPropertyTypeValue::String(out_string.to_string());
            }
            EPropertyTypeInfo::IntegerArray => {
                let mut out_vec: std::mem::MaybeUninit<i32> = std::mem::MaybeUninit::uninit();
                let mut p_max: std::mem::MaybeUninit<u32> = std::mem::MaybeUninit::uninit();
                unsafe {
                    let status = aiGetMaterialIntegerArray(
                        ai_material,
                        ai_material_property.mKey.data.as_ptr(),
                        ai_material_property.mSemantic as _,
                        index,
                        out_vec.as_mut_ptr(),
                        p_max.as_mut_ptr(),
                    );
                    assert_eq!(aiReturn_aiReturn_SUCCESS, status);
                };
                let array = unsafe {
                    std::slice::from_raw_parts(out_vec.as_ptr(), p_max.assume_init() as _)
                };
                value = EPropertyTypeValue::IntegerArray(array.to_vec());
            }
            EPropertyTypeInfo::Buffer => {
                value = EPropertyTypeValue::Buffer(vec![]);
            }
            EPropertyTypeInfo::Force32Bit => {
                todo!()
            }
        }
        MaterialProperty {
            _ai_material_property: ai_material_property,
            key,
            _semantic: semantic,
            _index: index,
            _property_type_info: property_type_info,
            _data: data,
            value,
            marker: PhantomData,
        }
    }
}

pub struct Material<'a> {
    _ai_material: &'a mut russimp_sys::aiMaterial,
    pub num_allocated: u32,
    pub material_properties: Vec<MaterialProperty<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Material<'a> {
    pub fn borrow_from(ai_material: &'a mut russimp_sys::aiMaterial) -> Material<'a> {
        for texture_type in TextureType::iter() {
            unsafe {
                for index in 0..aiGetMaterialTextureCount(ai_material, texture_type as _) {
                    let mut path = aiString {
                        length: 0,
                        data: [0; 1024],
                    };
                    let status = aiGetMaterialTexture(
                        ai_material,
                        texture_type as _,
                        index,
                        &mut path,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    );
                    assert_eq!(aiReturn_aiReturn_SUCCESS, status);
                    // aiGetMaterialProperty(c, pKey, texture_type as _, index, pPropOut);
                    // println!("{}", path.to_string());
                }
            }
        }
        let ai_properties = unsafe {
            std::ptr::slice_from_raw_parts(ai_material.mProperties, ai_material.mNumProperties as _)
                .as_ref()
                .unwrap()
        };
        let num_allocated: u32 = ai_material.mNumAllocated;
        let mut material_properties: Vec<MaterialProperty> = vec![];
        for property in ai_properties {
            let property = unsafe { property.as_mut().unwrap() };
            material_properties.push(MaterialProperty::new(property, ai_material))
        }
        Material {
            _ai_material: ai_material,
            num_allocated,
            marker: PhantomData,
            material_properties,
        }
    }
}
