use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use serde::{Deserialize, Serialize};
use slotmap::{KeyData, SecondaryMap, SlotMap, new_key_type};
use std::collections::HashMap;

#[typetag::serde]
pub trait Node: erased_serde::Serialize + Downcast + DynClone {}
impl_downcast!(Node);
clone_trait_object!(Node);

#[repr(C)]
#[derive(Clone, Copy)]
struct Key {
    idx: u32,
    version: u32,
}

#[repr(C)]
union RawValue {
    key: Key,
    raw: u64,
}

new_key_type! {
    pub struct NodeId;
}

impl NodeId {
    fn idx(&self) -> u32 {
        let raw_value = RawValue {
            raw: self.0.as_ffi(),
        };
        unsafe { raw_value.key.idx }
    }

    fn from_idx(idx: u32) -> NodeId {
        let raw_value = RawValue {
            key: Key {
                idx: idx,
                version: 1,
            },
        };
        NodeId::from(KeyData::from_ffi(unsafe { raw_value.raw }))
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NodeRelation {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
}

impl NodeRelation {
    fn remap_idx(&self, new_idx_map: &HashMap<u32, u32>) -> NodeRelation {
        let mut new_one = NodeRelation {
            parent: None,
            children: Vec::with_capacity(self.children.len()),
        };
        if let Some(parent) = &self.parent {
            let parent = NodeId::from_idx(new_idx_map[&parent.idx()]);
            new_one.parent = Some(parent);
        }
        for child in &self.children {
            let child = NodeId::from_idx(new_idx_map[&child.idx()]);
            new_one.children.push(child);
        }
        new_one
    }
}

#[derive(Serialize, Deserialize)]
pub struct NodeTree {
    node_relations: SlotMap<NodeId, NodeRelation>,
    nodes: SecondaryMap<NodeId, Box<dyn Node>>,
}

impl NodeTree {
    pub fn new() -> Self {
        Self {
            node_relations: SlotMap::with_key(),
            nodes: SecondaryMap::new(),
        }
    }

    pub fn create_node(&mut self, parent: Option<NodeId>, node: impl Node) -> NodeId {
        let id = self.node_relations.insert(NodeRelation {
            parent,
            children: Vec::new(),
        });

        if let Some(p) = parent {
            self.node_relations[p].children.push(id);
        }

        self.nodes.insert(id, Box::new(node));

        id
    }

    pub fn remove_node(&mut self, id: NodeId) {
        let (parent, children) = match self.node_relations.get(id) {
            Some(node) => (node.parent, node.children.clone()),
            None => return,
        };

        for child in children {
            self.remove_node(child);
        }

        if let Some(p) = parent {
            if let Some(parent_node) = self.node_relations.get_mut(p) {
                parent_node.children.retain(|&c| c != id);
            }
        }

        self.node_relations.remove(id);
        self.nodes.remove(id);
    }

    pub fn remove_and_promote_children(&mut self, id: NodeId) -> Option<Box<dyn Node>> {
        let (parent, children) = match self.node_relations.get(id) {
            Some(node) => (node.parent, node.children.clone()),
            None => return None,
        };

        if let Some(p) = parent {
            if let Some(parent_node) = self.node_relations.get_mut(p) {
                parent_node.children.extend(children.iter().copied());
            }

            for child in &children {
                if let Some(child_node) = self.node_relations.get_mut(*child) {
                    child_node.parent = Some(p);
                }
            }
        } else {
            for child in &children {
                if let Some(child_node) = self.node_relations.get_mut(*child) {
                    child_node.parent = None;
                }
            }
        }

        if let Some(p) = parent {
            if let Some(parent_node) = self.node_relations.get_mut(p) {
                parent_node.children.retain(|&c| c != id);
            }
        }

        self.node_relations.remove(id);
        self.nodes.remove(id)
    }

    pub fn shrink(&self) -> NodeTree {
        assert_eq!(self.node_relations.len(), self.nodes.len());

        let mut new_idx_map: HashMap<u32, u32> = HashMap::new();
        for (index, (id, _)) in self.node_relations.iter().enumerate() {
            let new_idx = index + 1;
            new_idx_map.insert(id.idx(), new_idx as u32);
        }

        let mut pairs: Vec<(NodeRelation, Box<dyn Node>)> = Vec::new();

        for (id, node_relation) in &self.node_relations {
            let new_node = self.nodes[id].clone();
            let new_node_relation = node_relation.remap_idx(&new_idx_map);
            pairs.push((new_node_relation, new_node));
        }

        let mut node_relations: SlotMap<NodeId, NodeRelation> =
            SlotMap::with_capacity_and_key(self.node_relations.len());
        let mut nodes = SecondaryMap::with_capacity(self.nodes.len());

        for (node_relation, node) in pairs {
            let id = node_relations.insert(node_relation);
            nodes.insert(id, node);
        }

        NodeTree {
            node_relations,
            nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Clone)]
    struct TestNode {
        name: String,
    }

    impl TestNode {
        fn new() -> TestNode {
            TestNode {
                name: "".to_string(),
            }
        }

        fn from_name<S: Into<String>>(name: S) -> TestNode {
            TestNode { name: name.into() }
        }
    }
    #[typetag::serde]
    impl Node for TestNode {}

    fn setup_tree() -> NodeTree {
        NodeTree::new()
    }

    #[test]
    fn shirk() {
        let mut tree = setup_tree();
        let root = tree.create_node(None, TestNode::from_name("root"));
        let child = tree.create_node(Some(root), TestNode::from_name("child"));
        let _ = tree.create_node(Some(root), TestNode::from_name("child1"));
        tree.remove_node(child);
        let shrunk = tree.shrink();
        let mut has_root = false;
        let mut has_child1 = false;
        for (id, node) in &shrunk.nodes {
            let test_node = node.downcast_ref::<TestNode>().unwrap();
            if test_node.name == "root" {
                has_root = true;
                let node_relation = &shrunk.node_relations[id];
                assert_eq!(node_relation.children.len(), 1);
                let child1 = &shrunk.nodes[node_relation.children[0]];
                assert_eq!(child1.downcast_ref::<TestNode>().unwrap().name, "child1");
            } else if test_node.name == "child1" {
                has_child1 = true;
                let node_relation = &shrunk.node_relations[id];
                let parent = node_relation.parent.unwrap();
                let parent_node = &shrunk.nodes[parent];
                assert_eq!(parent_node.downcast_ref::<TestNode>().unwrap().name, "root");
            } else {
                assert!(false);
            }
        }
        assert!(has_root);
        assert!(has_child1);
    }

    #[test]
    fn test_create_node() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        assert!(tree.node_relations.get(root).is_some());
        assert_eq!(tree.node_relations[root].parent, None);
        assert!(tree.node_relations[root].children.is_empty());

        let child = tree.create_node(Some(root), TestNode::new());
        assert!(tree.node_relations.get(child).is_some());
        assert_eq!(tree.node_relations[child].parent, Some(root));
        assert_eq!(tree.node_relations[root].children, vec![child]);
    }

    #[test]
    fn test_remove_node_recursive() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let b = tree.create_node(Some(a), TestNode::new());
        let c = tree.create_node(Some(a), TestNode::new());

        tree.remove_node(a);

        assert!(tree.node_relations.get(a).is_none());
        assert!(tree.node_relations.get(b).is_none());
        assert!(tree.node_relations.get(c).is_none());

        assert!(tree.node_relations[root].children.is_empty());
    }

