use crate::{
    convert::{ConvertToMat4, ConvertToString},
    mesh::Mesh,
    metadata::Metadata,
};
use russimp_sys::aiNode;
use std::{
    cell::RefCell,
    collections::HashMap,
    marker::PhantomData,
    rc::{Rc, Weak},
};

pub fn get_node_path(ai_node: &mut aiNode) -> String {
    let mut parent: Option<&mut aiNode> = unsafe { ai_node.mParent.as_mut() };
    let mut path = format!("/{}", ai_node.mName.to_string());
    while let Some(ref _parent) = parent {
        path = format!("/{}{}", _parent.mName.to_string(), path);
        parent = unsafe { _parent.mParent.as_mut() };
    }
    path
}

#[derive(Debug, PartialEq, Eq)]
pub enum ENodeType {
    Axis,
    Bone,
    Mesh,
    Armature,
}

pub struct Node<'a> {
    c: &'a mut russimp_sys::aiNode,
    pub parent: Option<Weak<RefCell<Node<'a>>>>,
    pub name: String,
    pub path: String,
    pub children: Vec<Rc<RefCell<Node<'a>>>>,
    pub metadata: Option<Metadata<'a>>,
    pub transformation: glam::Mat4,
    pub meshes: Vec<Rc<RefCell<Mesh<'a>>>>,
    pub(crate) is_bone: bool,
    pub bone_offset_matrix: Option<glam::Mat4>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Node<'a> {
    pub fn new(c: &'a mut russimp_sys::aiNode, path: String) -> Node<'a> {
        let name = c.mName.to_string();
        let metadata = match unsafe { c.mMetaData.as_mut() } {
            Some(m_meta_data) => Some(Metadata::borrow_from(m_meta_data)),
            None => None,
        };
        let transformation = c.mTransformation.to_mat4();

        Node {
            c,
            meshes: vec![],
            parent: None,
            name,
            path,
            children: vec![],
            metadata,
            transformation,
            marker: PhantomData,
            is_bone: false,
            bone_offset_matrix: None,
        }
    }

    pub fn parent_and_leaf(&mut self, map: &mut HashMap<String, Rc<RefCell<Node<'a>>>>) {
        match unsafe { self.c.mParent.as_mut() } {
            Some(_) => match map.get_mut(&self.path) {
                Some(parent_node) => {
                    self.parent = Some(Rc::downgrade(parent_node));
                }
                None => {}
            },
            None => {}
        }
        self.children.clear();
        let children = rs_foundation::get_vec_from_raw_mut(self.c.mChildren, self.c.mNumChildren);
        for item in children {
            match map.get_mut(&get_node_path(item)) {
                Some(node) => {
                    self.children.push(node.clone());
                }
                None => {}
            }
        }
    }

    pub fn update_meshes(&mut self, all_meshes: Vec<Rc<RefCell<Mesh<'a>>>>) {
        let c = &self.c;
        let ai_meshes: Vec<usize> =
            unsafe { std::slice::from_raw_parts_mut(c.mMeshes, c.mNumMeshes as _) }
                .iter()
                .map(|x| *x as usize)
                .collect();
        self.meshes.clear();
        for ai_meshe in ai_meshes {
            self.meshes.push(all_meshes[ai_meshe].clone());
        }
    }

    pub fn get_node_type(&self) -> ENodeType {
        if !self.meshes.is_empty() {
            ENodeType::Mesh
        } else if self.is_bone {
            if self.bone_offset_matrix.is_none() {
                ENodeType::Armature
            } else {
                ENodeType::Bone
            }
        } else {
            ENodeType::Axis
        }
    }
}
