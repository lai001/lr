use crate::{
    content::content_file_type::EContentFileType,
    drawable::EDrawObjectType,
    engine::Engine,
    misc,
    player_viewport::PlayerViewport,
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
        engine: &mut Engine,
        files: &[EContentFileType],
        player_viewport: &mut PlayerViewport,
    ) {
        Actor::walk_node(self.scene_node.clone(), &mut |node| {
            node.borrow_mut().initialize(engine, files, player_viewport);
        });
        self.update_components_world_transformation();
    }

    pub fn initialize_physics(
        &mut self,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        Actor::walk_node(self.scene_node.clone(), &mut |node| {
            node.borrow_mut()
                .initialize_physics(rigid_body_set, collider_set);
        });
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
                EComponentType::CollisionComponent(component) => {
                    let component = component.borrow();
                    let mut sub_draw_objects: Vec<_> = component
                        .get_draw_objects()
                        .iter()
                        .map(|x| (*x).clone())
                        .collect();
                    draw_objects.append(&mut sub_draw_objects);
                }
                EComponentType::SpotLightComponent(_) => {}
                EComponentType::PointLightComponent(_) => {}
            },
        );
        draw_objects
    }

    pub fn tick(
        &mut self,
        time: f32,
        engine: &mut Engine,
        rigid_body_set: &mut RigidBodySet,
        collider_set: &mut ColliderSet,
    ) {
        self.update_components_world_transformation();

        Actor::walk_node(self.scene_node.clone(), {
            &mut |node| {
                let mut node = node.borrow_mut();
                node.tick(time, engine, rigid_body_set, collider_set);
            }
        });
    }

    // pub fn tick_physics(
    //     &mut self,
    //     rigid_body_set: &mut RigidBodySet,
    //     collider_set: &mut ColliderSet,
    // ) {
    //     Actor::walk_node(self.scene_node.clone(), {
    //         &mut |node| {
    //             let node = node.borrow_mut();
    //             match &node.component {
    //                 EComponentType::SceneComponent(_) => {}
    //                 EComponentType::StaticMeshComponent(_) => {}
    //                 EComponentType::SkeletonMeshComponent(component) => {
    //                     let mut component = component.borrow_mut();
    //                     component.update_physics(rigid_body_set, collider_set);
    //                 }
    //                 EComponentType::CameraComponent(_) => {}
    //                 EComponentType::CollisionComponent(_) => {}
    //             }
    //         }
    //     });
    // }

    pub fn set_world_transformation_recursion(
        scene_node: &mut SceneNode,
        parent_transformation: glam::Mat4,
    ) {
        let current_transformation = scene_node.get_transformation();
        let final_transformation = parent_transformation * current_transformation;
        scene_node.set_parent_final_transformation(parent_transformation);
        scene_node.set_final_transformation(final_transformation);

        for child in scene_node.childs.clone() {
            let parent_transformation = final_transformation;
            Self::set_world_transformation_recursion(
                &mut child.borrow_mut(),
                parent_transformation,
            );
        }
    }

    pub fn update_components_world_transformation(&mut self) {
        let parent_transformation = glam::Mat4::IDENTITY;
        Self::set_world_transformation_recursion(
            &mut self.scene_node.borrow_mut(),
            parent_transformation,
        );
    }

    pub fn on_post_update_transformation_recursion(
        scene_node: &mut SceneNode,
        level_physics: Option<&mut crate::content::level::Physics>,
    ) {
        if let Some(level_physics) = level_physics {
            scene_node.on_post_update_transformation(Some(level_physics));
            for child in scene_node.childs.clone() {
                Self::on_post_update_transformation_recursion(
                    &mut child.borrow_mut(),
                    Some(level_physics),
                );
            }
        } else {
            scene_node.on_post_update_transformation(None);
            for child in scene_node.childs.clone() {
                Self::on_post_update_transformation_recursion(&mut child.borrow_mut(), None);
            }
        }
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

    pub fn find_node_by_collider_handle(
        &self,
        collider: &rapier3d::prelude::ColliderHandle,
    ) -> Option<SingleThreadMutType<SceneNode>> {
        let mut find_node = None;
        Self::walk_node(self.scene_node.clone(), &mut |scene_node| {
            if find_node.is_some() {
                return;
            }
            let is_contain: bool = (|| {
                let scene_node = scene_node.borrow();
                match &scene_node.component {
                    EComponentType::SceneComponent(_) => {
                        return false;
                    }
                    EComponentType::StaticMeshComponent(component) => {
                        let component = component.borrow();
                        let collider_handles = component
                            .get_physics()
                            .map(|x| x.collider_handles.clone())
                            .unwrap_or_default();
                        if collider_handles.contains(collider) {
                            return true;
                        }
                    }
                    EComponentType::SkeletonMeshComponent(component) => {
                        let component = component.borrow();
                        let collider_handles = component
                            .get_physics()
                            .map(|x| x.collider_handles.clone())
                            .unwrap_or_default();
                        if collider_handles.contains(collider) {
                            return true;
                        }
                    }
                    EComponentType::CameraComponent(_) => {
                        return false;
                    }
                    EComponentType::CollisionComponent(component) => {
                        let component = component.borrow();
                        let collider_handles = component
                            .get_physics()
                            .map(|x| x.collider_handles.clone())
                            .unwrap_or_default();
                        if collider_handles.contains(collider) {
                            return true;
                        }
                    }
                    EComponentType::SpotLightComponent(_) => return false,
                    EComponentType::PointLightComponent(_) => return false,
                }
                false
            })();
            if is_contain && find_node.is_none() {
                find_node = Some(scene_node.clone());
            }
        });
        find_node
    }

    pub fn compute_components_aabb(&self) -> Option<rapier3d::prelude::Aabb> {
        let mut aabbs: Vec<rapier3d::prelude::Aabb> = vec![];
        Self::walk_node(self.scene_node.clone(), &mut |node| {
            if let Some(aabb) = node.borrow().get_aabb() {
                aabbs.push(aabb);
            }
        });
        misc::merge_aabb(&aabbs)
    }

    pub fn copy_without_initialization(&self, name: String) -> Actor {
        let copy_root_scene_node = Self::copy_recursion(&self.scene_node.borrow());
        let copy_actor = Actor {
            name,
            scene_node: SingleThreadMut::new(copy_root_scene_node),
        };
        copy_actor
    }

    fn copy_recursion(scene_node: &SceneNode) -> SceneNode {
        let mut copy_scene_node = scene_node.clone();
        copy_scene_node.component = copy_scene_node.component.copy();
        copy_scene_node.childs.clear();
        for child in &scene_node.childs {
            let copy_node = Self::copy_recursion(&child.borrow());
            copy_scene_node.childs.push(SingleThreadMut::new(copy_node));
        }
        copy_scene_node
    }
}
