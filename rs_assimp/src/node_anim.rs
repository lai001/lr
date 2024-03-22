use crate::{anim_behaviour::EAnimBehaviour, node::Node, quat_key::QuatKey, vector_key::VectorKey};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

pub struct NodeAnim<'a> {
    c: &'a mut russimp_sys::aiNodeAnim,
    pub node: Option<Rc<RefCell<Node<'a>>>>,
    pub pre_state: EAnimBehaviour,
    pub post_state: EAnimBehaviour,
    pub position_keys: Vec<VectorKey<'a>>,
    pub scaling_keys: Vec<VectorKey<'a>>,
    pub rotation_keys: Vec<QuatKey<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> NodeAnim<'a> {
    pub fn borrow_from(
        c: &'a mut russimp_sys::aiNodeAnim,
        map: &mut HashMap<String, Rc<RefCell<Node<'a>>>>,
    ) -> NodeAnim<'a> {
        let node_name: String = c.mNodeName.into();
        let pre_state = c.mPreState;
        let post_state = c.mPostState;
        let position_keys =
            unsafe { std::slice::from_raw_parts_mut(c.mPositionKeys, c.mNumPositionKeys as _) };
        let position_keys = position_keys
            .iter_mut()
            .map(|x| VectorKey::borrow_from(x))
            .collect();

        let scaling_keys =
            unsafe { std::slice::from_raw_parts_mut(c.mScalingKeys, c.mNumScalingKeys as _) };
        let scaling_keys = scaling_keys
            .iter_mut()
            .map(|x| VectorKey::borrow_from(x))
            .collect();

        let rotation_keys =
            unsafe { std::slice::from_raw_parts_mut(c.mRotationKeys, c.mNumRotationKeys as _) };
        let rotation_keys = rotation_keys
            .iter_mut()
            .map(|x| QuatKey::borrow_from(x))
            .collect();

        let node = (|| {
            for node in map.values() {
                if node.borrow().name == node_name {
                    return Some(node.clone());
                }
            }
            None
        })();
        NodeAnim {
            c,
            node,
            pre_state: pre_state.try_into().unwrap(),
            post_state: post_state.try_into().unwrap(),
            position_keys,
            scaling_keys,
            rotation_keys,
            marker: PhantomData,
        }
    }
}
