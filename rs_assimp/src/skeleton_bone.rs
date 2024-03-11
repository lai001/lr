use crate::{convert::ConvertToString, node::Node};
use std::{cell::RefCell, collections::HashMap, marker::PhantomData, rc::Rc};

pub struct SkeletonBone<'a> {
    c: &'a mut russimp_sys::aiSkeletonBone,
    pub armature: Option<Rc<RefCell<Node<'a>>>>,
    pub node: Option<Rc<RefCell<Node<'a>>>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> SkeletonBone<'a> {
    pub fn borrow_from(c: &'a mut russimp_sys::aiSkeletonBone) -> SkeletonBone<'a> {
        SkeletonBone {
            c,
            marker: PhantomData,
            armature: None,
            node: None,
        }
    }

    pub fn execute(&mut self, map: &HashMap<String, Rc<RefCell<Node<'a>>>>) {
        match unsafe { self.c.mArmature.as_mut() } {
            Some(ai_armature) => match map.get(&ai_armature.mName.to_string()) {
                Some(node) => {
                    self.armature = Some(node.clone());
                }
                None => {
                    self.armature = None;
                }
            },
            None => {
                self.armature = None;
            }
        }

        match unsafe { self.c.mNode.as_mut() } {
            Some(ai_node) => match map.get(&ai_node.mName.to_string()) {
                Some(node) => {
                    self.node = Some(node.clone());
                }
                None => {
                    self.node = None;
                }
            },
            None => {
                self.node = None;
            }
        }
    }
}
