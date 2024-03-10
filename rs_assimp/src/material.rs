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
        match t {
            aiPropertyTypeInfo_aiPTI_Float => Some(EPropertyTypeInfo::FloatArray),
            aiPropertyTypeInfo_aiPTI_Double => Some(EPropertyTypeInfo::DoubleArray),
            aiPropertyTypeInfo_aiPTI_String => Some(EPropertyTypeInfo::String),
            aiPropertyTypeInfo_aiPTI_Integer => Some(EPropertyTypeInfo::IntegerArray),
            aiPropertyTypeInfo_aiPTI_Buffer => Some(EPropertyTypeInfo::Buffer),
            _ => None,
        }
    }
}

pub struct MaterialProperty<'a> {
    c: &'a mut russimp_sys::aiMaterialProperty,
    pub key: String,
    semantic: u32,
    index: u32,
    property_type_info: EPropertyTypeInfo,
    data: Vec<i8>,
    pub value: EPropertyTypeValue,
    marker: PhantomData<&'a ()>,
}

impl<'a> MaterialProperty<'a> {
    pub fn new(
        c: &'a mut aiMaterialProperty,
        ai_material: &russimp_sys::aiMaterial,
    ) -> MaterialProperty<'a> {
        let key = c.mKey.to_string();
        let semantic = c.mSemantic;
        let index = c.mIndex;
        let property_type_info = EPropertyTypeInfo::new(c.mType).unwrap();
        let data = unsafe { std::slice::from_raw_parts(c.mData, c.mDataLength as _).to_vec() };
        let value: EPropertyTypeValue;
        match property_type_info {
            EPropertyTypeInfo::FloatArray => {
                let mut out_vec: std::mem::MaybeUninit<f32> = std::mem::MaybeUninit::uninit();
                let mut p_max: std::mem::MaybeUninit<u32> = std::mem::MaybeUninit::uninit();
                unsafe {
                    let status = aiGetMaterialFloatArray(
                        ai_material,
                        c.mKey.data.as_ptr(),
                        c.mSemantic as _,
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
                        c.mKey.data.as_ptr(),
                        c.mSemantic as _,
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
                        c.mKey.data.as_ptr(),
                        c.mSemantic as _,
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
            c,
            key,
            semantic,
            index,
            property_type_info,
            data,
            value,
            marker: PhantomData,
        }
    }
}

pub struct Material<'a> {
    c: &'a mut russimp_sys::aiMaterial,
    pub num_allocated: u32,
    pub material_properties: Vec<MaterialProperty<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Material<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiMaterial) -> Material<'a> {
        for texture_type in TextureType::iter() {
            unsafe {
                for index in 0..aiGetMaterialTextureCount(c, texture_type as _) {
                    let mut path = aiString {
                        length: 0,
                        data: [0; 1024],
                    };
                    let status = aiGetMaterialTexture(
                        c,
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
            std::ptr::slice_from_raw_parts(c.mProperties, c.mNumProperties as _)
                .as_ref()
                .unwrap()
        };
        let num_allocated: u32 = c.mNumAllocated;
        let mut material_properties: Vec<MaterialProperty> = vec![];
        for property in ai_properties {
            let property = unsafe { property.as_mut().unwrap() };
            material_properties.push(MaterialProperty::new(property, c))
        }
        Material {
            c,
            num_allocated,
            marker: PhantomData,
            material_properties,
        }
    }
}
