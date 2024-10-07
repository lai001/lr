use crate::{
    content::content_file_type::EContentFileType,
    drawable::EDrawObjectType,
    engine::Engine,
    player_viewport::PlayerViewport,
    resource_manager::ResourceManager,
    scene_node::{EComponentType, SceneNode},
};
use rapier3d::prelude::{ColliderSet, RigidBodySet};
use rs_foundation::new::{SingleThreadMut, SingleThreadMutType};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, rc::Rc};

#[derive(Serialize, Deserialize, Clone)]
pub struct Actor {
    pub name: String,
    pub scene_node: SingleThreadMutType<SceneNode>,
}

impl Actor {
    pub fn new(name: String) -> Actor {
        let scene_node = SceneNode::new_sp("Scene".to_string());
        Actor { name, scene_node }
    }

    pub fn new_sp(name: String) -> SingleThreadMutType<Actor> {
        SingleThreadMut::new(Self::new(name))
    }

    pub fn walk_node(
        node: SingleThreadMutType<SceneNode>,
        walk: &mut impl FnMut(SingleThreadMutType<SceneNode>) -> (),
    ) {
        walk(node.clone());
        let node = node.borrow();
        for child in node.childs.clone() {
            Self::walk_node(child, walk);
        }
    }

    pub fn initialize(
        &mut self,
        resource_manager: ResourceManager,
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        Actor::walk_node(
            self.scene_node.clone(),
            &mut |node| match &node.borrow().component {
                EComponentType::SceneComponent(component) => {
                    let mut component = component.borrow_mut();
                    component.initialize();
                }
                EComponentType::StaticMeshComponent(component) => {
                    let mut component = component.borrow_mut();
                    component.initialize(resource_manager.clone(), engine, files, player_viewport);
                }
                EComponentType::SkeletonMeshComponent(component) => {
                    let mut component = component.borrow_mut();
                    component.initialize(resource_manager.clone(), engine, files, player_viewport);
                }
                EComponentType::CameraComponent(component) => {
                    let mut component = component.borrow_mut();
                    component.initialize(engine, player_viewport);
                }
            },
        );
        self.update_components_world_transformation();
    }

