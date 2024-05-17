use crate::{
    bone::Bone,
    convert::{ConvertToVec3, ConvertToVec4},
    face::Face,
    node::Node,
    primitive_type::EPrimitiveType,
};
// use russimp_sys::AI_MAX_NUMBER_OF_TEXTURECOORDS;
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

pub struct Mesh<'a> {
    _ai_mesh: &'a mut russimp_sys::aiMesh,
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
    pub fn borrow_from(
        ai_mesh: &'a mut russimp_sys::aiMesh,
        map: &mut HashMap<String, Rc<RefCell<Node<'a>>>>,
    ) -> Mesh<'a> {
        let name = ai_mesh.mName.into();
        let mut bones = Vec::new();
        if ai_mesh.mBones.is_null() == false {
            let slice = unsafe {
                std::ptr::slice_from_raw_parts(ai_mesh.mBones, ai_mesh.mNumBones as usize)
                    .as_ref()
                    .unwrap()
            };
            for bone in slice {
                bones.push(Rc::new(RefCell::new(Bone::borrow_from(
                    unsafe { bone.as_mut().unwrap() },
                    map,
                ))));
            }
        }

        let primitive_type = (ai_mesh.mPrimitiveTypes as russimp_sys::aiPrimitiveType)
            .try_into()
            .unwrap();
        let vertices = unsafe {
            std::slice::from_raw_parts_mut(ai_mesh.mVertices, ai_mesh.mNumVertices as usize)
        };
        let vertices = vertices.iter_mut().map(|x| x.to_vec3()).collect();

        let normals = if ai_mesh.mNormals == std::ptr::null_mut() {
            vec![]
        } else {
            unsafe {
                std::slice::from_raw_parts_mut(ai_mesh.mNormals, ai_mesh.mNumVertices as usize)
            }
            .iter_mut()
            .map(|x| x.to_vec3())
            .collect()
        };

        let tangents = if ai_mesh.mTangents == std::ptr::null_mut() {
            vec![]
        } else {
            unsafe {
                std::slice::from_raw_parts_mut(ai_mesh.mTangents, ai_mesh.mNumVertices as usize)
            }
            .iter_mut()
            .map(|x| x.to_vec3())
            .collect()
        };

        let bitangents = if ai_mesh.mBitangents == std::ptr::null_mut() {
            vec![]
        } else {
            unsafe {
                std::slice::from_raw_parts_mut(ai_mesh.mBitangents, ai_mesh.mNumVertices as usize)
            }
            .iter_mut()
            .map(|x| x.to_vec3())
            .collect()
        };

        let mut texture_coords: Vec<Vec<glam::Vec3>> = vec![];
        for ai_texture_coords in ai_mesh.mTextureCoords {
            if ai_texture_coords == std::ptr::null_mut() {
            } else {
                let texture_coord = unsafe {
                    std::slice::from_raw_parts_mut(ai_texture_coords, ai_mesh.mNumVertices as usize)
                }
                .iter_mut()
                .map(|x| x.to_vec3())
                .collect();
                texture_coords.push(texture_coord);
            };
        }

        let mut colors: Vec<Vec<glam::Vec4>> = vec![];
        for ai_colors in ai_mesh.mColors {
            if ai_colors == std::ptr::null_mut() {
            } else {
                let color = unsafe {
                    std::slice::from_raw_parts_mut(ai_colors, ai_mesh.mNumVertices as usize)
                }
                .iter_mut()
                .map(|x| x.to_vec4())
                .collect();
                colors.push(color);
            };
        }

        let faces =
            unsafe { std::slice::from_raw_parts_mut(ai_mesh.mFaces, ai_mesh.mNumFaces as usize) }
                .iter_mut()
                .map(|x| Face::borrow_from(x))
                .collect();

        Mesh {
            _ai_mesh: ai_mesh,
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
