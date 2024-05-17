use crate::node::{get_node_path, Node};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

pub struct SkeletonBone<'a> {
    _ai_skeleton_bone: &'a mut russimp_sys::aiSkeletonBone,
    pub armature: Option<Rc<RefCell<Node<'a>>>>,
    pub node: Option<Rc<RefCell<Node<'a>>>>,

    marker: PhantomData<&'a ()>,
}

impl<'a> SkeletonBone<'a> {
    pub fn borrow_from(
        ai_skeleton_bone: &'a mut russimp_sys::aiSkeletonBone,
        map: &HashMap<String, Rc<RefCell<Node<'a>>>>,
    ) -> SkeletonBone<'a> {
        let armature = match unsafe { ai_skeleton_bone.mArmature.as_mut() } {
            Some(ai_armature) => match map.get(&get_node_path(ai_armature)) {
                Some(node) => Some(node.clone()),
                None => None,
            },
            None => None,
        };

        let node = match unsafe { ai_skeleton_bone.mNode.as_mut() } {
            Some(ai_node) => match map.get(&get_node_path(ai_node)) {
                Some(node) => Some(node.clone()),
                None => None,
            },
            None => None,
        };

        SkeletonBone {
            _ai_skeleton_bone: ai_skeleton_bone,
            marker: PhantomData,
            armature,
            node,
        }
    }
}
