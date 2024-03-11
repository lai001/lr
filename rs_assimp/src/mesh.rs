use crate::{
    bone::Bone,
    convert::{ConvertToVec3, ConvertToVec4},
    face::Face,
    primitive_type::EPrimitiveType,
};
// use russimp_sys::AI_MAX_NUMBER_OF_TEXTURECOORDS;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

pub struct Mesh<'a> {
    c: &'a mut russimp_sys::aiMesh,
    pub name: String,
    pub bones: Vec<Rc<RefCell<Bone<'a>>>>,
    pub primitive_type: EPrimitiveType,
    pub vertices: Vec<glam::Vec3>,
    pub normals: Vec<glam::Vec3>,
    pub tangents: Vec<glam::Vec3>,
    pub bitangents: Vec<glam::Vec3>,
    // pub texture_coords: [Vec<glam::Vec3>; AI_MAX_NUMBER_OF_TEXTURECOORDS as _],
    pub texture_coords: Vec<Vec<glam::Vec3>>,
    pub colors: Vec<Vec<glam::Vec4>>,
    pub faces: Vec<Face<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Mesh<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiMesh) -> Mesh<'a> {
        let name = c.mName.into();
        let mut bones = Vec::new();
        if c.mBones.is_null() == false {
            let slice = unsafe {
                std::ptr::slice_from_raw_parts(c.mBones, c.mNumBones as usize)
                    .as_ref()
                    .unwrap()
            };
            for bone in slice {
                bones.push(Rc::new(RefCell::new(Bone::borrow_from(unsafe {
                    bone.as_mut().unwrap()
                }))));
            }
        }

        let primitive_type =
            EPrimitiveType::from(c.mPrimitiveTypes as russimp_sys::aiPrimitiveType).unwrap();
        let vertices =
            unsafe { std::slice::from_raw_parts_mut(c.mVertices, c.mNumVertices as usize) };
        let vertices = vertices.iter_mut().map(|x| x.to_vec3()).collect();

        let normals = if c.mNormals == std::ptr::null_mut() {
            vec![]
        } else {
            unsafe { std::slice::from_raw_parts_mut(c.mNormals, c.mNumVertices as usize) }
                .iter_mut()
                .map(|x| x.to_vec3())
                .collect()
        };

        let tangents = if c.mTangents == std::ptr::null_mut() {
            vec![]
        } else {
            unsafe { std::slice::from_raw_parts_mut(c.mTangents, c.mNumVertices as usize) }
                .iter_mut()
                .map(|x| x.to_vec3())
                .collect()
        };

        let bitangents = if c.mBitangents == std::ptr::null_mut() {
            vec![]
        } else {
            unsafe { std::slice::from_raw_parts_mut(c.mBitangents, c.mNumVertices as usize) }
                .iter_mut()
                .map(|x| x.to_vec3())
                .collect()
        };

        let mut texture_coords: Vec<Vec<glam::Vec3>> = vec![];
        for ai_texture_coords in c.mTextureCoords {
            if ai_texture_coords == std::ptr::null_mut() {
            } else {
                let texture_coord = unsafe {
                    std::slice::from_raw_parts_mut(ai_texture_coords, c.mNumVertices as usize)
                }
                .iter_mut()
                .map(|x| x.to_vec3())
                .collect();
                texture_coords.push(texture_coord);
            };
        }

        let mut colors: Vec<Vec<glam::Vec4>> = vec![];
        for ai_colors in c.mColors {
            if ai_colors == std::ptr::null_mut() {
            } else {
                let color =
                    unsafe { std::slice::from_raw_parts_mut(ai_colors, c.mNumVertices as usize) }
                        .iter_mut()
                        .map(|x| x.to_vec4())
                        .collect();
                colors.push(color);
            };
        }

        let faces = unsafe { std::slice::from_raw_parts_mut(c.mFaces, c.mNumFaces as usize) }
            .iter_mut()
            .map(|x| Face::borrow_from(x))
            .collect();

        Mesh {
            c,
            name,
            marker: PhantomData,
            bones,
            primitive_type,
            vertices,
            normals,
            tangents,
            bitangents,
            texture_coords,
            colors,
            faces,
        }
    }
}
