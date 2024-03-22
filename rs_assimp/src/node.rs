use crate::{
    convert::{ConvertToMat4, ConvertToString},
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

pub struct Node<'a> {
    c: &'a mut russimp_sys::aiNode,
    pub parent: Option<Weak<RefCell<Node<'a>>>>,
    pub name: String,
    pub path: String,
    pub children: Vec<Rc<RefCell<Node<'a>>>>,
    pub metadata: Option<Metadata<'a>>,
    pub transformation: glam::Mat4,
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
            parent: None,
            name,
            path,
            children: vec![],
            metadata,
            transformation,
            marker: PhantomData,
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
}
