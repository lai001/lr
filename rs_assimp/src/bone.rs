use crate::{
    convert::{ConvertToMat4, ConvertToString},
    node::Node,
    vertex_weight::VertexWeight,
};
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    marker::PhantomData,
    rc::Rc,
};

pub struct Bone<'a> {
    c: &'a mut russimp_sys::aiBone,
    pub name: String,
    pub offset_matrix: glam::Mat4,
    pub weights: Vec<VertexWeight>,
    pub node: Option<Rc<RefCell<Node<'a>>>>,
    pub armature: Option<Rc<RefCell<Node<'a>>>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Bone<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiBone) -> Bone<'a> {
        let name = c.mName.into();
        let offset_matrix = c.mOffsetMatrix.to_mat4();
        let ai_weights = unsafe { std::slice::from_raw_parts(c.mWeights, c.mNumWeights as _) };
        let weights = ai_weights.iter().map(|x| VertexWeight::new(x)).collect();
        Bone {
            c,
            weights,
            name,
            marker: PhantomData,
            offset_matrix,
            node: None,
            armature: None,
        }
    }

    pub fn execute(&mut self, map: &mut HashMap<String, Rc<RefCell<Node<'a>>>>) {
        match unsafe { self.c.mArmature.as_mut() } {
            Some(ai_armature) => {
                let mut h = DefaultHasher::new();
                format!("{:p}", ai_armature).hash(&mut h);
                let key = h.finish().to_string();
                match map.get(&key) {
                    Some(node) => {
                        self.armature = Some(node.clone());
                    }
                    None => {
                        self.armature = None;
                    }
                }
            }
            None => {
                self.armature = None;
            }
        }

        match unsafe { self.c.mNode.as_mut() } {
            Some(ai_node) => {
                let mut h = DefaultHasher::new();
                format!("{:p}", ai_node).hash(&mut h);
                let key = h.finish().to_string();
                match map.get(&key) {
                    Some(node) => {
                        self.node = Some(node.clone());
                    }
                    None => {
                        self.node = None;
                    }
                }
            }
            None => {
                self.node = None;
            }
        }
    }
}
