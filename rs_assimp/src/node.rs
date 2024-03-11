use crate::{convert::ConvertToString, metadata::Metadata};
use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    marker::PhantomData,
    rc::{Rc, Weak},
};

pub struct Node<'a> {
    c: &'a mut russimp_sys::aiNode,
    pub parent: Option<Weak<RefCell<Node<'a>>>>,
    pub name: String,
    pub children: Vec<Rc<RefCell<Node<'a>>>>,
    pub metadata: Option<Metadata<'a>>,
    marker: PhantomData<&'a ()>,
}

impl<'a> Hash for Node<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:p}", self.c).hash(state);
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Self) -> bool {
        format!("{:p}", self.c) == format!("{:p}", other)
    }
}

impl<'a> Eq for Node<'a> {}

impl<'a> Node<'a> {
    pub fn new(c: &'a mut russimp_sys::aiNode) -> Node<'a> {
        let name = c.mName.to_string();
        let metadata = match unsafe { c.mMetaData.as_mut() } {
            Some(m_meta_data) => Some(Metadata::borrow_from(m_meta_data)),
            None => None,
        };
        Node {
            c,
            parent: None,
            name,
            metadata,
            children: vec![],
            marker: PhantomData,
        }
    }

    pub fn parent_and_leaf(&mut self, map: &mut HashMap<String, Rc<RefCell<Node<'a>>>>) {
        match unsafe { self.c.mParent.as_mut() } {
            Some(parent) => {
                let mut h = DefaultHasher::new();
                format!("{:p}", parent).hash(&mut h);
                let key = h.finish().to_string();
                match map.get_mut(&key) {
                    Some(parent_node) => {
                        self.parent = Some(Rc::downgrade(parent_node));
                    }
                    None => {}
                }
            }
            None => {}
        }
        self.children.clear();
        let children = rs_foundation::get_vec_from_raw_mut(self.c.mChildren, self.c.mNumChildren);
        for item in children {
            let mut h = DefaultHasher::new();
            format!("{:p}", item).hash(&mut h);
            let key = h.finish().to_string();
            match map.get_mut(&key) {
                Some(node) => {
                    self.children.push(node.clone());
                }
                None => {}
            }
        }
    }
}