    #[test]
    fn test_remove_and_promote_children() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let b = tree.create_node(Some(a), TestNode::new());
        let c = tree.create_node(Some(a), TestNode::new());

        tree.remove_and_promote_children(a);

        assert!(tree.node_relations.get(a).is_none());

        assert!(tree.node_relations.get(b).is_some());
        assert!(tree.node_relations.get(c).is_some());

        assert_eq!(tree.node_relations[b].parent, Some(root));
        assert_eq!(tree.node_relations[c].parent, Some(root));

        let mut children = tree.node_relations[root].children.clone();
        children.sort();
        let mut expected = vec![b, c];
        expected.sort();
        assert_eq!(children, expected);
    }

    #[test]
    fn test_remove_and_promote_children_on_root() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let b = tree.create_node(Some(root), TestNode::new());

        tree.remove_and_promote_children(root);

        assert!(tree.node_relations.get(root).is_none());

        assert_eq!(tree.node_relations[a].parent, None);
        assert_eq!(tree.node_relations[b].parent, None);
    }

    #[test]
    fn test_promote_children_basic() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let b = tree.create_node(Some(a), TestNode::new());
        let c = tree.create_node(Some(a), TestNode::new());

        tree.remove_and_promote_children(a);

        assert!(tree.node_relations.get(a).is_none());
        assert_eq!(tree.node_relations[b].parent, Some(root));
        assert_eq!(tree.node_relations[c].parent, Some(root));

        let mut children = tree.node_relations[root].children.clone();
        children.sort();
        assert_eq!(children, vec![b, c]);
    }

    #[test]
    fn test_promote_children_single_child() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let b = tree.create_node(Some(a), TestNode::new());

        tree.remove_and_promote_children(a);

        assert!(tree.node_relations.get(a).is_none());
        assert_eq!(tree.node_relations[b].parent, Some(root));
        assert_eq!(tree.node_relations[root].children, vec![b]);
    }

    #[test]
    fn test_promote_children_no_children() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());

        tree.remove_and_promote_children(a);

        assert!(tree.node_relations.get(a).is_none());
        assert!(tree.node_relations[root].children.is_empty());
    }

    #[test]
    fn test_promote_children_root() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let b = tree.create_node(Some(root), TestNode::new());

        tree.remove_and_promote_children(root);

        assert!(tree.node_relations.get(root).is_none());
        assert_eq!(tree.node_relations[a].parent, None);
        assert_eq!(tree.node_relations[b].parent, None);

        let mut roots = vec![a, b];
        roots.sort();

        let mut actual: Vec<NodeId> = tree
            .node_relations
            .iter()
            .filter(|(_, n)| n.parent.is_none())
            .map(|(id, _)| id)
            .collect();
        actual.sort();

        assert_eq!(actual, roots);
    }

    #[test]
    fn test_promote_children_preserves_grandchildren() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let b = tree.create_node(Some(a), TestNode::new());
        let c = tree.create_node(Some(b), TestNode::new());

        tree.remove_and_promote_children(a);

        assert!(tree.node_relations.get(a).is_none());
        assert_eq!(tree.node_relations[b].parent, Some(root));
        assert_eq!(tree.node_relations[c].parent, Some(b));
    }

    #[test]
    fn test_promote_children_does_not_affect_siblings() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let x = tree.create_node(Some(root), TestNode::new());
        let b = tree.create_node(Some(a), TestNode::new());

        tree.remove_and_promote_children(a);

        assert!(tree.node_relations.get(a).is_none());
        assert_eq!(tree.node_relations[b].parent, Some(root));
        assert_eq!(tree.node_relations[x].parent, Some(root));

        let mut children = tree.node_relations[root].children.clone();
        children.sort();

        let mut expected = vec![b, x];
        expected.sort();

        assert_eq!(children, expected);
    }

    #[test]
    fn test_promote_children_no_cycles() {
        let mut tree = setup_tree();

        let root = tree.create_node(None, TestNode::new());
        let a = tree.create_node(Some(root), TestNode::new());
        let _ = tree.create_node(Some(a), TestNode::new());

        tree.remove_and_promote_children(a);

        for (id, node) in tree.node_relations.iter() {
            assert_ne!(node.parent, Some(id));
        }
    }
}
