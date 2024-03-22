use crate::{
    convert::ConvertToString,
    node::{get_node_path, Node},
};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

pub struct SkeletonBone<'a> {
    c: &'a mut russimp_sys::aiSkeletonBone,
    pub armature: Option<Rc<RefCell<Node<'a>>>>,
    pub node: Option<Rc<RefCell<Node<'a>>>>,

    marker: PhantomData<&'a ()>,
}

impl<'a> SkeletonBone<'a> {
    pub fn borrow_from(
        c: &'a mut russimp_sys::aiSkeletonBone,
        map: &HashMap<String, Rc<RefCell<Node<'a>>>>,
    ) -> SkeletonBone<'a> {
        let armature = match unsafe { c.mArmature.as_mut() } {
            Some(ai_armature) => match map.get(&get_node_path(ai_armature)) {
                Some(node) => Some(node.clone()),
                None => None,
            },
            None => None,
        };

        let node = match unsafe { c.mNode.as_mut() } {
            Some(ai_node) => match map.get(&get_node_path(ai_node)) {
                Some(node) => Some(node.clone()),
                None => None,
            },
            None => None,
        };

        SkeletonBone {
            c,
            marker: PhantomData,
            armature,
            node,
        }
    }
}
