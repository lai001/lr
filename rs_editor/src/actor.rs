use crate::scene_node::SceneNode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Actor {
    pub name: String,
    pub scene_node: SceneNode,
}
