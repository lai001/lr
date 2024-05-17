use crate::{
    convert::ConvertToMat4,
    node::{self, Node},
    vertex_weight::VertexWeight,
};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

pub struct Bone<'a> {
    _ai_bone: &'a mut russimp_sys::aiBone,
    pub name: String,
    pub offset_matrix: glam::Mat4,
    pub weights: Vec<VertexWeight>,
    pub node: Option<Rc<RefCell<Node<'a>>>>,
    pub armature: Option<Rc<RefCell<Node<'a>>>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Bone<'a> {
    pub fn borrow_from(
        ai_bone: &'a mut russimp_sys::aiBone,
        map: &mut HashMap<String, Rc<RefCell<Node<'a>>>>,
    ) -> Bone<'a> {
        let name = ai_bone.mName.into();
        let offset_matrix = ai_bone.mOffsetMatrix.to_mat4();
        let ai_weights =
            unsafe { std::slice::from_raw_parts(ai_bone.mWeights, ai_bone.mNumWeights as _) };
        let weights = ai_weights.iter().map(|x| VertexWeight::new(x)).collect();

        let armature = match unsafe { ai_bone.mArmature.as_mut() } {
            Some(ai_armature) => {
                let path = node::get_node_path(ai_armature);
                match map.get(&path) {
                    Some(node) => Some(node.clone()),
                    None => None,
                }
            }
            None => None,
        };

        let node = match unsafe { ai_bone.mNode.as_mut() } {
            Some(ai_node) => {
                let path = node::get_node_path(ai_node);
                match map.get(&path) {
                    Some(node) => Some(node.clone()),
                    None => None,
                }
            }
            None => None,
        };

        Bone {
            _ai_bone: ai_bone,
            weights,
            name,
            marker: PhantomData,
            offset_matrix,
            node,
            armature,
        }
    }
}