    pub fn initialize_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        Actor::walk_node(
            self.scene_node.clone(),
            &mut |node| match &node.borrow().component {
                EComponentType::SceneComponent(_) => {}
                EComponentType::StaticMeshComponent(component) => {
                    let mut component = component.borrow_mut();
                    component.init_physics(rigid_body_set, collider_set)
                }
                EComponentType::SkeletonMeshComponent(component) => {
                    let mut component = component.borrow_mut();
                    component.init_physics(rigid_body_set, collider_set)
                }
                EComponentType::CameraComponent(_) => {}
            },
        );
    }

    pub fn collect_draw_objects(&self) -> Vec<EDrawObjectType> {
        let mut draw_objects = vec![];
        Actor::walk_node(
            self.scene_node.clone(),
            &mut |node| match &node.borrow().component {
                EComponentType::SceneComponent(_) => {}
                EComponentType::StaticMeshComponent(component) => {
                    let component = component.borrow();
                    let mut sub_draw_objects: Vec<_> = component
                        .get_draw_objects()
                        .iter()
                        .map(|x| (*x).clone())
                        .collect();
                    draw_objects.append(&mut sub_draw_objects);
                }
                EComponentType::SkeletonMeshComponent(component) => {
                    let component = component.borrow();
                    let mut sub_draw_objects: Vec<_> = component
                        .get_draw_objects()
                        .iter()
                        .map(|x| (*x).clone())
                        .collect();
                    draw_objects.append(&mut sub_draw_objects);
                }
                EComponentType::CameraComponent(component) => {
                    let component = component.borrow();
                    let mut sub_draw_objects: Vec<_> = component
                        .get_draw_objects()
                        .iter()
                        .map(|x| (*x).clone())
                        .collect();
                    draw_objects.append(&mut sub_draw_objects);
                }
            },
        );
        draw_objects
    }

    pub fn tick(
        &mut self,
        time: f32,
        engine: &mut Engine,
        rigid_body_set: Option<&mut RigidBodySet>,
    ) {
        self.update_components_world_transformation();

        match rigid_body_set {
            Some(rigid_body_set) => {
                Actor::walk_node(self.scene_node.clone(), {
                    &mut |node| {
                        let node = node.borrow_mut();
                        match &node.component {
                            EComponentType::SceneComponent(_) => {}
                            EComponentType::StaticMeshComponent(component) => {
                                let mut component = component.borrow_mut();
                                component.update(time, engine, Some(rigid_body_set));
                            }
                            EComponentType::SkeletonMeshComponent(component) => {
                                let mut component = component.borrow_mut();
                                component.update(time, engine);
                            }
                            EComponentType::CameraComponent(component) => {
                                let mut component = component.borrow_mut();
                                component.update(time, engine);
                            }
                        }
                    }
                });
            }
            None => {
                Actor::walk_node(self.scene_node.clone(), {
                    &mut |node| {
                        let node = node.borrow_mut();
                        match &node.component {
                            EComponentType::SceneComponent(_) => {}
                            EComponentType::StaticMeshComponent(component) => {
                                let mut component = component.borrow_mut();
                                component.update(time, engine, None);
                            }
                            EComponentType::SkeletonMeshComponent(component) => {
                                let mut component = component.borrow_mut();
                                component.update(time, engine);
                            }
                            EComponentType::CameraComponent(component) => {
                                let mut component = component.borrow_mut();
                                component.update(time, engine);
                            }
                        }
                    }
                });
            }
        }
    }

    pub fn tick_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        Actor::walk_node(self.scene_node.clone(), {
            &mut |node| {
                let node = node.borrow_mut();
                match &node.component {
                    EComponentType::SceneComponent(_) => {}
                    EComponentType::StaticMeshComponent(_) => {}
                    EComponentType::SkeletonMeshComponent(component) => {
                        let mut component = component.borrow_mut();
                        component.update_physics(rigid_body_set, collider_set);
                    }
                    EComponentType::CameraComponent(_) => {}
                }
            }
        });
    }

    fn set_world_transformation_recursion(
        scene_node: SingleThreadMutType<SceneNode>,
        parent_transformation: glam::Mat4,
    ) {
        let scene_node = scene_node.borrow();
        let component_type = &scene_node.component;
        let final_transformation = match component_type {
            EComponentType::SceneComponent(component) => {
                let mut component = component.borrow_mut();
                let current_transformation = *component.get_transformation();
                let final_transformation = parent_transformation * current_transformation;
                component.set_parent_final_transformation(parent_transformation);
                component.set_final_transformation(final_transformation);
                final_transformation
            }
            EComponentType::StaticMeshComponent(component) => {
                let mut component = component.borrow_mut();
                let current_transformation = *component.get_transformation();
                let final_transformation = parent_transformation * current_transformation;
                component.set_parent_final_transformation(parent_transformation);
                component.set_final_transformation(final_transformation);
                final_transformation
            }
            EComponentType::SkeletonMeshComponent(component) => {
                let mut component = component.borrow_mut();
                let current_transformation = *component.get_transformation();
                let final_transformation = parent_transformation * current_transformation;
                component.set_parent_final_transformation(parent_transformation);
                component.set_final_transformation(final_transformation);
                final_transformation
            }
            EComponentType::CameraComponent(component) => {
                let mut component = component.borrow_mut();
                let current_transformation = *component.get_transformation();
                let final_transformation = parent_transformation * current_transformation;
                component.set_parent_final_transformation(parent_transformation);
                component.set_final_transformation(final_transformation);
                final_transformation
            }
        };

        for child in scene_node.childs.clone() {
            let parent_transformation = final_transformation;
            Self::set_world_transformation_recursion(child, parent_transformation);
        }
    }

    pub fn update_components_world_transformation(&mut self) {
        let parent_transformation = glam::Mat4::IDENTITY;
        Self::set_world_transformation_recursion(self.scene_node.clone(), parent_transformation);
    }

    pub fn remove_node(&mut self, node_will_remove: SingleThreadMutType<SceneNode>) {
        if Rc::ptr_eq(&self.scene_node, &node_will_remove) {
            return;
        }
        Actor::walk_node(self.scene_node.clone(), &mut move |node| {
            let mut node = node.borrow_mut();
            node.childs
                .retain(|element| !Rc::ptr_eq(element, &node_will_remove));
        });
    }

    pub fn find_path_by_node(&self, node: SingleThreadMutType<SceneNode>) -> Option<String> {
        let map = self.collect_node_map();
        for (path, item) in map {
            if Rc::ptr_eq(&item, &node) {
                return Some(path);
            }
        }
        return None;
    }

    pub fn find_node_by_path(
        &self,
        path: impl AsRef<str>,
    ) -> Option<SingleThreadMutType<SceneNode>> {
        let map = self.collect_node_map();
        map.get(path.as_ref()).cloned()
    }

    pub fn collect_node_map(&self) -> HashMap<String, SingleThreadMutType<SceneNode>> {
        let mut node_map = HashMap::new();
        Self::collect_node_map_internal("", self.scene_node.clone(), &mut node_map);
        node_map
    }

    fn collect_node_map_internal(
        parent_path: impl AsRef<str>,
        node: SingleThreadMutType<SceneNode>,
        node_map: &mut HashMap<String, SingleThreadMutType<SceneNode>>,
    ) {
        let path = {
            let node = node.borrow();
            format!("{}/{}", parent_path.as_ref(), node.get_name())
        };
        node_map.insert(path.clone(), node.clone());
        let node = node.borrow();
        for child in node.childs.clone() {
            Self::collect_node_map_internal(&path, child.clone(), node_map);
        }
    }
}
